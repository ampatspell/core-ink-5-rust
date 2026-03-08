use defmt::Format;
use embassy_executor::Spawner;
use esp_hal::gpio::AnyPin;

use crate::buttons::task::buttons_task;

pub mod task;

pub fn spawn_buttons_task(spawner: &Spawner, pins: ButtonPins) {
    spawner.spawn(buttons_task(pins)).unwrap();
}

#[derive(Format, Clone, Debug)]
pub enum Button {
    Up,
    Down,
    Middle,
    User,
}

pub struct ButtonPins {
    pub up: AnyPin<'static>,
    pub middle: AnyPin<'static>,
    pub down: AnyPin<'static>,
    pub user: AnyPin<'static>,
}
