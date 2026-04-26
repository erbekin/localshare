//! This module handles QR Code generation for Host IP
//!

use std::path::PathBuf;



use crate::{config::{self, Config}, utils};

/// Tries to generate qr using configurated values
/// Returns true if generation was successful
/// Failure or success will be logged
pub fn generate_qr(workdir : &PathBuf, conf : &Config) -> bool {
    let ip = utils::get_local_ip();
    // Format with port
    let addr_string = format!("http://{}:{}", ip, conf.app.port);
    let qr_path = workdir.join(&conf.path.r#static).join(config::QR_ACCESS_FNAME);

    let code = match qrcode::QrCode::new(addr_string.as_bytes()) {
        Ok(code) => code,
        Err(e) => {
            log::error!("QR code generation failed: {}", e);
            return false;
        }
    };

    let img = code.render::<image::Luma<u8>>().build();
    if let Err(e) = img.save(&qr_path) {
        log::error!("QR code generation failed: {}", e);
        return false;
    }
    true
}
