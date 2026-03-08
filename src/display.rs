use defmt::info;
use embassy_executor::Spawner;
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyle, iso_8859_1::FONT_9X18_BOLD},
    prelude::*,
    primitives::{PrimitiveStyle, StyledDrawable},
    text::Text,
};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use epd_waveshare::{
    color::Color,
    epd1in54_v2::{Display1in54, Epd1in54},
    prelude::{RefreshLut, WaveshareDisplay},
};
use esp_hal::{
    Blocking,
    delay::Delay,
    gpio::{AnyPin, Input, InputConfig, Level, Output, OutputConfig},
    peripherals::SPI2,
    spi::{
        Mode,
        master::{Config, Spi},
    },
    time::Rate,
};
use no_std_strings::{str_format, str16};

use crate::channels::{RANDOM, Random::Button};

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

pub fn spawn_display_task(spawner: &Spawner, pins: DisplayPins) {
    spawner.spawn(display_task(pins)).unwrap();
}

pub struct Display {
    driver: Epd1in54<
        ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, NoDelay>,
        Input<'static>,
        Output<'static>,
        Output<'static>,
        Delay,
    >,
    display: epd_waveshare::graphics::Display<200, 200, false, 5000, Color>,
    spi: ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, NoDelay>,
    delay: Delay,
}

impl Display {
    pub fn new(pins: DisplayPins) -> Self {
        let busy = Input::new(pins.busy, InputConfig::default());
        let rst = Output::new(pins.rst, Level::Low, OutputConfig::default());
        let dc = Output::new(pins.dc, Level::Low, OutputConfig::default());
        let cs = Output::new(pins.cs, Level::Low, OutputConfig::default());
        let sck = pins.sck;
        let mosi = pins.mosi;
        let miso = pins.miso;

        let bus = Spi::new(
            pins.spi,
            Config::default()
                .with_frequency(Rate::from_mhz(4))
                .with_mode(Mode::_0),
        )
        .unwrap()
        .with_sck(sck)
        .with_mosi(mosi)
        .with_miso(miso);

        let mut spi = ExclusiveDevice::new_no_delay(bus, cs);
        let mut delay = Delay::new();

        let driver = Epd1in54::new(&mut spi, busy, dc, rst, &mut delay, None).unwrap();
        let display = Display1in54::default();

        Self {
            driver,
            display,
            spi,
            delay,
        }
    }

    pub fn set_lut(&mut self, refresh_rate: RefreshLut) {
        self.driver
            .set_lut(&mut self.spi, &mut self.delay, Some(refresh_rate))
            .unwrap();
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.driver.set_background_color(color);
    }

    pub fn clear_frame(&mut self) {
        self.driver
            .clear_frame(&mut self.spi, &mut self.delay)
            .unwrap();
    }

    pub fn update_frame(&mut self) {
        self.driver
            .update_frame(&mut self.spi, &self.display.buffer(), &mut self.delay)
            .unwrap();
    }

    pub fn display_frame(&mut self) {
        self.driver
            .display_frame(&mut self.spi, &mut self.delay)
            .unwrap();
    }
}

impl DrawTarget for Display {
    type Color = Color;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        self.display.draw_iter(pixels)
    }
}

impl Dimensions for Display {
    fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
        self.display.bounding_box()
    }
}

#[embassy_executor::task]
async fn display_task(pins: DisplayPins) {
    info!("display_task");

    let mut display = Display::new(pins);

    let clear = |display: &mut Display| {
        let style = PrimitiveStyle::with_fill(Color::White);
        display.bounding_box().draw_styled(&style, display);
    };

    display.set_lut(RefreshLut::Full);
    clear(&mut display);
    display.update_frame();
    display.display_frame();
    display.set_lut(RefreshLut::Quick);

    let style = MonoTextStyle::new(&FONT_9X18_BOLD, Color::Black);
    let mut label = str16::from("CoreInk M5");

    loop {
        clear(&mut display);

        Text::new(label.to_str(), Point::new(20, 30), style)
            .draw(&mut display)
            .unwrap();

        display.update_frame();
        display.display_frame();

        let message = RANDOM.receive().await;
        match message {
            Button { button, on } => {
                label = str_format!(str16, "{:?} {}", button, on);
                info!("{}", label.to_str());
            }
        }
    }
}
