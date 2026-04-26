//! This module publishs mDNS service

use anyhow::Context;
use mdns_sd::{DaemonStatus, ServiceDaemon, ServiceInfo};

use crate::{config::Config, utils};

pub struct MDnsService {
    daemon: ServiceDaemon,
    shutdown : bool,
}
impl MDnsService {
    pub fn shutdown(&mut self) {
        if self.shutdown {
            return;
        }
        match self.daemon.shutdown() {
            Ok(r) => {
                if let Ok(status) =  r.recv() {
                    if status == DaemonStatus::Shutdown {
                        log::info!("mDNS daemon has been succesfully shut down");
                    } else {
                        log::warn!("mDNS daemon was shut down, but shutting process might have gone wrong");
                    }
                }
                self.shutdown = true;
            }
            Err(mdns_sd::Error::Again) => {
                log::warn!("Could not shut down mDNS daemon, try again");
            }
            Err(e) => {
                log::error!("Could not shut down mDNS daemon: {}", e);
            }
        }
    }
}
pub fn start_service(conf: &Config) -> anyhow::Result<MDnsService> {
    let mdns = ServiceDaemon::new()?;
    let service_type = "_localshare._tcp.local.";
    let instance_name = "localshare"; // This creates "localshare.local"
    let host_name = format!("{}.local.", instance_name);
    let service_info = ServiceInfo::new(
        service_type,
        instance_name,
        &host_name,
        utils::get_local_ip(),
        conf.app
            .port
            .parse()
            .context("Could not parse port string to u16")?,
        None,
    ).context("Could not initialize service info")?;
    mdns.register(service_info).context("Could not register mdns service")?;
    Ok(MDnsService { daemon: mdns, shutdown: false })
}
