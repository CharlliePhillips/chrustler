use awedio::{backends::CpalBufferSize, manager::Manager, sounds::{MemorySound, wrappers::{AdjustableSpeed, Controllable, Controller, Pausable, Stoppable}}, *};
use pitch_detection::{detector::{mcleod::McLeodDetector, PitchDetector}, *};
use rppal::{gpio::{Event, Gpio, Trigger}, i2c::I2c};
use std::{env, sync::{Arc, Mutex, atomic::{AtomicBool, AtomicI64, AtomicU16}}, thread::{current, sleep}, time::Duration};
use std::fs::File;
use std::io;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};
use vl53l1x::{Vl53l1x, Vl53l1xRangeStatus};
use mcp23017::MCP23017;
use num_traits::pow::Pow;

// use crate::keypad::*;
// use crate::encoders::*;
// use crate::tof::*;
mod keypad;
mod encoders;
mod tof;




const SAMPLE_RATE: usize = 48000;
const SIZE: usize = 1024;
const PADDING: usize = SIZE / 2;
const POWER_THRESHOLD: f64 = 0.0001;
const CLARITY_THRESHOLD: f64 = 0.25;

const TEMP_PIN: u8 = 27;
const TOF_INT_PIN: u8 = 17;

const DEFAULT_EQ_LEVEL: u16 = 12;


const TRIADS: u16 = 3;
const SEVENTHS: u16 = 4;
const NINTHS: u16 = 5;

const MAJ_MUL: [f64; 15] = [1.0, 1.1225, 1.2599, 1.3348, 1.4983, 1.6818, 1.887, 2.0, 2.2449, 2.5198, 2.6697, 2.9966, 3.3636, 3.7755, 4.0];
const MIN_MUL: [f64; 15] = [1.0, 1.1225, 1.1892, 1.3348, 1.4983, 1.5874, 1.7818, 2.0, 2.2449, 2.3784, 2.6697, 2.9966, 3.1748, 3.5636, 4.0];

#[derive(Clone, Copy)]
enum Chords {
    I,
    i,
    ii,
    iid,
    III,
    iii,
    IV,
    iv,
    V,
    v,
    vi,
    VI,
    vii,
    VII
}

impl Chords {
    fn note_indices(self) -> Vec<u16> {
        match self {
            Chords::I => vec![1, 3, 5, 7, 9],
            Chords::i => vec![1, 3, 5, 7, 9],
            Chords::ii => vec![2, 4, 6, 8, 10],
            Chords::iid => vec![2, 4, 6, 8, 10],
            Chords::III => vec![3, 5, 7, 9, 11],
            Chords::iii => vec![3, 5, 7, 9, 11],
            Chords::IV => vec![4, 6, 8, 10, 12],
            Chords::iv => vec![4, 6, 8, 10, 12],
            Chords::V => vec![5, 7, 9, 11, 13],
            Chords::v => vec![5, 7, 9, 11, 13],
            Chords::vi => vec![6, 8, 10, 12, 14],
            Chords::VI => vec![6, 8, 10, 12, 14],
            Chords::vii => vec![7, 9, 11, 13, 15],
            Chords::VII => vec![7, 9, 11, 13, 15]
        }
    }
}

#[derive(Clone, Copy)]
enum Key {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B
}

