#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core_ink_5::ble::BlePins;
use core_ink_5::ble::tasks::spawn_ble_tasks;
use core_ink_5::buttons::{ButtonPins, spawn_buttons_task};
use core_ink_5::display::{DisplayPins, spawn_display_task};
use core_ink_5::wifi::WifiPins;
use core_ink_5::wifi::wifi::spawn_wifi_tasks;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, gpio::Pin};
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

    {
        let mut pc = Output::new(peripherals.GPIO12, Level::High, OutputConfig::default());
        pc.set_high();
    }

    spawn_display_task(
        &spawner,
        DisplayPins {
            busy: peripherals.GPIO4.degrade(),
            rst: peripherals.GPIO0.degrade(),
            dc: peripherals.GPIO15.degrade(),
            cs: peripherals.GPIO9.degrade(),
            sck: peripherals.GPIO18.degrade(),
            mosi: peripherals.GPIO23.degrade(),
            miso: peripherals.GPIO34.degrade(),
            spi: peripherals.SPI2,
        },
    );

    spawn_buttons_task(
        &spawner,
        ButtonPins {
            up: peripherals.GPIO37.degrade(),
            down: peripherals.GPIO39.degrade(),
            middle: peripherals.GPIO38.degrade(),
            user: peripherals.GPIO5.degrade(),
        },
    );

    spawn_wifi_tasks(
        &spawner,
        WifiPins {
            wifi: peripherals.WIFI,
        },
    );

    spawn_ble_tasks(&spawner, BlePins { bt: peripherals.BT });

    let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
    // let mut buzzer = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    loop {
        Timer::after(Duration::from_secs(3)).await;
        led.set_low();
        // buzzer.set_low();
        Timer::after(Duration::from_millis(500)).await;
        led.set_high();
        // buzzer.set_high();
    }
}
