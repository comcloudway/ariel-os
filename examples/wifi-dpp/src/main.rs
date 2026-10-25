#![no_main]
#![no_std]

use ariel_os::{
    debug::{ExitCode, exit},
    log::*,
    time::Timer,
};

#[ariel_os::task(autostart)]
async fn main() {
    info!(
        "ArielOS Wifi-DPP example.\nGenerate a QR code from the provisioning payload and scan it with a phone"
    );

    loop {
        Timer::after_secs(5).await;
        info!("Main still alive");
        if let Some(stack) = ariel_os::net::network_stack().await {
            let link_is_up = stack.is_link_up();
            info!("Link is up: {:?}", link_is_up);
            if link_is_up {
                break;
            }
        }
    }

    exit(ExitCode::SUCCESS);
}
