use defmt::info;
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyle, iso_8859_5::FONT_10X20},
    prelude::*,
    primitives::{PrimitiveStyle, StyledDrawable},
    text::Text,
};
use epd_waveshare::{color::Color, prelude::RefreshLut};
use no_std_strings::{str_format, str16};

use crate::{
    channels::{
        RANDOM,
        Random::{self},
    },
    display::{DisplayPins, display::Display},
};

#[embassy_executor::task]
pub async fn display_task(pins: DisplayPins) {
    info!("start display_task");

    let mut display = Display::new(pins);

    let clear = |display: &mut Display| {
        let style = PrimitiveStyle::with_fill(Color::White);
        display.bounding_box().draw_styled(&style, display);
    };

    display.set_lut(RefreshLut::Full);
    clear(&mut display);

    let style = MonoTextStyle::new(&FONT_10X20, Color::Black);
    let mut label = str16::from("CoreInk M5");
    let mut ip = str16::from("No IP");

    display.update_and_display();
    display.set_lut(RefreshLut::Quick);

    loop {
        clear(&mut display);

        Text::new(label.to_str(), Point::new(20, 30), style)
            .draw(&mut display)
            .unwrap();
        Text::new(ip.to_str(), Point::new(20, 50), style)
            .draw(&mut display)
            .unwrap();

        display.update_and_display();

        let message = RANDOM.receive().await;
        match message {
            Random::Button { button, on } => {
                label = str_format!(str16, "{:?} {}", button, on);
            }
            Random::IP { value } => match value {
                Some(value) => ip = value,
                None => ip = str16::from("No IP"),
            },
        }
    }
}
