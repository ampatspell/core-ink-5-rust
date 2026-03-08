use defmt::{Format, info};
use embassy_futures::select::select4;
use esp_hal::gpio::{AnyPin, Input, InputConfig};

use crate::{
    buttons::{Button, ButtonPins},
    channels::{RANDOM, Random},
};

#[embassy_executor::task]
pub async fn buttons_task(pins: ButtonPins) {
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
