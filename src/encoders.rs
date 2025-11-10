use rppal::{gpio::{Event, Gpio, InputPin}, i2c::I2c};
use std::sync::{Arc, atomic::AtomicI64};

pub const ENC_A_DT: u8 = 5;
pub const ENC_A_CLK: u8 = 6;
pub const ENC_A_PB: u8 = 16;
pub const ENC_B_DT: u8 = 12;
pub const ENC_B_CLK: u8 = 13;
pub const ENC_B_PB: u8 = 26;

pub fn encoder_pos(event: Event, dt_pin: &InputPin, counter: &AtomicI64) {
    if dt_pin.is_high() {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    } else {
        counter.fetch_add(-1, std::sync::atomic::Ordering::SeqCst);
    }
}