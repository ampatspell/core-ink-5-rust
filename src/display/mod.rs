use crate::display::task::display_task;
use embassy_executor::Spawner;
use esp_hal::{gpio::AnyPin, peripherals::SPI2};

pub mod display;
pub mod task;

pub fn spawn_display_task(spawner: &Spawner, pins: DisplayPins) {
    spawner.spawn(display_task(pins)).unwrap();
}

pub struct DisplayPins {
    pub busy: AnyPin<'static>,
    pub rst: AnyPin<'static>,
    pub dc: AnyPin<'static>,
    pub cs: AnyPin<'static>,
    pub sck: AnyPin<'static>,
    pub mosi: AnyPin<'static>,
    pub miso: AnyPin<'static>,
    pub spi: SPI2<'static>,
}
