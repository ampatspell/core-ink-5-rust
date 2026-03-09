pub mod tasks;

pub struct BlePins {
    pub bt: esp_hal::peripherals::BT<'static>,
}
