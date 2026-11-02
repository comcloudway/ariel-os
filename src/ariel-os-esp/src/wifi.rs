#[cfg(not(feature = "wifi-dpp"))]
use ariel_os_utils::str_from_env;

#[cfg(feature = "wifi-esp")]
pub mod esp_wifi;

// TODO: this should be factored out in ariel-os-embassy again
#[cfg(not(feature = "wifi-dpp"))]
pub(crate) const WIFI_NETWORK: &str =
    str_from_env!("CONFIG_WIFI_NETWORK", "Wi-Fi SSID (network name)");
#[cfg(not(feature = "wifi-dpp"))]
pub(crate) const WIFI_PASSWORD: &str = str_from_env!("CONFIG_WIFI_PASSWORD", "Wi-Fi password");
