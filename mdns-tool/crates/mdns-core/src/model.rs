use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxtRecord(pub Vec<(String, String)>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAnnouncement {
    /// e.g. "_myapp._tcp.local."
    pub service_type: String,
    /// e.g. "My Host Instance"
    pub instance_name: String,
    /// e.g. "myhost.local."
    pub host_name: String,
    pub ip_addr: String,
    pub port: u16,
    pub txt: Option<TxtRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    pub fullname: String,
    pub instance_name: String,
    pub service_type: String,
    pub host_name: String,
    pub port: u16,
    pub addresses: Vec<IpAddr>,
    pub txt: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceIp {
    pub name: String,
    pub ip: IpAddr,
    pub family: &'static str,  //ipv4 or ipv6
    pub is_loopback: bool,
}
