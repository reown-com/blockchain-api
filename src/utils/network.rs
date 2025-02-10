use {axum::http::HeaderMap, ipnet::IpNet, std::net::IpAddr};

#[derive(thiserror::Error, Debug)]
pub enum NetworkInterfaceError {
    #[error("machine has no public IP address")]
    PublicAddressNotFound,
    #[error("machine has multiple public IP addresses")]
    MultiplePublicAddresses,
}

/// Attempts to find the public IP address of this machine.
pub fn find_public_ip_addr() -> Result<IpAddr, NetworkInterfaceError> {
    let addrs: Vec<_> = pnet_datalink::interfaces()
        .into_iter()
        .flat_map(|iface| {
            iface
                .ips
                .into_iter()
                .filter(|ip| ip.is_ipv4() && is_public_ip_addr(ip.ip()))
                .map(|ip| ip.ip())
        })
        .collect();

    if addrs.is_empty() {
        Err(NetworkInterfaceError::PublicAddressNotFound)
    } else if addrs.len() > 1 {
        Err(NetworkInterfaceError::MultiplePublicAddresses)
    } else {
        Ok(addrs[0])
    }
}

fn is_public_ip_addr(addr: IpAddr) -> bool {
    use once_cell::sync::Lazy;

    static RESERVED_NETWORKS: Lazy<[IpNet; 24]> = Lazy::new(|| {
        [
            "0.0.0.0/8",
            "0.0.0.0/32",
            "100.64.0.0/10",
            "127.0.0.0/8",
            "169.254.0.0/16",
            "172.16.0.0/12",
            "192.0.0.0/24",
            "192.0.0.0/29",
            "192.0.0.8/32",
            "192.0.0.9/32",
            "192.0.0.10/32",
            "192.0.0.170/32",
            "192.0.0.171/32",
            "192.0.2.0/24",
            "192.31.196.0/24",
            "192.52.193.0/24",
            "192.88.99.0/24",
            "192.168.0.0/16",
            "192.175.48.0/24",
            "198.18.0.0/15",
            "198.51.100.0/24",
            "203.0.113.0/24",
            "240.0.0.0/4",
            "255.255.255.255/32",
        ]
        .map(|net| net.parse().unwrap())
    });

    RESERVED_NETWORKS.iter().all(|range| !range.contains(&addr))
}

pub fn get_forwarded_ip(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get("X-Forwarded-For")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.split(',').last())
        .and_then(|client_ip| client_ip.trim().parse::<IpAddr>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_forwarded_ip() {
        // Singe IP
        let mut headers_single = HeaderMap::new();
        headers_single.insert("X-Forwarded-For", "10.128.128.1".parse().unwrap());
        assert_eq!(
            get_forwarded_ip(&headers_single).unwrap(),
            "10.128.128.1".parse::<IpAddr>().unwrap()
        );

        // Muplitple IPs appended by ALB
        let mut headers_multiple = HeaderMap::new();
        headers_multiple.insert(
            "X-Forwarded-For",
            "10.128.128.1, 10.128.128.2".parse().unwrap(),
        );
        assert_eq!(
            get_forwarded_ip(&headers_multiple).unwrap(),
            "10.128.128.2".parse::<IpAddr>().unwrap()
        );
    }
}
