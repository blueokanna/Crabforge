use std::{
    borrow::Cow,
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use multihash::Multihash;
use unsigned_varint::{decode, encode};

use crate::{Error, PeerId};

struct Wire<'a>(&'a [u8]);

impl<'a> Wire<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let (value, rest) = self.0.split_at_checked(len).ok_or(Error::InvalidEncoding)?;
        self.0 = rest;
        Ok(value)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        self.take(N)?.try_into().map_err(|_| Error::InvalidEncoding)
    }

    fn number(&mut self) -> Result<usize, Error> {
        let (value, rest) = decode::usize(self.0).map_err(|_| Error::InvalidEncoding)?;
        self.0 = rest;
        Ok(value)
    }

    fn payload(&mut self) -> Result<&'a [u8], Error> {
        let length = self.number()?;
        self.take(length)
    }

    fn text(&mut self) -> Result<Cow<'a, str>, Error> {
        Ok(Cow::Borrowed(
            std::str::from_utf8(self.payload()?).map_err(|_| Error::InvalidEncoding)?,
        ))
    }

    fn peer(&mut self) -> Result<PeerId, Error> {
        PeerId::from_bytes(self.payload()?).map_err(|_| Error::InvalidEncoding)
    }

    fn cert(&mut self) -> Result<Multihash<64>, Error> {
        Multihash::from_bytes(self.payload()?).map_err(|_| Error::InvalidEncoding)
    }
}

fn number(out: &mut Vec<u8>, value: usize) {
    out.extend_from_slice(encode::usize(value, &mut encode::usize_buffer()));
}

fn payload(out: &mut Vec<u8>, value: &[u8]) {
    number(out, value.len());
    out.extend_from_slice(value);
}

fn hash(out: &mut Vec<u8>, value: &Multihash<64>) {
    number(out, value.encoded_len());
    value.write(out).expect("writing a multihash to a vector");
}

fn component<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, Error> {
    parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or(Error::InvalidComponent)
}

macro_rules! parse_value {
    (text, $parts:ident) => {
        Cow::Borrowed(component($parts)?)
    };
    (cert, $parts:ident) => {{
        let (_, bytes) =
            multibase::decode(component($parts)?).map_err(|_| Error::InvalidComponent)?;
        Multihash::from_bytes(&bytes).map_err(|_| Error::InvalidComponent)?
    }};
    ($kind:ident, $parts:ident) => {
        FromStr::from_str(component($parts)?).map_err(|_| Error::InvalidComponent)?
    };
}

macro_rules! read_value {
    (ip, $wire:ident) => {
        $wire.fixed()?.into()
    };
    (integer, $wire:ident) => {
        u64::from_be_bytes($wire.fixed()?)
    };
    (port, $wire:ident) => {
        u16::from_be_bytes($wire.fixed()?)
    };
    ($kind:ident, $wire:ident) => {
        $wire.$kind()?
    };
}

macro_rules! write_value {
    (ip, $out:ident, $value:ident) => {
        $out.extend_from_slice(&$value.octets())
    };
    (integer, $out:ident, $value:ident) => {
        $out.extend_from_slice(&$value.to_be_bytes())
    };
    (port, $out:ident, $value:ident) => {
        $out.extend_from_slice(&$value.to_be_bytes())
    };
    (text, $out:ident, $value:ident) => {
        payload($out, $value.as_bytes())
    };
    (peer, $out:ident, $value:ident) => {
        hash($out, $value.as_ref())
    };
    (cert, $out:ident, $value:ident) => {
        hash($out, $value)
    };
}

macro_rules! owned_value {
    (text, $value:ident) => {
        Cow::Owned($value.into_owned())
    };
    ($kind:ident, $value:ident) => {
        $value
    };
}

macro_rules! display_value {
    (cert, $f:ident, $value:ident) => {
        write!(
            $f,
            "/{}",
            multibase::encode(multibase::Base::Base64Url, $value.to_bytes())
        )
    };
    ($kind:ident, $f:ident, $value:ident) => {
        write!($f, "/{}", $value)
    };
}

