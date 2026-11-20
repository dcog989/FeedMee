use std::net::IpAddr;

use url::Url;

/// URLs the app is willing to fetch: plain HTTP(S) on a host that is not local
/// or private. Feed content decides these URLs, so anything that could reach
/// the user's machine or LAN is refused.
///
/// shortcut: host names are not resolved, so a public name that resolves to a
/// private address still gets through. Upgrade when that becomes the common
/// case instead of a hand-written IP.
pub(crate) fn is_public_url(raw: &str) -> bool {
    let Ok(url) = Url::parse(raw) else {
        return false;
    };
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }

    match url.host() {
        Some(url::Host::Ipv4(v4)) => is_public_ip(IpAddr::V4(v4)),
        Some(url::Host::Ipv6(v6)) => is_public_ip(IpAddr::V6(v6)),
        Some(url::Host::Domain(domain)) => is_public_hostname(domain),
        None => false,
    }
}

fn is_public_hostname(domain: &str) -> bool {
    let host = domain.trim_end_matches('.').to_ascii_lowercase();

    !(host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.ends_with(".home.arpa")
        // A name without a dot is resolved by the local resolver, not DNS.
        || !host.contains('.'))
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let raw = u32::from(v4);
            // `is_private` covers RFC1918 only. Reject the rest of the ranges
            // that reach the user's machine or VPN: this-network (0/8), carrier
            // NAT (100.64/10, used by Tailscale and some ISPs) and the
            // benchmarking range (198.18/15).
            !(v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_multicast()
                || raw >> 24 == 0
                || raw & 0xffc0_0000 == 0x6440_0000
                || raw & 0xfffe_0000 == 0xc612_0000)
        },
        IpAddr::V6(v6) => match v6.to_ipv4() {
            Some(v4) => is_public_ip(IpAddr::V4(v4)),
            None => {
                let segments = v6.segments();
                let unique_local = (segments[0] & 0xfe00) == 0xfc00;
                let link_local = (segments[0] & 0xffc0) == 0xfe80;
                !(v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() || unique_local || link_local)
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::is_public_url;

    #[test]
    fn accepts_public_http_urls() {
        assert!(is_public_url("https://example.com/feed.xml"));
        assert!(is_public_url("http://news.example.co.uk/pic.png"));
    }

    #[test]
    fn rejects_local_and_private_targets() {
        for url in [
            "http://localhost:8080/admin",
            "http://127.0.0.1/",
            "http://192.168.1.1/reboot",
            "http://10.0.0.5/",
            "http://169.254.169.254/latest/meta-data",
            "http://100.64.0.1/",
            "http://100.100.100.100/",
            "http://0.1.2.3/",
            "http://198.18.0.1/",
            "http://[::1]/",
            "http://[fd00::1]/",
            "http://[::ffff:127.0.0.1]/",
            "http://[::127.0.0.1]/",
            "http://router.local/",
            "http://nas/",
            "file:///etc/passwd",
            "ftp://example.com/x",
        ] {
            assert!(!is_public_url(url), "{url} should be rejected");
        }
    }
}
