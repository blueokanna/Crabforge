//! Multiaddr encoding for Crabforge's libp2p transport and peer discovery.

use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr,
    sync::Arc,
};

mod protocol;

pub use libp2p_identity::PeerId;
pub use protocol::Protocol;

#[macro_export]
macro_rules! multiaddr {
    ($($variant:ident $(($value:expr))?),* $(,)?) => {
        $crate::Multiaddr::from_iter([
            $($crate::Protocol::$variant $(($value.into()))?),*
        ])
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    UnsupportedProtocol,
    InvalidComponent,
    InvalidAddress,
    InvalidEncoding,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedProtocol => "unsupported multiaddr protocol",
            Self::InvalidComponent => "invalid multiaddr component",
            Self::InvalidAddress => "invalid multiaddr syntax",
            Self::InvalidEncoding => "invalid multiaddr binary encoding",
        })
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Multiaddr(Option<Arc<Vec<u8>>>);

impl Multiaddr {
    pub const fn empty() -> Self {
        Self(None)
    }

    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter(self.as_ref())
    }

    pub fn push(&mut self, part: Protocol<'_>) {
        let bytes = self.0.get_or_insert_with(|| Arc::new(Vec::new()));
        part.encode(Arc::make_mut(bytes));
    }

    pub fn pop(&mut self) -> Option<Protocol<'static>> {
        let mut iter = self.iter();
        let mut start = 0;
        let mut last = None;
        while let Some(part) = iter.next() {
            last = Some((start, part));
            start = self.len() - iter.0.len();
        }
        let (start, last) = last?;
        let last = last.into_owned();
        if start == 0 {
            self.0 = None;
        } else if let Some(bytes) = &mut self.0 {
            Arc::make_mut(bytes).truncate(start);
        }
        Some(last)
    }

    pub fn with(mut self, part: Protocol<'_>) -> Self {
        self.push(part);
        self
    }

    pub fn with_p2p(self, peer: PeerId) -> Result<Self, Self> {
        match self.iter().last() {
            Some(Protocol::P2p(id)) if id == peer => Ok(self),
            Some(Protocol::P2p(_)) => Err(self),
            _ => Ok(self.with(Protocol::P2p(peer))),
        }
    }

    pub fn replace<'a>(
        &'a self,
        index: usize,
        edit: impl FnOnce(Protocol<'a>) -> Option<Protocol<'a>>,
    ) -> Option<Self> {
        let mut parts = self.iter();
        if index > 0 {
            parts.nth(index - 1)?;
        }
        let start = self.len() - parts.0.len();
        let next = edit(parts.next()?)?;
        let mut bytes = Vec::with_capacity(self.len());
        bytes.extend_from_slice(&self.as_ref()[..start]);
        next.encode(&mut bytes);
        bytes.extend_from_slice(parts.0);
        Some(Self(Some(Arc::new(bytes))))
    }
}

impl AsRef<[u8]> for Multiaddr {
    fn as_ref(&self) -> &[u8] {
        self.0.as_deref().map_or(&[], Vec::as_slice)
    }
}

impl fmt::Display for Multiaddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in self {
            part.fmt(f)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Multiaddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FromStr for Multiaddr {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Error> {
        if input.is_empty() {
            return Ok(Self::empty());
        }
        let mut parts = input
            .strip_prefix('/')
            .ok_or(Error::InvalidAddress)?
            .split('/');
        let mut bytes = Vec::with_capacity(input.len());
        while let Some(name) = parts.next() {
            Protocol::parse(name, &mut parts)?.encode(&mut bytes);
        }
        Ok(Self(Some(Arc::new(bytes))))
    }
}

impl TryFrom<Vec<u8>> for Multiaddr {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Error> {
        let mut rest = bytes.as_slice();
        while !rest.is_empty() {
            let (part, tail) = Protocol::decode(rest)?;
            part.validate_text_component()?;
            rest = tail;
        }
        Ok(Self((!bytes.is_empty()).then(|| Arc::new(bytes))))
    }
}

impl From<IpAddr> for Multiaddr {
    fn from(ip: IpAddr) -> Self {
        match ip {
            IpAddr::V4(ip) => ip.into(),
            IpAddr::V6(ip) => ip.into(),
        }
    }
}

impl From<Ipv4Addr> for Multiaddr {
    fn from(ip: Ipv4Addr) -> Self {
        Self::empty().with(Protocol::Ip4(ip))
    }
}

impl From<Ipv6Addr> for Multiaddr {
    fn from(ip: Ipv6Addr) -> Self {
        Self::empty().with(Protocol::Ip6(ip))
    }
}

impl From<Protocol<'_>> for Multiaddr {
    fn from(part: Protocol<'_>) -> Self {
        Self::empty().with(part)
    }
}