// One registry drives the text and wire codecs. Codes and field widths follow
// https://github.com/multiformats/multiaddr/blob/master/protocols.csv.
macro_rules! protocols {
    ($($code:literal $name:literal $variant:ident $(($ty:ty, $kind:ident, $value:ident))?;)*) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        #[non_exhaustive]
        pub enum Protocol<'a> {
            $($variant $(($ty))?,)*
        }

        impl<'a> Protocol<'a> {
            pub(crate) fn parse(name: &str, parts: &mut impl Iterator<Item = &'a str>) -> Result<Self, Error> {
                let name = if name == "ipfs" { "p2p" } else { name };
                Ok(match name {
                    $($name => Self::$variant $((parse_value!($kind, parts)))?,)*
                    _ => return Err(Error::UnsupportedProtocol),
                })
            }

            pub(crate) fn decode(bytes: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
                let mut wire = Wire(bytes);
                let part = match wire.number()? {
                    $($code => Self::$variant $((read_value!($kind, wire)))?,)*
                    _ => return Err(Error::UnsupportedProtocol),
                };
                Ok((part, wire.0))
            }

            pub(crate) fn encode(&self, out: &mut Vec<u8>) {
                match self {
                    $(Self::$variant $(($value))? => {
                        number(out, $code);
                        $(write_value!($kind, out, $value);)?
                    },)*
                }
            }

            pub(crate) fn into_owned(self) -> Protocol<'static> {
                match self {
                    $(Self::$variant $(($value))? => Protocol::$variant $((owned_value!($kind, $value)))?,)*
                }
            }
        }

        impl fmt::Display for Protocol<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Self::$variant $(($value))? => {
                        f.write_str(concat!("/", $name))?;
                        $(display_value!($kind, f, $value)?;)?
                        Ok(())
                    },)*
                }
            }
        }
    };
}

protocols! {
    4 "ip4" Ip4(Ipv4Addr, ip, value);
    6 "tcp" Tcp(u16, port, value);
    41 "ip6" Ip6(Ipv6Addr, ip, value);
    42 "ip6zone" Ip6zone(Cow<'a, str>, text, value);
    53 "dns" Dns(Cow<'a, str>, text, value);
    54 "dns4" Dns4(Cow<'a, str>, text, value);
    55 "dns6" Dns6(Cow<'a, str>, text, value);
    56 "dnsaddr" Dnsaddr(Cow<'a, str>, text, value);
    273 "udp" Udp(u16, port, value);
    280 "webrtc-direct" WebRTCDirect;
    281 "webrtc" WebRTC;
    290 "p2p-circuit" P2pCircuit;
    421 "p2p" P2p(PeerId, peer, value);
    443 "https" Https;
    448 "tls" Tls;
    449 "sni" Sni(Cow<'a, str>, text, value);
    454 "noise" Noise;
    460 "quic" Quic;
    461 "quic-v1" QuicV1;
    465 "webtransport" WebTransport;
    466 "certhash" Certhash(Multihash<64>, cert, value);
    477 "ws" Ws;
    478 "wss" Wss;
    480 "http" Http;
    777 "memory" Memory(u64, integer, value);
}

impl Protocol<'_> {
    pub(crate) fn validate_text_component(&self) -> Result<(), Error> {
        let value = match self {
            Self::Ip6zone(value)
            | Self::Dns(value)
            | Self::Dns4(value)
            | Self::Dns6(value)
            | Self::Dnsaddr(value)
            | Self::Sni(value) => value,
            _ => return Ok(()),
        };
        if value.is_empty() || value.contains('/') {
            return Err(Error::InvalidEncoding);
        }
        Ok(())
    }
}

impl From<IpAddr> for Protocol<'_> {
    fn from(ip: IpAddr) -> Self {
        match ip {
            IpAddr::V4(ip) => Self::Ip4(ip),
            IpAddr::V6(ip) => Self::Ip6(ip),
        }
    }
}
