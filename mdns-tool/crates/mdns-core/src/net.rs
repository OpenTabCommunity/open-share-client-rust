use std::io::Error;
use crate::model::InterfaceIp;

pub fn list_interface_ips_result() -> Result<Vec<InterfaceIp>, Error> {
    let ifs = if_addrs::get_if_addrs()?;

    let mut out: Vec<InterfaceIp> = ifs
        .into_iter()
        .filter_map(|ifa| {
            let ip = ifa.ip();
            // ip() returns std::net::IpAddr
            let family = if ip.is_ipv4() {"ipv4"} else {"ipv6"};
            let is_loopback = ip.is_loopback();

            Some(InterfaceIp {
                name: ifa.name,
                ip,
                family,
                is_loopback,
            })
        }).collect();

    out.sort_by(|a, b| (&a.name, &a.ip).cmp(&(&b.name, &b.ip)));
    out.dedup_by(|a, b| a.name == b.name && a.ip == b.ip);
    Ok(out)
}