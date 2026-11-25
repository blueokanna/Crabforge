use anyhow::{Result, ensure};
use reqwest::Url;
use std::collections::BTreeSet;

/// Store origins in the form a browser sends in its Origin header.
pub(crate) fn clean(origins: &BTreeSet<String>) -> Result<BTreeSet<String>> {
    origins
        .iter()
        .map(|raw| raw.trim())
        .filter(|s| !s.is_empty())
        .map(normalize)
        .collect()
}

fn normalize(value: &str) -> Result<String> {
    let error = || {
        anyhow::anyhow!(
            "invalid browser origin {value:?}; enter a web or extension origin without a path"
        )
    };
    let text = value.strip_suffix('/').unwrap_or(value);
    let (scheme, authority) = text.split_once("://").ok_or_else(error)?;
    ensure!(
        !authority.is_empty()
            && !authority.contains(['*', '/', '\\', '@', '?', '#', '%'])
            && !authority
                .chars()
                .any(|c| c.is_control() || c.is_whitespace()),
        error()
    );
    let scheme = scheme.to_ascii_lowercase();
    let host = authority.to_ascii_lowercase();
    let valid = match scheme.as_str() {
        "http" | "https" => return Ok(Url::parse(text)?.origin().ascii_serialization()),
        "chrome-extension" => host.len() == 32 && host.bytes().all(|b| (b'a'..=b'p').contains(&b)),
        "moz-extension" | "safari-web-extension" => {
            host.len() == 36
                && host.bytes().enumerate().all(|(i, b)| {
                    if matches!(i, 8 | 13 | 18 | 23) {
                        b == b'-'
                    } else {
                        b.is_ascii_hexdigit()
                    }
                })
        }
        _ => false,
    };
    ensure!(valid, error());
    Ok(format!("{scheme}://{host}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn origins_validate_extension_ids_and_keep_nondefault_ports() {
        let clean_one = |s: &str| clean(&BTreeSet::from([s.to_owned()]));
        for (s, expected) in [
            ("HTTPS://Example.Org:443/", "https://example.org"),
            ("http://[::1]:3425", "http://[::1]:3425"),
            (
                "chrome-extension://ABCDEFGHIJKLMNOPabcdefghijklmnop/",
                "chrome-extension://abcdefghijklmnopabcdefghijklmnop",
            ),
            (
                "moz-extension://01234567-89ab-cdef-0123-456789abcdef",
                "moz-extension://01234567-89ab-cdef-0123-456789abcdef",
            ),
        ] {
            assert_eq!(clean_one(s).unwrap(), BTreeSet::from([expected.to_owned()]));
        }
        for s in [
            "*",
            "https://*.example.org",
            "https://user@example.org",
            "https://example.org/path",
            "http://example.org?key=value",
            "null",
            "chrome-extension://short",
            "safari-web-extension://wrong",
            "file:///a",
        ] {
            assert!(clean_one(s).is_err(), "{s}");
        }
    }
}
