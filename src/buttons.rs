use defmt::{Format, info};
use embassy_executor::Spawner;
use embassy_futures::select::select4;
use esp_hal::gpio::{AnyPin, Input, InputConfig};

use crate::channels::{RANDOM, Random};

pub struct ButtonPins {
    pub up: AnyPin<'static>,
    pub middle: AnyPin<'static>,
    pub down: AnyPin<'static>,
    pub user: AnyPin<'static>,
}

#[derive(Format, Clone, Debug)]
pub enum Button {
    Up,
    Down,
    Middle,
    User,
}

pub fn spawn_buttons_task(spawner: &Spawner, pins: ButtonPins) {
    spawner.spawn(buttons_task(pins)).unwrap();
}

#[embassy_executor::task]
async fn buttons_task(pins: ButtonPins) {
    info!("buttons_task");

    let mut up = ButtonPinHandler::new(Button::Up, pins.up);
    let mut down = ButtonPinHandler::new(Button::Down, pins.down);
    let mut middle = ButtonPinHandler::new(Button::Middle, pins.middle);
    let mut user = ButtonPinHandler::new(Button::User, pins.user);

    loop {
        select4(up.wait(), down.wait(), middle.wait(), user.wait()).await;
    }
}

#[derive(Format)]
struct ButtonPinHandler<'a> {
    pub button: Button,
    pub on: bool,
    pub input: Input<'a>,
}

impl<'a> ButtonPinHandler<'a> {
    pub fn new(button: Button, pin: AnyPin<'a>) -> Self {
        let input = Input::new(pin, InputConfig::default());
        let on = input.is_low();
        Self { button, input, on }
    }

    fn on_edge(&mut self) -> Option<bool> {
        let on = self.input.is_low();
        if self.on != on {
            self.on = on;
            return Some(on);
        }
        None
    }

    pub async fn wait(&mut self) {
        self.input.wait_for_any_edge().await;
        match self.on_edge() {
            Some(state) => {
                RANDOM
                    .send(Random::Button {
                        button: self.button.clone(),
                        on: state,
                    })
                    .await;
            }
            None => {}
        }
    }
}