pub struct Iter<'a>(&'a [u8]);

impl<'a> Iterator for Iter<'a> {
    type Item = Protocol<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }
        let (part, rest) = Protocol::decode(self.0).expect("validated multiaddr encoding");
        self.0 = rest;
        Some(part)
    }
}

impl<'a> IntoIterator for &'a Multiaddr {
    type Item = Protocol<'a>;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> FromIterator<Protocol<'a>> for Multiaddr {
    fn from_iter<T: IntoIterator<Item = Protocol<'a>>>(parts: T) -> Self {
        let mut address = Self::empty();
        for part in parts {
            address.push(part);
        }
        address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quic_addresses_match_the_wire_format() {
        let v4 = "/ip4/127.0.0.1/udp/4001/quic-v1";
        let v6 = "/ip6/::1/udp/4001/quic-v1";
        for (text, bytes) in [
            (v4, vec![4, 127, 0, 0, 1, 145, 2, 15, 161, 205, 3]),
            (
                v6,
                vec![
                    41, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 145, 2, 15, 161, 205, 3,
                ],
            ),
        ] {
            let address: Multiaddr = text.parse().unwrap();
            assert_eq!(address.as_ref(), bytes);
            assert_eq!(Multiaddr::try_from(bytes).unwrap().to_string(), text);
        }
    }

    #[test]
    fn peer_id_and_circuit_roundtrip_with_component_edits() {
        let id = "QmYwAPJzv5CZsnAzt8auVZRnLQPyxWLz4vF6pSRahFhXS9";
        let address: Multiaddr =
            format!("/dns4/relay.example/tcp/443/wss/p2p/{id}/p2p-circuit/p2p/{id}")
                .parse()
                .unwrap();
        let mut copy = Multiaddr::try_from(address.to_vec()).unwrap();
        assert_eq!(address, copy);
        assert_eq!(copy.pop(), Some(Protocol::P2p(id.parse().unwrap())));
        let other = copy.replace(1, |_| Some(Protocol::Tcp(8443))).unwrap();
        assert_eq!(
            other.to_string(),
            format!("/dns4/relay.example/tcp/8443/wss/p2p/{id}/p2p-circuit")
        );
        assert_eq!(copy.iter().nth(1), Some(Protocol::Tcp(443)));
        assert!(copy.replace(99, Some).is_none());
        let same: PeerId = id.parse().unwrap();
        let different = PeerId::from_bytes(&[0, 1, 1]).unwrap();
        assert_eq!(address.clone().with_p2p(same).unwrap(), address);
        assert_eq!(address.clone().with_p2p(different).unwrap_err(), address);
        assert_eq!(
            Multiaddr::empty().with_p2p(different).unwrap().as_ref(),
            [165, 3, 3, 0, 1, 1]
        );
    }

    #[test]
    fn rejects_truncated_and_noncanonical_wire_input() {
        for bytes in [
            vec![4, 127, 0, 0],
            vec![132, 0, 127, 0, 0, 1],
            vec![53, 3, 0xFF, 0xFE, 0x80],
            vec![53, 12, b'a'],
            vec![53, 0],
            vec![53, 3, b'a', b'/', b'b'],
            vec![165, 3, 3, 0, 0, 0],
            vec![165, 3, 3, 1, 1, 0],
        ] {
            assert!(Multiaddr::try_from(bytes).is_err());
        }
        assert!("/ip/127.0.0.1".parse::<Multiaddr>().is_err());
        assert!("/ip4/127.0.0.1/udp/65536".parse::<Multiaddr>().is_err());
    }
}
