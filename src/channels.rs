use crate::buttons::Button;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use no_std_strings::{str16, str32};

pub enum Random {
    Button { button: Button, on: bool },
    IP { value: Option<str32> },
    BLE { total: usize },
    Time { current: str16 },
}

pub static RANDOM: Channel<CriticalSectionRawMutex, Random, 5> = Channel::new();
