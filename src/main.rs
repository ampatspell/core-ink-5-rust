#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    prelude::*,
    primitives::{Line, PrimitiveStyleBuilder, Rectangle},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::prelude::{RefreshLut, WaveshareDisplay};
use epd_waveshare::{color::Color, epd1in54_v2::Display1in54, epd1in54_v2::Epd1in54};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    spi::{Mode, master::Config},
    time::Rate,
};
use esp_hal::{spi::master::Spi, timer::timg::TimerGroup};
use {esp_backtrace as _, esp_println as _};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let software_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

    let busy = Input::new(peripherals.GPIO4, InputConfig::default());
    let rst = Output::new(peripherals.GPIO0, Level::Low, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
    let cs = Output::new(peripherals.GPIO9, Level::Low, OutputConfig::default());
    let sck = peripherals.GPIO18;
    let mosi = peripherals.GPIO23;
    let miso = peripherals.GPIO34;

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(4))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sck)
    .with_mosi(mosi)
    .with_miso(miso);

    let mut spi_device = ExclusiveDevice::new_no_delay(spi, cs);
    let mut delay = Delay::new();
    let mut epd = Epd1in54::new(&mut spi_device, busy, dc, rst, &mut delay, Some(5)).unwrap();
    let mut display = Display1in54::default();

    let _ = spawner;

    epd.set_lut(&mut spi_device, &mut delay, Some(RefreshLut::Full))
        .unwrap();
    epd.set_background_color(Color::Black);
    epd.clear_frame(&mut spi_device, &mut delay).unwrap();
    epd.display_frame(&mut spi_device, &mut delay).unwrap();

    loop {
        info!("Loop");

        {
            let style = PrimitiveStyleBuilder::new()
                .fill_color(Color::White)
                .build();

            Rectangle::new(Point::new(0, 0), Size::new(200, 200))
                .into_styled(style)
                .draw(&mut display)
                .unwrap();
        }
        {
            let style = PrimitiveStyleBuilder::new()
                .stroke_color(Color::Black)
                .stroke_width(5)
                .build();

            Line::new(Point::new(0, 0), Point::new(200, 200))
                .into_styled(style)
                .draw(&mut display)
                .unwrap();

            Line::new(Point::new(200, 0), Point::new(0, 200))
                .into_styled(style)
                .draw(&mut display)
                .unwrap();
        }

        epd.update_frame(&mut spi_device, &display.buffer(), &mut delay)
            .unwrap();
        epd.display_frame(&mut spi_device, &mut delay).unwrap();
        epd.sleep(&mut spi_device, &mut delay).unwrap();

        Timer::after(Duration::from_secs(1)).await;
    }
}
