use ariel_os_log::{debug, info};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_radio::wifi::{Config, ControllerConfig, Interface, WifiController, sta::StationConfig};

#[cfg(feature = "wifi-dpp")]
use ariel_os_random as random;
#[cfg(feature = "wifi-dpp")]
use esp_radio::wifi::sniffer::Sniffer;
#[cfg(feature = "wifi-dpp")]
use wifi_dpp::platform::esp::EspDppRunner;
#[cfg(feature = "wifi-dpp")]
use wifi_dpp::{self, BootstrappingInformation, DppManager};

#[cfg(feature = "wifi-dpp")]
use alloc::boxed::Box;

use ariel_os_log::error;

pub type NetworkDevice = Interface<'static>;

pub fn init(
    peripherals: &mut crate::OptionalPeripherals,
    spawner: Spawner,
    credentials: Option<(Option<String>, Option<String>)>,
    update_credentials: Option<fn(String, String)>,
) -> NetworkDevice {
    let config = ControllerConfig::default();
    let wifi = peripherals.WIFI.take().unwrap();

    // (swallowing the `WifiError` here, it pulls in quite a bit of fmt code.)
    info!("Starting wifi interface");
    let (controller, interfaces) = esp_radio::wifi::new(wifi, config).ok().unwrap();
    info!("Initialized interface");

    #[cfg(feature = "wifi-dpp")]
    {
        let (wifi_ssid, wifi_pass) = if let Some(credentials) = credentials {
            (credentials.0, credentials.1)
        } else {
            (None, None)
        };

        if wifi_ssid.is_none() {
            let mut rng = random::crypto_rng();

            #[cfg(feature = "wifi-dpp-pem")]
            let private_key =
                p256::SecretKey::from_sec1_pem(include_str!(env!("WIFI_DPP_PEM_FILE")))
                    .expect("Loaded PEM file should be valid");

            #[cfg(not(feature = "wifi-dpp-pem"))]
            let private_key = wifi_dpp::generate_private_key(&mut rng);

            let bootstrapping_information =
                BootstrappingInformation::from_private_key(private_key).unwrap();

            info!(
                "DPP provisioning payload: '{}'",
                bootstrapping_information
                    .get_provisioning_uri()
                    .unwrap()
                    .as_str()
            );
            spawner.spawn(
                dpp_connection(
                    spawner,
                    controller,
                    bootstrapping_information,
                    interfaces.sniffer,
                    update_credentials,
                )
                .unwrap(),
            );
        } else {
            info!("Skipping DPP protocol, as we already have credentials");
            spawner.spawn(
                connection(
                    controller,
                    wifi_ssid.unwrap(),
                    wifi_pass.unwrap_or_default(),
                )
                .unwrap(),
            );
        }
    }

    #[cfg(not(feature = "wifi-dpp"))]
    {
        info!("Connection to wifi network with credentials provided via env vars");
        let (wifi_ssid, wifi_pass) = (
            crate::wifi::WIFI_NETWORK.try_into().unwrap(),
            crate::wifi::WIFI_PASSWORD.try_into().unwrap(),
        );
        spawner.spawn(connection(controller, wifi_ssid, wifi_pass).unwrap());
    }

    interfaces.station
}
use alloc::string::String;

#[cfg(feature = "wifi-dpp")]
#[embassy_executor::task]
async fn dpp_connection(
    spawner: Spawner,
    mut controller: WifiController<'static>,
    bootstrapping_information: BootstrappingInformation,
    sniffer: Sniffer<'static>,
    update_credentials: Option<fn(String, String)>,
) {
    info!("wifi-dpp enabled and no credentials present, engaging for one minute to obtain data");
    let esp_runner = EspDppRunner::try_init(sniffer).unwrap();
    let mut dpp = DppManager::new(esp_runner, bootstrapping_information);

    let configuration_request = wifi_dpp::proto::DppConfigurationRequest {
        name: String::from("ArielOS Enrollee"),
        ..Default::default()
    };

    let mut rng = random::crypto_rng();

    info!("DPP start");
    match dpp
        .enroll_async(
            &configuration_request,
            core::time::Duration::from_mins(2),
            &mut rng,
        )
        .await
    {
        Ok(c) => {
            info!("[DPP]: Received Wifi credentials");
            let wifi_ssid = c.discovery.ssid;
            let wifi_pass = c.credentials.pass.unwrap_or(String::new());
            if let Some(update_credentials) = update_credentials {
                info!("Storing credentials received via DPP");
                update_credentials(wifi_ssid.clone(), wifi_pass.clone());
            }
            // handover to connection with credentials
            spawner.spawn(connection(controller, wifi_ssid, wifi_pass).unwrap());
        }
        Err(_) => {
            error!("Wifi DPP failed");
        }
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>, wifi_ssid: String, wifi_pass: String) {
    debug!("start connection task");

    debug!("Configuring Wi-Fi");
    let client_config = Config::Station(
        StationConfig::default()
            .with_ssid(wifi_ssid)
            .with_password(wifi_pass),
    );

    // (swallowing the `WifiError` here, it pulls in quite a bit of fmt code.)
    controller.set_config(&client_config).ok().unwrap();

    debug!("Wi-Fi configured!");

    loop {
        debug!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => {
                info!("Wifi connected!");
                // Wait until we're no longer connected, then loop to reconnect.
                let _ = controller.wait_for_disconnect_async().await;
            }
            Err(e) => {
                info!("Failed to connect to Wi-Fi: {:?}", e);
            }
        }

        Timer::after(Duration::from_secs(5)).await;
    }
}
