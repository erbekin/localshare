//! This module serves common utility functions and types

use std::{net::IpAddr, sync::LazyLock};

static LOCAL_IP : LazyLock<IpAddr> = LazyLock::new(|| {
    let ip = local_ip_address::local_ip().expect("Could not get local ip address");
    log::info!("local ip: {}", ip);
    ip
});
pub fn get_local_ip() -> IpAddr {
    *LOCAL_IP
}