impl Key {
    fn frequency(self) -> f64 {
        match (self) {
            Key::C => {
                261.63
            }
            Key::Cs => {
                277.2
            }
            Key::D => {
                283.66
            }
            Key::Ds => {
                311.1
            }
            Key::E => {
                329.63
            }
            Key::F => {
                349.23
            }
            Key::Fs => {
                370.0
            }
            Key::G => {
                382.0
            }
            Key::Gs => {
                415.3
            }
            Key::A => {
                440.0
            }
            Key::As => {
                466.2
            }
            Key::B => {
                493.88
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Octave {
    LOW,
    MID,
    HIGH
}
type SoundTup = (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>);
fn main() {
    // Setup
    let gpio = Gpio::new().expect("failed to init gpio");
    let i2c = rppal::i2c::I2c::new().expect("failed to open I2C bus!");
    let mut ex_gpio = MCP23017::new(i2c, 0x27).expect("failed to initialize GPIO expander");
    ex_gpio.init_hardware();


    // using an alternate address: https://docs.rs/ssd1306/latest/ssd1306/struct.I2CDisplayInterface.html

    let i2c = rppal::i2c::I2c::new().expect("failed to open I2C bus!");
    let interface = I2CDisplayInterface::new_custom_address(i2c, 0x3C);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();
    display.init().unwrap();

        // init display, set message

        // init filters
    tof::init_eq();
    
        // init TOF sensor & interrupt
    
    let tof_enabled:Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let pass_enabled = tof_enabled.clone(); 
    let tof_sensor: Arc<Mutex<Vl53l1x>> = Arc::new(Mutex::new(tof::init_tof()));
    let thr_sens = tof_sensor.clone();
    let main_thr_sens = tof_sensor.clone();
    let cur_roi: tof::ROIRight = tof::ROIRight::new(true);
    let cur_eq3: AtomicU16 = AtomicU16::new(DEFAULT_EQ_LEVEL);
    let mut tof_int_pin = gpio.get(TOF_INT_PIN).expect("failed to get tof interrupt pin").into_input();
    tof_int_pin.set_async_interrupt(Trigger::FallingEdge, None, move |e| tof::tof_eq_int(e, thr_sens.clone(), &cur_roi, &cur_eq3, &pass_enabled)).expect("failed to setup TOF interrupt");
    let mut sensor = main_thr_sens.lock().expect("failed to lock sensor to begin ranging");
    sensor.start_ranging(vl53l1x::DistanceMode::Short).expect("failed to begin tof ranging");
    drop(sensor);

        // init encoders & interrupt
    let mut enc_a_DT = gpio.get(encoders::ENC_A_DT).expect("couldn't get GPIO").into_input();
    let mut enc_a_CLK= gpio.get(encoders::ENC_A_CLK).expect("couldn't get GPIO").into_input();
   
    let mut enc_b_DT = gpio.get(encoders::ENC_B_DT).expect("couldn't get GPIO").into_input();
    let mut enc_b_CLK= gpio.get(encoders::ENC_B_CLK).expect("couldn't get GPIO").into_input();
    
    let counter_a = Arc::new(AtomicI64::new(0));
    let counter_b = Arc::new(AtomicI64::new(0));
    let counter_a_int = counter_a.clone();
    let counter_b_int = counter_b.clone();
    enc_a_CLK.set_async_interrupt(rppal::gpio::Trigger::FallingEdge, Some(Duration::from_millis(7)),  move |e| encoders::encoder_pos(e, &enc_a_DT, &counter_a_int));
    enc_b_CLK.set_async_interrupt(rppal::gpio::Trigger::FallingEdge, Some(Duration::from_millis(7)),  move |e| encoders::encoder_pos(e, &enc_b_DT, &counter_b_int));
 

        // init keypad
    let i2c = rppal::i2c::I2c::new().expect("failed to open I2C bus!");
    keypad::init_keypad(i2c);
        // TODO: Search for sample file names
        // automatically select sound_xx where xx is the largest integer found there, and record xx + 1 as the next sample name

    // Setup audio backend
    let mut backend =
        backends::CpalBackend::with_default_host_and_device(1,48000,CpalBufferSize::Default).ok_or(backends::CpalBackendError::NoDevice).expect("failed to initilize cpal backend!");
    let mut manager = backend.start(|error| eprintln!("error with cpal output stream: {}", error)).expect("failed to initialize sound manager!");

    // TODO: Grab test sound
    let wav_sound = awedio::sounds::open_file("test_arec.wav").expect("couldn't open audio file");
    let mut test_sound = wav_sound.into_memory_sound().expect("Could not make memory sound");
    let sound = test_sound.clone();

    let mut samples: [f64; 1024] = [0.0; 1024];
    for i in 0..1024 {
        samples[i] = match test_sound.next_sample().expect("not enough samples!") {
            NextSample::Sample(s) =>  {
                let test_sample = (s as f64) / 32768.0;
                test_sample
            },
            _ => 0.0
        };
    }
    
    let mut detector = McLeodDetector::new(SIZE, PADDING);

        // detect frequency and TODO: record frequency
    let pitch = detector
        .get_pitch(&samples, SAMPLE_RATE, POWER_THRESHOLD, CLARITY_THRESHOLD)
        .unwrap();
    let mut current_freq: f64 = pitch.frequency;
    let key =  Key::C;
    let correction: f64 = key.frequency() / (current_freq as f64);
    let mut major = true;
    let mut chord_type = TRIADS;
        // init scale and mode to C major
        // init chords to triads
    let mut current_octave = Octave::MID;
        // init 14 sound "cache" array
    let mut sound_cache: 
        Vec<SoundTup> 
        = Vec::new();
        // init current notes vector
    change_octave_key(sound.clone(), pitch.frequency, &mut sound_cache, key, current_octave);

    // let two: f64 = 2.0;
    // let mut base: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((correction) as f32).stoppable().controllable();
    // sound_cache.push(base); 
    // let mut second: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((2 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(second); 
    // let mut third: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((4 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(third); 
    // let mut fourth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((5 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(fourth); 
    // let mut fifth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((7 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(fifth); 
    // let mut sixth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((9 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(sixth); 
    // let mut seventh: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((11 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(seventh); 
    // let mut base_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * correction) as f32).stoppable().controllable();
    // sound_cache.push(base_oct);
    // let mut second_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((2 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(second_oct); 
    // let mut third_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((4 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(third_oct); 
    // let mut fourth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((5 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(fourth_oct); 
    // let mut fifth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((7 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(fifth_oct); 
    // let mut sixth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((9 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(sixth_oct); 
    // let mut seventh_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((11 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    // sound_cache.push(seventh_oct);
    // let mut base_oct2: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of(((4 as f64) * correction) as f32).stoppable().controllable();
    // sound_cache.push(base_oct2);

    let mut current_notes: 
        Vec<Controller<Stoppable<AdjustableSpeed<MemorySound>>>> 
        = Vec::new();

    let mut hold = false;
    let mut gate = false;
    let mut last_input: Option<keypad::Keypad> = None;
    loop {
        // if volume - previous encoder value is different from current encoder value
        
        // match keypad input
        match keypad::get_keypad(&mut ex_gpio, last_input) {
            // ZERO - Play root note
            Some(keypad::Keypad::ZERO) => {

            },
            // ONE - I chord for major/ i chord minor
            Some(keypad::Keypad::ONE) => {
                if last_input != Some(keypad::Keypad::ONE) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if (major) {
                        play_chord(&mut manager, sound.clone(), correction, Chords::I, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), correction, Chords::i, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::ONE);
            },
            // TWO - ii chord / ii chord (flat 5, 2 semitones between)
            Some(keypad::Keypad::TWO) => {
                if last_input != Some(keypad::Keypad::TWO) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if (major) {
                        play_chord(&mut manager, sound.clone(), correction, Chords::ii, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), correction, Chords::iid, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::TWO);
            },
            // THREE - iii chord/ III chord 
            Some(keypad::Keypad::THREE) => {
                if last_input != Some(keypad::Keypad::THREE) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if (major) {
                        play_chord(&mut manager, sound.clone(), correction, Chords::III, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), correction, Chords::iii, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::THREE);
            },
            // FOUR - IV chord/ iv chord
            Some(keypad::Keypad::FOUR) => {
                if last_input != Some(keypad::Keypad::THREE) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if (major) {
                        play_chord(&mut manager, sound.clone(), correction, Chords::IV, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), correction, Chords::iv, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::FOUR);
            },
            // FIVE - V chord/ v chord
            Some(keypad::Keypad::FIVE) => {
                if last_input != Some(keypad::Keypad::THREE) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if (major) {
                        play_chord(&mut manager, sound.clone(), correction, Chords::V, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), correction, Chords::v, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::FIVE);

            },
            // SIX - vi Chord/  VI chord
            Some(keypad::Keypad::SIX) => {
                if last_input != Some(keypad::Keypad::THREE) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if (major) {
                        play_chord(&mut manager, sound.clone(), correction, Chords::VI, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), correction, Chords::vi, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::SIX);
            },
           Some(keypad::Keypad::SEVEN)=> {
                if last_input != Some(keypad::Keypad::THREE) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if (major) {
                        play_chord(&mut manager, sound.clone(), correction, Chords::VII, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), correction, Chords::vii, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::SEVEN);
            },
            // SEVEN - vii chord (flat 5, 2 semitones between)/ VII dom 7 (7th note has 2 semitones in between)
                // ^^ Record current input
                // if current input contains last input - do nothing, continue looping until sound is complete (with gate on)
                // only if current input is Some() and different from previous, play chord (gate off)
                // Before playing a sound, pull it out of "cache", replace it and place &mut manager in current notes vector
 
            // POUND - Hold playing chord - maybe use completion notifier to wait unless another input before completion. (does nothing with gate off)
            Some(keypad::Keypad::POUND) => {
                hold = true;
            },
            
            None => {
                if gate && !hold {
                    gate_sound(chord_type, &mut current_notes);
                }
                last_input = None;
            }

            Some(keypad::Keypad::EIGHT)=> {
                gate_sound(chord_type, &mut current_notes);
                match current_octave {
                    Octave::LOW => {}
                    Octave::MID => {
                        change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, Octave::LOW);
                        current_octave = Octave::LOW;
                    }
                    Octave::HIGH => {
                        change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, Octave::MID);
                        current_octave = Octave::MID;

                    }
                }
            },
            
            Some(keypad::Keypad::NINE) => {
                gate_sound(chord_type, &mut current_notes);
                match current_octave {
                    Octave::LOW => {
                        change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, Octave::MID);
                        current_octave = Octave::MID;

                    }
                    Octave::MID => {
                        change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, Octave::HIGH);
                        current_octave = Octave::HIGH;
                    }
                    Octave::HIGH => {}
                }
            },

            // Below - only accept these inputs if current input == None
            // A - Change mode
            Some(keypad::Keypad::A) => {
                gate_sound(chord_type, &mut current_notes);
                if major {
                    major = false;
                } else {
                    major = true;
                }
            },

            // B - Gate On/Off
            Some(keypad::Keypad::B) => {
                gate_sound(chord_type, &mut current_notes);
                if gate {
                    gate = false;
                } else {
                    gate = true;
                }

            },
            // C - TOF/Filter On/Off
            Some(keypad::Keypad::C) => {
                gate_sound(chord_type, &mut current_notes);
                if tof_enabled.load(std::sync::atomic::Ordering::SeqCst) {
                    tof_enabled.store(false, std::sync::atomic::Ordering::SeqCst);
                } else {
                    tof_enabled.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            },
            // D - Toggle Triads 7ths or 9ths
            Some(keypad::Keypad::D) => {
                gate_sound(chord_type, &mut current_notes);
                // TODO: Add display logic.
                match chord_type {
                    TRIADS => {
                        chord_type = SEVENTHS;
                    }
                    SEVENTHS => {
                        chord_type = NINTHS;
                    }
                    NINTHS => {
                        chord_type = TRIADS;
                    }
                    _ => {
                        chord_type = TRIADS;
                    }
                }
            },
            // STAR - Record sample
            Some(keypad::Keypad::STAR) => {
                gate_sound(chord_type, &mut current_notes);
                let pre_rec_tof = tof_enabled.load(std::sync::atomic::Ordering::SeqCst);
                tof_enabled.store(false, std::sync::atomic::Ordering::SeqCst);
                // TODO: recording routine
                tof_enabled.store(pre_rec_tof, std::sync::atomic::Ordering::SeqCst);

            },
        }

        // if audio output change - volume encoder push button

        // if root note change - previous encoder value is different from current 

        // if file select toggle - enter sample select mode if in playback

    }    
}

fn play_chord(manager: &mut Manager, sound: MemorySound, correction: f64, chord: Chords, chord_type: u16, major: bool,/*octave,*/ cache: &mut Vec<SoundTup>, curr: &mut Vec<Controller<Stoppable<AdjustableSpeed<MemorySound>>>>) {
    //let mut curr: Vec<SoundTup> = Vec::new();
    for i in 0..chord_type {
        let idx: usize = chord.note_indices()[i as usize] as usize;
        let (play_snd, ctrl_snd) = cache.remove(idx as usize);
        manager.play(Box::new(play_snd));
        curr.push(ctrl_snd);

        if major {
            let new_snd: SoundTup =  sound.clone().with_adjustable_speed_of((MAJ_MUL[idx] * correction) as f32).stoppable().controllable();
            cache.insert(idx, new_snd);
        } else {
            let new_snd =  sound.clone().with_adjustable_speed_of((MIN_MUL[idx] * correction) as f32).stoppable().controllable();
            cache.insert(idx, new_snd);
        }
    }
}

fn gate_sound(chord_type: u16, curr: &mut Vec<Controller<Stoppable<AdjustableSpeed<MemorySound>>>>) {
    for i in  0..chord_type {
        if (i as usize) < curr.len() {
            let mut stop_snd = curr.remove(i as usize);
            stop_snd.set_stopped();
        }
    }
}

fn sample_select(/*scale, mode*/) {
    // start on current sample

    // while root note encoder push button has not been pressed
        // determine encoder left and right to cycle through discovered samples
        // set new sample file as the current sample
    // detect frequency and record

}

// fn record_sample(&mut display: Ssd1306/*scale, mode, sampleno*/) /*-> new sample cache, file to add, sound freq*/ {
//     // disable TOF interrupt
//     // give countdown
//     // record sample
//     // detect frequency and record
//     // enable TOF interrupt
// 
//     let text_style = MonoTextStyleBuilder::new()
//         .font(&FONT_6X10)
//         .text_color(BinaryColor::On)
//         .build();
// 
//     Text::with_baseline("3", Point::new(8, 8), text_style, Baseline::Top)
//         .draw(&mut display)
//         .unwrap();
//     display.flush().unwrap();
//     sleep(Duration::from_secs(1));
//     display.clear_buffer();
//     Text::with_baseline("2", Point::new(8, 8), text_style, Baseline::Top)
//         .draw(&mut display)
//         .unwrap();
//     display.flush().unwrap();
// 
//     sleep(Duration::from_secs(1));
//     display.clear_buffer();
//     Text::with_baseline("1", Point::new(8, 8), text_style, Baseline::Top)
//         .draw(&mut display)
//         .unwrap();
//     display.flush().unwrap();
// 
//     sleep(Duration::from_secs(1));
// 
//     let arec= std::process::Command::new("arecord")
//         .args(vec!["-D", "plughw:1,0", "-f", "S16_LE", "-c", "1", "-r", "48000", "test_arec.wav"])
//         .spawn().expect("Failed to launch arecord!");
//     println!("recording...");
//     
//     display.clear_buffer();
//     Text::with_baseline("Recording", Point::new(8, 8), text_style, Baseline::Top)
//         .draw(&mut display)
//         .unwrap();
//     display.flush().unwrap();
// 
//     
//     // println!("Press enter to stop recording");
//     // let stdin = io::stdin();
//     // let input = &mut String::new();
//     // let _ = stdin.read_line(input);
//     let input = gpio.get(TEMP_PIN).expect("failed to get gpio 27!").into_input();
//     
//     let mut wait = input.is_high();
//     while wait {
//         wait = input.is_high();
//     }
// 
//     // TODO: kill arec in case of sigint failure
//     let _ = nix::sys::signal::kill(nix::unistd::Pid::from_raw(arec.id() as i32), nix::sys::signal::Signal::SIGINT);
// 
// }

// fn change_root(/*sound_freq, sound, new root*/) /*-> new sample cache */ {
// 
// }

fn change_octave_key(sound: MemorySound, freq: f64, sound_cache: &mut Vec<SoundTup>, key: Key, octave: Octave) {
    for i in 0..sound_cache.len() {
        sound_cache.remove(0);
    }
    let correction = match octave {
        Octave::LOW => {
            (key.frequency()/2.0) / (freq as f64)
        }
        Octave::MID => {
            (key.frequency()) / (freq as f64)
        }
        Octave::HIGH => {
            (key.frequency() * 2.0) / (freq as f64)
        }
    };

    let two: f64 = 2.0;
    let mut base: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((correction) as f32).stoppable().controllable();
    sound_cache.push(base); 
    let mut second: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((2 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(second); 
    let mut third: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((4 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(third); 
    let mut fourth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((5 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(fourth); 
    let mut fifth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((7 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(fifth); 
    let mut sixth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((9 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(sixth); 
    let mut seventh: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two.pow((11 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(seventh); 
    let mut base_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * correction) as f32).stoppable().controllable();
    sound_cache.push(base_oct);
    let mut second_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((2 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(second_oct); 
    let mut third_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((4 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(third_oct); 
    let mut fourth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((5 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(fourth_oct); 
    let mut fifth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((7 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(fifth_oct); 
    let mut sixth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((9 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(sixth_oct); 
    let mut seventh_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((two * two.pow((11 as f64)/(12 as f64)) * correction) as f32).stoppable().controllable();
    sound_cache.push(seventh_oct);
    let mut base_oct2: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of(((4 as f64) * correction) as f32).stoppable().controllable();
    sound_cache.push(base_oct2); 

}
