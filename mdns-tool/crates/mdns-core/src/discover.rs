use crate::model::DiscoveredService;
use anyhow::Result;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::time::Duration;

pub fn browse_blocking(service_type: &str, timeout: Duration, _interface: &str) -> Result<Vec<DiscoveredService>> {
    let daemon = ServiceDaemon::new()?;
    let service_type = if service_type.ends_with('.') {
        service_type.to_string()
    } else {
        format!("{}.", service_type)
    };

    let receiver = daemon.browse(&service_type)?;
    let mut out = Vec::new();

    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if let Ok(event) = receiver.recv_timeout(Duration::from_millis(2000)) {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let txt = info
                        .get_properties()
                        .iter()
                        .map(|prop| (prop.key().to_string(), prop.val_str().to_string()))
                        .collect::<Vec<_>>();

                    out.push(DiscoveredService {
                        fullname: info.get_fullname().to_string(),
                        instance_name: info.get_hostname().to_string(),
                        service_type: service_type.clone(),
                        host_name: info.get_hostname().to_string(),
                        port: info.get_port(),
                        addresses: info.get_addresses().iter().copied().collect(),
                        txt,
                    });
                }
                ServiceEvent::ServiceFound(service_name, full_name) => {
                    println!("Found service with name of {} and type of {}", full_name, service_name);
                }
                _ => {}
            }
        }
    }
    Ok(out)
}