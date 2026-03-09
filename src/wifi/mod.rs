use esp_hal::peripherals::WIFI;

pub mod http;
pub mod wifi;

pub struct WifiPins {
    pub wifi: WIFI<'static>,
}
