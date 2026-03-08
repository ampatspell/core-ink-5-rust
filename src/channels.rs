use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

use crate::buttons::Button;

pub enum Random {
    Button { button: Button, on: bool },
}

pub static RANDOM: Channel<CriticalSectionRawMutex, Random, 1> = Channel::new();
