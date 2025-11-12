use std::error::Error;
use std::sync::{Arc, atomic::AtomicI64};
use std::thread::sleep;
use std::time::Duration;

use mcp23017::MCP23017;
use rppal::{gpio::{Event, Gpio, InputPin}, i2c::I2c};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Keypad {
    ZERO,
    ONE,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
    SEVEN,
    EIGHT,
    NINE,
    A,
    B,
    C,
    D,
    POUND,
    STAR,
    VOL,
    KEY,
}

pub fn init_keypad(i2c: I2c) -> Option<MCP23017<I2c>> {
    let mut ex_gpio = MCP23017::new(i2c, 0x27).expect("failed to initialize GPIO expander");
    ex_gpio.init_hardware();
    
    let _ = ex_gpio.pin_mode(8, mcp23017::PinMode::OUTPUT).expect("failed to init ouput gpio");
    let _ = ex_gpio.pin_mode(9, mcp23017::PinMode::OUTPUT).expect("failed to init ouput gpio");
    let _ = ex_gpio.pin_mode(10, mcp23017::PinMode::OUTPUT).expect("failed to init ouput gpio");
    let _ = ex_gpio.pin_mode(11, mcp23017::PinMode::OUTPUT).expect("failed to init ouput gpio");
    ex_gpio.digital_write(8, true);
    ex_gpio.digital_write(9, true);
    ex_gpio.digital_write(10, true);
    ex_gpio.digital_write(11, true);
    // ex_gpio.digital_write(8, false);
    // ex_gpio.digital_write(9, false);
    // ex_gpio.digital_write(10, false);
    // ex_gpio.digital_write(11, false);
    
    let _ = ex_gpio.pin_mode(12, mcp23017::PinMode::INPUT).expect("failed to init input gpio");
    let _ = ex_gpio.pin_mode(13, mcp23017::PinMode::INPUT).expect("failed to init input gpio");
    let _ = ex_gpio.pin_mode(14, mcp23017::PinMode::INPUT).expect("failed to init input gpio");
    let _ = ex_gpio.pin_mode(15, mcp23017::PinMode::INPUT).expect("failed to init input gpio");
    
    ex_gpio.pull_up(12, true);
    ex_gpio.pull_up(13, true);
    ex_gpio.pull_up(14, true);
    ex_gpio.pull_up(15, true);

    ex_gpio.invert_input_polarity(12, true);
    ex_gpio.invert_input_polarity(13, true);
    ex_gpio.invert_input_polarity(14, true);
    ex_gpio.invert_input_polarity(15, true);
    Some(ex_gpio)
}

pub fn get_keypad(ex_gpio: &mut MCP23017<I2c>, last_input: Option<Keypad>) -> Option<Keypad>{
    let mut out = None;
    for row in 0..4 {
        let _ = ex_gpio.digital_write(row + 8, false);
        sleep(Duration::from_millis(1));
        for col in 4..8 {
            if ex_gpio.digital_read(col + 8).unwrap_or(false) {
                //println!("row {}, col {}", row, col);
                out = get_keycode(row, col);
                if out == last_input {
                    return out;
                }
            }
        }
        let _ = ex_gpio.digital_write(row + 8, true);
    }
    out
}

pub fn get_keycode(row: u8, column: u8) -> Option<Keypad> {
    let mut out = None;
    match row {
        0 => {
            match column {
                4 => {
                    out = Some(Keypad::ONE);
                },
                5 => {
                    out = Some(Keypad::TWO);
                },
                6 => {
                    out = Some(Keypad::THREE);
                },
                7 => {
                    out = Some(Keypad::A);
                },
                _ => {}
            }
        }
        1 => {
            match column {
                4 => {
                    out = Some(Keypad::FOUR);
                },
                5 => {
                    out = Some(Keypad::FIVE);
                },
                6 => {
                    out = Some(Keypad::SIX);
                },
                7 => {
                    out = Some(Keypad::B);
                },
                _ => {}
            }
        }
        2 => {
            match column {
                4 => {
                    out = Some(Keypad::SEVEN);
                },
                5 => {
                    out = Some(Keypad::EIGHT);
                },
                6 => {
                    out = Some(Keypad::NINE);
                },
                7 => {
                    out = Some(Keypad::C);
                },
                _ => {}
            }
        }
        3 => {
             match column {
                 4 => {
                     out = Some(Keypad::STAR);
                 },
                 5 => {
                     out = Some(Keypad::ZERO);
                 },
                 6 => {
                     out = Some(Keypad::POUND);
                 },
                 7 => {
                     out = Some(Keypad::D);
                 },
                 _ => {}
             }
         }
        _ => {}
    }
    out
}