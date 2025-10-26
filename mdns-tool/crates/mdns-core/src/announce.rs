use crate::model::{ServiceAnnouncement, TxtRecord};
use anyhow::Result;
use mdns_sd::{ServiceDaemon, ServiceInfo};

/// Handle so the service stays registered while this is alive.
pub struct Announcer {
    _daemon: ServiceDaemon,  // Keep daemon alive
    fullname: String,
}

impl Announcer {
    pub fn register(ann: ServiceAnnouncement) -> Result<Self> {
        let daemon = ServiceDaemon::new()?;

        let txt_kv = ann
            .txt
            .unwrap_or(TxtRecord(vec![]))
            .0
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect::<Vec<_>>();

        // Ensure trailing dots as mdns-sd expects FQDNs.
        let service_type = ensure_dot(&ann.service_type);
        let host_name = ensure_dot(&ann.host_name);

        let info = ServiceInfo::new(
            &service_type,
            &ann.instance_name,
            &host_name,
            &ann.ip_addr,
            ann.port,
            &*txt_kv,
        )?;

        daemon.register(info.clone())?;
        Ok(Self {
            _daemon: daemon,
            fullname: info.get_fullname().to_string(),
        })
    }

    pub fn fullname(&self) -> &str {
        &self.fullname
    }
}

fn ensure_dot(s: &str) -> String {
    if s.ends_with('.') {
        s.to_string()
    } else {
        format!("{}.", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ensure_dot() {
        assert_eq!(ensure_dot("test_case").contains("."), true );
    }
}