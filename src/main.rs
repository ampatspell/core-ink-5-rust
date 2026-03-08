#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::future;
use core_ink_5::{
    buttons::{ButtonPins, spawn_buttons_task},
    display::{DisplayPins, spawn_display_task},
};
use embassy_executor::Spawner;
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

    future::pending().await
}
