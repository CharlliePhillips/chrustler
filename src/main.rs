use awedio::{backends::{CpalBackend, CpalBufferSize}, manager::Manager, sounds::{MemorySound, wrappers::{AdjustableSpeed, Controllable, Controller, Pausable, Stoppable}}, *};
use nix::libc::major;
use pitch_detection::{detector::{mcleod::McLeodDetector, PitchDetector}, *};
use rppal::{gpio::{Event, Gpio, InputPin, Trigger}, i2c::I2c};
use core::num;
use std::{env, fmt::format, fs, path, sync::{Arc, Mutex, atomic::{AtomicBool, AtomicI64, AtomicU16}}, thread::{current, sleep}, time::{Duration, Instant}};
use std::fs::File;
use std::io;
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::{FONT_6X10, FONT_8X13}},
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
const VOL_LUT: [u16; 76] = [0,5,9,13,17,20,24,26,29,32,34,37,39,41,43,45,47,48,50,52,53,55,56,58,59,60,61,63,64,65,66,67,68,70,71,72,73,74,75,75,76,77,78,79,80,81,81,82,83,84,85,85,86,87,87,88,89,89,90,91,91,92,93,93,94,94,95,96,96,97,97,98,98,99,99,100];

const TRIADS: u16 = 3;
const SEVENTHS: u16 = 4;
const NINTHS: u16 = 5;

const MAJ_MUL: [f64; 15] = [1.0, 1.1225, 1.2599, 1.3348, 1.4983, 1.6818, 1.887, 2.0, 2.2449, 2.5198, 2.6697, 2.9966, 3.3636, 3.7755, 4.0];
const MIN_MUL: [f64; 15] = [1.0, 1.1225, 1.1892, 1.3348, 1.4983, 1.5874, 1.7818, 2.0, 2.2449, 2.3784, 2.6697, 2.9966, 3.1748, 3.5636, 4.0];

const INPUT_TIMEOUT: u64 = 150;
const FULLSCREEN_TIMEOUT: u64 = 75;

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

#[derive(Clone, Copy, Debug)]
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

const KEYS: [Key; 12] = [Key::C, Key::Cs, Key::D, Key::Ds, Key::E, Key::F, Key::Fs, Key::G, Key::Gs, Key::A, Key::As, Key::B];

impl Key {
    fn frequency(self) -> f64 {
        match self {
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
    let mut volume: i64 = 75;
    let vol_string = format!("{}%", VOL_LUT[volume as usize]);
    let vol = vol_string.as_str();
    let _amix = std::process::Command::new("amixer")
        .args(vec!["-c", "1", "cset", "numid=6", vol])
        .spawn().expect("Failed to launch amixer!");
    
    let mut int_io = true;

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
    let cur_lpf: Arc<AtomicU16> = Arc::new(AtomicU16::new(DEFAULT_EQ_LEVEL));
    let cur_hpf: Arc<AtomicU16> = Arc::new(AtomicU16::new(DEFAULT_EQ_LEVEL));
    let pass_lpf = cur_lpf.clone();
    let pass_hpf = cur_hpf.clone();
    let mut tof_int_pin = gpio.get(TOF_INT_PIN).expect("failed to get tof interrupt pin").into_input();
    tof_int_pin.set_async_interrupt(Trigger::FallingEdge, None, move |e| tof::tof_eq_int(e, thr_sens.clone(), &cur_roi, pass_hpf.clone(), pass_lpf.clone(), &pass_enabled)).expect("failed to setup TOF interrupt");
    let mut sensor = main_thr_sens.lock().expect("failed to lock sensor to begin ranging");
    sensor.start_ranging(vl53l1x::DistanceMode::Short).expect("failed to begin tof ranging");
    drop(sensor);

        // init encoders & interrupt
    let mut enc_a_DT = gpio.get(encoders::ENC_A_DT).expect("couldn't get GPIO").into_input();
    let mut enc_a_CLK= gpio.get(encoders::ENC_A_CLK).expect("couldn't get GPIO").into_input();
   
    let mut enc_b_DT = gpio.get(encoders::ENC_B_DT).expect("couldn't get GPIO").into_input();
    let mut enc_b_CLK= gpio.get(encoders::ENC_B_CLK).expect("couldn't get GPIO").into_input();
    let mut last_counter_a: i64 = 0;
    let mut last_counter_b: i64 = 0;
    let counter_a = Arc::new(AtomicI64::new(0));
    let counter_b = Arc::new(AtomicI64::new(0));
    let counter_a_int = counter_a.clone();
    let counter_b_int = counter_b.clone();
    let _ =  enc_a_CLK.set_async_interrupt(rppal::gpio::Trigger::FallingEdge, Some(Duration::from_millis(7)),  move |e| encoders::encoder_pos(e, &enc_a_DT, &counter_a_int));
    let _ = enc_b_CLK.set_async_interrupt(rppal::gpio::Trigger::FallingEdge, Some(Duration::from_millis(7)),  move |e| encoders::encoder_pos(e, &enc_b_DT, &counter_b_int));
 
    let enc_a_pb = gpio.get(encoders::ENC_A_PB).expect("couldn't get GPIO").into_input_pullup();
    let enc_b_pb = gpio.get(encoders::ENC_B_PB).expect("couldn't get GPIO").into_input_pullup();


        // init keypad
    let i2c = rppal::i2c::I2c::new().expect("failed to open I2C bus!");
    keypad::init_keypad(i2c);
        // TODO: Search for sample file names
        // automatically select sound_xx where xx is the largest integer found there, and record xx + 1 as the next sample name

    // Setup audio backend
    let mut backend =
        backends::CpalBackend::with_default_host_and_device(1,48000,CpalBufferSize::Default).ok_or(backends::CpalBackendError::NoDevice).expect("failed to initilize cpal backend!");
    let mut manager = backend.start(|error| eprintln!("error with cpal output stream: {}", error)).expect("failed to initialize sound manager!");

    let mut next_sample_no: usize = 0;
    let mut sample_paths: Vec<String> = Vec::new();

    // These three file operations should not fail on a Pi with user logged in.
    let mut media_users= fs::read_dir("/media").expect("No '/media' Directory!"); // list users
    let user_media = media_users.next().unwrap().expect("No media users!"); // get user dir
    let mut user_media_dir = fs::read_dir(user_media.path()).expect("No USB media!"); // list user drives
    // ^^^

    // If the usb drive is plugged in use that, if not default to CWD
    let mut media_path_entry = match user_media_dir.next() {
        Some(usb_media_path) => {
            usb_media_path.expect("No USB media!").path() // get user drive
        }
        None => {
            env::current_dir().expect("No current working dir!")
        }
    };
    
    println!("using {:#?}", media_path_entry);
    let user_media_dir = match fs::read_dir(media_path_entry.clone()) { // list user drive files
         Ok(dir) => dir,
         Err(_) => {
            println!("using {:#?}", media_path_entry);
            media_path_entry = env::current_dir().expect("No current working dir!");
            fs::read_dir(media_path_entry.clone()).expect("failed to read CWD!")
         }
    };
    let media_path = media_path_entry.to_str().unwrap().to_string();
    
    for entry_res in user_media_dir {
        match entry_res {
            Ok(entry) => {
                if entry.path().extension().is_some() {
                    if entry.path().extension().unwrap().to_str().is_some() {
                        if entry.path().extension().unwrap().to_str().unwrap().eq("wav") {
                            let this_path = entry.path().to_str().unwrap().to_string();
                            sample_paths.push(entry.path().to_str().unwrap().to_string());
                            let after_slash = this_path.rfind("/").unwrap() + 1;
                            let dot = this_path.find(".").unwrap();
                            if this_path[after_slash..after_slash + 6].eq("sound_") {
                                println!("found '{}'", this_path);
                                match this_path[after_slash + 6..dot].parse() {
                                    Ok(sampleno) => {
                                        if sampleno > next_sample_no {
                                            next_sample_no = sampleno;
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }

    next_sample_no = next_sample_no + 1;

    let mut current_sample_idx: usize = 0;
    let init_smpl_path = if sample_paths.len() > 0 {
        sample_paths[current_sample_idx].clone()
    } else {
        "test_arec.wav".to_string()
    };

    let wav_sound = awedio::sounds::open_file(init_smpl_path).expect("couldn't open audio file");
    let mut test_sound = wav_sound.into_memory_sound().expect("Could not make memory sound");
    let mut sound = test_sound.clone();

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

    let mut key =  Key::C;
    let mut key_idx = 0;
    //let correction: f64 = key.frequency() / (current_freq as f64);
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
    change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, current_octave, major);

    let mut current_notes: 
        Vec<Controller<Stoppable<AdjustableSpeed<MemorySound>>>> 
        = Vec::new();

    let mut hold = false;
    let mut gate = false;
    let mut last_input: Option<keypad::Keypad> = None;
    loop {
        
        // match keypad input
        match keypad::get_keypad(&mut ex_gpio, last_input) {
            // ZERO - Play root note
            Some(keypad::Keypad::ZERO) => {
                if last_input != Some(keypad::Keypad::ZERO) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    let correction = match current_octave{
                        Octave::LOW => {
                            (key.frequency()/2.0) / (current_freq as f64)
                        }
                        Octave::MID => {
                            (key.frequency()) / (current_freq as f64)
                        }
                        Octave::HIGH => {
                            (key.frequency() * 2.0) / (current_freq as f64)
                        }
                    };
                        let (play_snd, ctrl_snd) = sound_cache.remove((0) as usize);
                        manager.play(Box::new(play_snd));
                        current_notes.push(ctrl_snd);

                        if major {
                            let new_snd: SoundTup =  sound.clone().with_adjustable_speed_of((MAJ_MUL[0] * correction) as f32).stoppable().controllable();
                            sound_cache.insert(0, new_snd);
                        } else {
                            let new_snd =  sound.clone().with_adjustable_speed_of((MIN_MUL[0] * correction) as f32).stoppable().controllable();
                            sound_cache.insert(0, new_snd);
                        }
                }
                last_input = Some(keypad::Keypad::ZERO);
            },
            // ONE - I chord for major/ i chord minor
            Some(keypad::Keypad::ONE) => {
                if last_input != Some(keypad::Keypad::ONE) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if major {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::I, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::i, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::ONE);
            },
            // TWO - ii chord / ii chord (flat 5, 2 semitones between)
            Some(keypad::Keypad::TWO) => {
                if last_input != Some(keypad::Keypad::TWO) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if major {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::ii, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::iid, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::TWO);
            },
            // THREE - iii chord/ III chord 
            Some(keypad::Keypad::THREE) => {
                if last_input != Some(keypad::Keypad::THREE) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if major {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::III, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::iii, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::THREE);
            },
            // FOUR - IV chord/ iv chord
            Some(keypad::Keypad::FOUR) => {
                if last_input != Some(keypad::Keypad::FOUR) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if major {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::IV, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::iv, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::FOUR);
            },
            // FIVE - V chord/ v chord
            Some(keypad::Keypad::FIVE) => {
                if last_input != Some(keypad::Keypad::FIVE) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if major {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::V, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::v, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::FIVE);
            },
            // SIX - vi Chord/  VI chord
            Some(keypad::Keypad::SIX) => {
                if last_input != Some(keypad::Keypad::SIX) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if major {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::VI, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::vi, chord_type, major, &mut sound_cache, &mut current_notes);
                    }
                }
                last_input = Some(keypad::Keypad::SIX);
            },
           Some(keypad::Keypad::SEVEN)=> {
                if last_input != Some(keypad::Keypad::SEVEN) {
                    hold = false;
                    gate_sound(chord_type, &mut current_notes);
                    if major {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::VII, chord_type, major, &mut sound_cache, &mut current_notes);
                    } else {
                        play_chord(&mut manager, sound.clone(), key, current_octave, current_freq, Chords::vii, chord_type, major, &mut sound_cache, &mut current_notes);
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
                        change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, Octave::LOW, major);
                        current_octave = Octave::LOW;
                        sleep(Duration::from_millis(INPUT_TIMEOUT));
                    }
                    Octave::HIGH => {
                        change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, Octave::MID, major);
                        current_octave = Octave::MID;
                        sleep(Duration::from_millis(INPUT_TIMEOUT));
                    }
                }
                last_input = Some(keypad::Keypad::EIGHT);
            },
            
            Some(keypad::Keypad::NINE) => {
                gate_sound(chord_type, &mut current_notes);
                match current_octave {
                    Octave::LOW => {
                        change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, Octave::MID, major);
                        current_octave = Octave::MID;
                        sleep(Duration::from_millis(INPUT_TIMEOUT));
                    }
                    Octave::MID => {
                        change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, Octave::HIGH, major);
                        current_octave = Octave::HIGH;
                        sleep(Duration::from_millis(INPUT_TIMEOUT));
                    }
                    Octave::HIGH => {}
                }
                last_input = Some(keypad::Keypad::NINE);
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
                change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, current_octave, major);
                sleep(Duration::from_millis(INPUT_TIMEOUT));
                last_input = Some(keypad::Keypad::A);
            },

            // B - Gate On/Off
            Some(keypad::Keypad::B) => {
                gate_sound(chord_type, &mut current_notes);
                if gate {
                    gate = false;
                } else {
                    gate = true;
                }
                sleep(Duration::from_millis(INPUT_TIMEOUT));
                last_input = Some(keypad::Keypad::B);
            },
            // C - TOF/Filter On/Off
            Some(keypad::Keypad::C) => {
                gate_sound(chord_type, &mut current_notes);
                if tof_enabled.load(std::sync::atomic::Ordering::SeqCst) {
                    tof_enabled.store(false, std::sync::atomic::Ordering::SeqCst);
                    println!("TOF disabled");
                } else {
                    tof_enabled.store(true, std::sync::atomic::Ordering::SeqCst);
                    println!("TOF enabled");
                }
                sleep(Duration::from_millis(INPUT_TIMEOUT));
                last_input = Some(keypad::Keypad::C);
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
                sleep(Duration::from_millis(INPUT_TIMEOUT));
                last_input = Some(keypad::Keypad::D);
            },
            // STAR - Record sample
            Some(keypad::Keypad::STAR) => {
                gate_sound(chord_type, &mut current_notes);
                let pre_rec_tof = tof_enabled.load(std::sync::atomic::Ordering::SeqCst);
                tof_enabled.store(false, std::sync::atomic::Ordering::SeqCst);
                let mut sound_dat = None;
                (backend, manager, sound_dat) = record_sample(media_path.clone(), &mut sample_paths, &mut current_sample_idx, &mut next_sample_no, &mut ex_gpio, backend, manager, &mut display);
                
                match sound_dat {
                    Some((new_snd, new_freq)) => {
                        sound = new_snd;
                        current_freq = new_freq;
                        change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, current_octave, major);
                    }
                    None => {}
                }
                tof_enabled.store(pre_rec_tof, std::sync::atomic::Ordering::SeqCst);
                last_input = Some(keypad::Keypad::STAR);
            },

            // To keep rust compiler happy, this accounts for Volume or Key change
            _ => {}
        }

        //update_display(&mut display, key, major, current_octave, 50, cur_hpf.load(std::sync::atomic::Ordering::SeqCst), cur_lpf.load(std::sync::atomic::Ordering::SeqCst), chord_type, gate);
        
        
        // if volume - previous encoder value is different from current encoder value
        let cur_counter_a = counter_a.load(std::sync::atomic::Ordering::SeqCst);
        if cur_counter_a != last_counter_a {
            let vol_diff: i64 = cur_counter_a - last_counter_a;
            let new_vol = volume + vol_diff;
            
            volume = if new_vol > 75 {
                75
            } else if new_vol < 0 {
                0
            } else {
                new_vol
            };

            let pre_rec_tof = tof_enabled.load(std::sync::atomic::Ordering::SeqCst);
            tof_enabled.store(false, std::sync::atomic::Ordering::SeqCst);
           
            let vol_string = format!("{}%", VOL_LUT[volume as usize]);
            let vol = vol_string.as_str();
            let _amix = std::process::Command::new("amixer")
                .args(vec!["-q", "-c", "1", "cset", "numid=6", vol])
                .spawn().expect("Failed to launch amixer!");
            
            tof_enabled.store(pre_rec_tof, std::sync::atomic::Ordering::SeqCst);
            fullscreen_msg(&mut display, format!("Volume: {}%", (100.0 * (volume as f32 / 75.0)).round() as u16));
            sleep(Duration::from_millis(FULLSCREEN_TIMEOUT));
            last_input = Some(keypad::Keypad::VOL);
        } else {
            update_display(&mut display, key, major, current_octave, tof_enabled.load(std::sync::atomic::Ordering::SeqCst), cur_hpf.load(std::sync::atomic::Ordering::SeqCst), cur_lpf.load(std::sync::atomic::Ordering::SeqCst), chord_type, gate);
        }
        last_counter_a = cur_counter_a;
        
        let mut cur_counter_b = counter_b.load(std::sync::atomic::Ordering::SeqCst);
        if last_input == None {
            // if audio output change - volume encoder push button
            if enc_a_pb.is_low() {
                int_io = set_io(int_io, &mut display);
                last_input = Some(keypad::Keypad::IO);
            }

            // if root note change - previous encoder value is different from current 
            if cur_counter_b != last_counter_b {
                gate_sound(chord_type, &mut current_notes);
                let key_diff: i64 = cur_counter_b - last_counter_b;
                let new_idx: i64= key_idx + key_diff;
            
                key_idx = if new_idx > 11 {
                    11
                } else if new_idx < 0 {
                    0
                } else {
                    new_idx
                };

                key = KEYS[key_idx as usize];
                change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, current_octave, major);
                update_display(&mut display, key, major, current_octave, tof_enabled.load(std::sync::atomic::Ordering::SeqCst), cur_hpf.load(std::sync::atomic::Ordering::SeqCst), cur_lpf.load(std::sync::atomic::Ordering::SeqCst), chord_type, gate);
                last_input = Some(keypad::Keypad::KEY);
            }

            // if file select toggle - enter sample select mode if in playback
            if enc_b_pb.is_low() {
                gate_sound(chord_type, &mut current_notes);
                match sample_select(&sample_paths, &mut current_sample_idx, &enc_b_pb, counter_b.clone(), &mut cur_counter_b, &mut display) {
                    Some((new_sound, new_freq)) => {
                        current_freq = new_freq;
                        sound = new_sound;
                        change_octave_key(sound.clone(), current_freq, &mut sound_cache, key, current_octave, major);
                    }
                    None => {}
                }
            }
        }
        last_counter_b = cur_counter_b;
    }    
}

fn play_chord(manager: &mut Manager, sound: MemorySound, key: Key, octave: Octave, freq: f64, chord: Chords, chord_type: u16, major: bool, cache: &mut Vec<SoundTup>, curr: &mut Vec<Controller<Stoppable<AdjustableSpeed<MemorySound>>>>) {
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
    for i in 0..chord_type {
        let idx: usize = chord.note_indices()[i as usize] as usize;
        let (play_snd, ctrl_snd) = cache.remove((idx - 1) as usize);
        manager.play(Box::new(play_snd));
        curr.push(ctrl_snd);

        if major {
            let new_snd: SoundTup =  sound.clone().with_adjustable_speed_of((MAJ_MUL[idx - 1] * correction) as f32).stoppable().controllable();
            cache.insert(idx - 1, new_snd);
        } else {
            let new_snd =  sound.clone().with_adjustable_speed_of((MIN_MUL[idx - 1] * correction) as f32).stoppable().controllable();
            cache.insert(idx - 1, new_snd);
        }
    }
}

fn gate_sound(chord_type: u16, curr: &mut Vec<Controller<Stoppable<AdjustableSpeed<MemorySound>>>>) {
    for _i in  0..chord_type {
        if (0) < curr.len() {
            let mut stop_snd = curr.remove(0);
            stop_snd.set_stopped();
        }
    }
}

fn sample_select(sample_paths: &Vec<String>, current_smpl_idx: &mut usize, enc_pb: &InputPin, enc_cnt: Arc<AtomicI64>, cur_enc_cnt: &mut i64, display: &mut Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>) -> Option<(MemorySound, f64)> {
    // start on current sample

    // while root note encoder push button has not been pressed
        // determine encoder left and right to cycle through discovered samples
        // set new sample file as the current sample
    // detect frequency and record
    fullscreen_msg(display, "Sample Select".to_string());
    sleep(Duration::from_millis(1000));

    let mut cur_smpl = sample_paths[*current_smpl_idx].clone();
    let mut name_begin = cur_smpl.rfind("/").unwrap_or(0);
    if name_begin > 0 {
        name_begin += 1;
    }
    let trunc_smpl = &cur_smpl[name_begin..cur_smpl.len()-4];
    let trunc_smpl_string = if trunc_smpl.len() > 16 {
        trunc_smpl[0..16].to_string()
    } else {
        trunc_smpl.to_string()
    };
    fullscreen_msg(display, trunc_smpl_string);
    sleep(Duration::from_millis(500));
    
    let mut last_enc_cnt = *cur_enc_cnt;
    loop {
        *cur_enc_cnt = enc_cnt.load(std::sync::atomic::Ordering::SeqCst);

        if *cur_enc_cnt != last_enc_cnt {
            let enc_diff: i64 = *cur_enc_cnt - last_enc_cnt;
            let new_idx: i64 = (*current_smpl_idx as i64) + enc_diff;
            
            *current_smpl_idx = if new_idx >= (sample_paths.len() as i64) {
                0
            } else if new_idx < 0 {
                sample_paths.len() - 1
            } else {
                new_idx as usize
            };
        }
        
        cur_smpl = sample_paths[*current_smpl_idx].clone();
        let mut name_begin = cur_smpl.rfind("/").unwrap_or(0);
        if name_begin > 0 {
            name_begin += 1;
        }
        let trunc_smpl = &cur_smpl[name_begin..cur_smpl.len()-4];
        let trunc_smpl_string = if trunc_smpl.len() > 16 {
            trunc_smpl[0..16].to_string()
        } else {
            trunc_smpl.to_string()
        };
        fullscreen_msg(display, trunc_smpl_string);
        //sleep(Duration::from_millis(500));

        if enc_pb.is_low() {
            break;
        }
        last_enc_cnt = *cur_enc_cnt;
    }
    let wav_sound = match awedio::sounds::open_file(cur_smpl) {
        Ok(sound) => sound,
        Err(_) => {
            fullscreen_msg(display, "Err opening!".to_string());
            sleep(Duration::from_secs(1));
            return None
        }
    };
    let mut test_sound = match wav_sound.into_memory_sound() {
        Ok(mem_snd) => mem_snd,
        Err(_) => {
            fullscreen_msg(display, "Err loading!".to_string());
            sleep(Duration::from_secs(1));
            return None
        }
    };

    let out_sound = test_sound.clone();

    let mut samples: [f64; 1024] = [0.0; 1024];
    for i in 0..1024 {
        samples[i] = match test_sound.next_sample() {
            Ok(sample) => {
                match sample {
                    NextSample::Sample(s) =>  {
                        let test_sample = (s as f64) / 32768.0;
                        test_sample
                    },
                    _ => 0.0
                }
            }
            Err(_) => {
                fullscreen_msg(display, "Too short!".to_string());
                sleep(Duration::from_secs(1));
                return None
            }
        } 
    }
    
    let mut detector = McLeodDetector::new(SIZE, PADDING);

        // detect frequency and TODO: record frequency
    let pitch = match detector
        .get_pitch(&samples, SAMPLE_RATE, POWER_THRESHOLD, CLARITY_THRESHOLD) {
            Some(pitch) => pitch,
            None => {
                fullscreen_msg(display, "Err no pitch!".to_string());
                sleep(Duration::from_secs(1));
                return None
            }
        };
    let out_freq: f64 = pitch.frequency;
    
    Some((out_sound, out_freq))
}

fn record_sample(media_path: String, sample_paths: &mut Vec<String>, current_smpl_idx: &mut usize, next_smpl_no: &mut usize, ex_gpio: &mut MCP23017<I2c>, backend: CpalBackend, manager: Manager, display: &mut Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>) -> (CpalBackend, Manager, Option<(MemorySound, f64)>) {

//     // give countdown
//     // record sample
//     // detect frequency
    drop(manager);
    drop(backend);
    //manager = backends::CpalBackend::new(1, 48000, CpalBufferSize::Default, cpal::platform::, sample_format)

    let sample_name = format!("sound_{}.wav", next_smpl_no);
    let rec_path = format!("{}/{}", media_path, sample_name);

    fullscreen_msg(display, "Recording in 3".to_string());
    sleep(Duration::from_secs(1));
    fullscreen_msg(display, "Recording in 2".to_string());
    sleep(Duration::from_secs(1));
    fullscreen_msg(display, "Recording in 1".to_string());
    sleep(Duration::from_secs(1));
 
    let mut arec= match std::process::Command::new("arecord")
        .args(vec!["-D", "plughw:1,0", "-f", "S32_LE", "-c", "1", "-r", "48000", rec_path.as_str()])
        .spawn() {
            Ok(arec) => arec,
            Err(_) => {
                fullscreen_msg(display, "Recording fail!".to_string());
                sleep(Duration::from_secs(1));
                let mut backend =
                    backends::CpalBackend::with_default_host_and_device(1,48000,CpalBufferSize::Default).ok_or(backends::CpalBackendError::NoDevice).expect("failed to initilize cpal backend!");
                let mut manager = backend.start(|error| eprintln!("error with cpal output stream: {}", error)).expect("failed to initialize sound manager!");
                return (backend, manager, None)
            }
        };
    
    fullscreen_msg(display, "Recording...".to_string());

    let rec_start_time = Instant::now();
    // Sample up to 10 minutes!
    let max_rec_time = Duration::from_secs(600); 
    loop {
        match arec.try_wait() {
            Ok(complete) => {
                match complete {
                    Some(_status) => break,
                    None => {}
                }
            }
            Err(_) => {
                fullscreen_msg(display, "System error!".to_string());
                sleep(Duration::from_secs(1));
                let mut backend =
                    backends::CpalBackend::with_default_host_and_device(1,48000,CpalBufferSize::Default).ok_or(backends::CpalBackendError::NoDevice).expect("failed to initilize cpal backend!");
                let mut manager = backend.start(|error| eprintln!("error with cpal output stream: {}", error)).expect("failed to initialize sound manager!");
                return (backend, manager, None)
            }
        }

        if keypad::get_keypad(ex_gpio, None) == Some(keypad::Keypad::STAR) || rec_start_time.elapsed() > max_rec_time {
            match nix::sys::signal::kill(nix::unistd::Pid::from_raw(arec.id() as i32), nix::sys::signal::Signal::SIGINT) {
                Ok(_) => {},
                Err(_) => {
                    let _ = nix::sys::signal::kill(nix::unistd::Pid::from_raw(arec.id() as i32), nix::sys::signal::Signal::SIGKILL);
                }
            }
            break;
        }
    }

    sleep(Duration::from_millis(500));
    let wav_sound = match awedio::sounds::open_file(rec_path.clone()) {
        Ok(sound) => sound,
        Err(_) => {
            fullscreen_msg(display, "Err opening!".to_string());
            sleep(Duration::from_secs(1));
            let mut backend =
                backends::CpalBackend::with_default_host_and_device(1,48000,CpalBufferSize::Default).ok_or(backends::CpalBackendError::NoDevice).expect("failed to initilize cpal backend!");
            let mut manager = backend.start(|error| eprintln!("error with cpal output stream: {}", error)).expect("failed to initialize sound manager!");
            return (backend, manager, None)
        }
    };
    let mut test_sound = match wav_sound.into_memory_sound() {
        Ok(mem_snd) => mem_snd,
        Err(_) => {
            fullscreen_msg(display, "Err loading!".to_string());
            sleep(Duration::from_secs(1));
            let mut backend =
                backends::CpalBackend::with_default_host_and_device(1,48000,CpalBufferSize::Default).ok_or(backends::CpalBackendError::NoDevice).expect("failed to initilize cpal backend!");
            let mut manager = backend.start(|error| eprintln!("error with cpal output stream: {}", error)).expect("failed to initialize sound manager!");
            return (backend, manager, None)

        }
    };

    let out_sound = test_sound.clone();

    let mut samples: [f64; 1024] = [0.0; 1024];
    for i in 0..1024 {
        samples[i] = match test_sound.next_sample() {
            Ok(sample) => {
                match sample {
                    NextSample::Sample(s) =>  {
                        let test_sample = (s as f64) / 32768.0;
                        test_sample
                    },
                    _ => 0.0
                }
            }
            Err(_) => {
                fullscreen_msg(display, "Too short!".to_string());
                sleep(Duration::from_secs(1));
                let mut backend =
                    backends::CpalBackend::with_default_host_and_device(1,48000,CpalBufferSize::Default).ok_or(backends::CpalBackendError::NoDevice).expect("failed to initilize cpal backend!");
                let mut manager = backend.start(|error| eprintln!("error with cpal output stream: {}", error)).expect("failed to initialize sound manager!");
                return (backend, manager, None)
            }
        } 
    }
    
    let mut detector = McLeodDetector::new(SIZE, PADDING);

        // detect frequency and TODO: record frequency
    let pitch = match detector
        .get_pitch(&samples, SAMPLE_RATE, POWER_THRESHOLD, CLARITY_THRESHOLD) {
            Some(pitch) => pitch,
            None => {
                fullscreen_msg(display, "Err no pitch!".to_string());
                sleep(Duration::from_secs(1));
                let mut backend =
                    backends::CpalBackend::with_default_host_and_device(1,48000,CpalBufferSize::Default).ok_or(backends::CpalBackendError::NoDevice).expect("failed to initilize cpal backend!");
                let mut manager = backend.start(|error| eprintln!("error with cpal output stream: {}", error)).expect("failed to initialize sound manager!");
                return (backend, manager, None)
            }
        };
    let out_freq: f64 = pitch.frequency;
    
    *next_smpl_no += 1;
    sample_paths.push(rec_path);
    *current_smpl_idx = sample_paths.len() - 1;

    let mut backend =
        backends::CpalBackend::with_default_host_and_device(1,48000,CpalBufferSize::Default).ok_or(backends::CpalBackendError::NoDevice).expect("failed to initilize cpal backend!");
    let mut manager = backend.start(|error| eprintln!("error with cpal output stream: {}", error)).expect("failed to initialize sound manager!");

    (backend, manager, Some((out_sound, out_freq)))
}

fn change_octave_key(sound: MemorySound, freq: f64, sound_cache: &mut Vec<SoundTup>, key: Key, octave: Octave, major: bool) {
    for _i in 0..sound_cache.len() {
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

    if major {
        let base: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[0] * correction) as f32).stoppable().controllable();
        sound_cache.push(base); 
        let second: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[1] * correction) as f32).stoppable().controllable();
        sound_cache.push(second); 
        let third: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[2] * correction) as f32).stoppable().controllable();
        sound_cache.push(third); 
        let fourth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[3] * correction) as f32).stoppable().controllable();
        sound_cache.push(fourth); 
        let fifth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[4] * correction) as f32).stoppable().controllable();
        sound_cache.push(fifth); 
        let sixth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[5] * correction) as f32).stoppable().controllable();
        sound_cache.push(sixth); 
        let seventh: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[6] * correction) as f32).stoppable().controllable();
        sound_cache.push(seventh); 
        let base_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[7] * correction) as f32).stoppable().controllable();
        sound_cache.push(base_oct);
        let second_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[8] * correction) as f32).stoppable().controllable();
        sound_cache.push(second_oct); 
        let third_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[9] * correction) as f32).stoppable().controllable();
        sound_cache.push(third_oct); 
        let fourth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[10] * correction) as f32).stoppable().controllable();
        sound_cache.push(fourth_oct); 
        let fifth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[11] * correction) as f32).stoppable().controllable();
        sound_cache.push(fifth_oct); 
        let sixth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[12] * correction) as f32).stoppable().controllable();
        sound_cache.push(sixth_oct); 
        let seventh_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[13] * correction) as f32).stoppable().controllable();
        sound_cache.push(seventh_oct);
        let base_oct2: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MAJ_MUL[14] * correction) as f32).stoppable().controllable();
        sound_cache.push(base_oct2); 
    } else {
        let base: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[0] * correction) as f32).stoppable().controllable();
        sound_cache.push(base); 
        let second: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[1] * correction) as f32).stoppable().controllable();
        sound_cache.push(second); 
        let third: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[2] * correction) as f32).stoppable().controllable();
        sound_cache.push(third); 
        let fourth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[3] * correction) as f32).stoppable().controllable();
        sound_cache.push(fourth); 
        let fifth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[4] * correction) as f32).stoppable().controllable();
        sound_cache.push(fifth); 
        let sixth: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[5] * correction) as f32).stoppable().controllable();
        sound_cache.push(sixth); 
        let seventh: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[6] * correction) as f32).stoppable().controllable();
        sound_cache.push(seventh); 
        let base_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[7] * correction) as f32).stoppable().controllable();
        sound_cache.push(base_oct);
        let second_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[8] * correction) as f32).stoppable().controllable();
        sound_cache.push(second_oct); 
        let third_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[9] * correction) as f32).stoppable().controllable();
        sound_cache.push(third_oct); 
        let fourth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[10] * correction) as f32).stoppable().controllable();
        sound_cache.push(fourth_oct); 
        let fifth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[11] * correction) as f32).stoppable().controllable();
        sound_cache.push(fifth_oct); 
        let sixth_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[12] * correction) as f32).stoppable().controllable();
        sound_cache.push(sixth_oct); 
        let seventh_oct: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[13] * correction) as f32).stoppable().controllable();
        sound_cache.push(seventh_oct);
        let base_oct2: (Controllable<Stoppable<AdjustableSpeed<MemorySound>>>, Controller<Stoppable<AdjustableSpeed<MemorySound>>>) = sound.clone().with_adjustable_speed_of((MIN_MUL[14] * correction) as f32).stoppable().controllable();
        sound_cache.push(base_oct2); 
    }

}

fn update_display(display: &mut Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>, key: Key, major: bool, octave: Octave, tof: bool, hpf: u16, lpf: u16, chord_type: u16, gate: bool) {
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_8X13)
        .text_color(BinaryColor::On)
        .build();

    let key_text: String = format!("Key:{:#?}", key); 
    let mode_text: String = if major {
        format!("Md:maj")
    } else {
        format!("Md:min")
    };
    let oct_text = match octave {
        Octave::LOW => format!("Oct:Low"),
        Octave::MID => format!("Oct:Mid"),
        Octave::HIGH => format!("Oct:Hi")
    };

    let chord_text: String = match chord_type {
        TRIADS => format!("Typ:Tri"),
        SEVENTHS => format!("Typ:7th"),
        NINTHS=> format!("Typ:9th"),
        _ => format!("Typ:Tri"),
    };   

    let tof_text: String = if tof {
        format!("TOF:On")
    } else {
        format!("TOF:Off")
    };

    let hpf_text: String = format!("HF:{:#?}", hpf); 
    let lpf_text: String = format!("LF:{:#?}", lpf);

    let gate_text: String = if gate {
        format!("Gat:ON")
    } else {
        format!("Gat:OFF")
    };

    display.clear_buffer(); 
    Text::with_baseline(&key_text, Point::new(2, 2), text_style, Baseline::Top)
        .draw(display)
        .unwrap();
    Text::with_baseline(&mode_text, Point::new(2, 16), text_style, Baseline::Top)
        .draw(display)
        .unwrap();
    Text::with_baseline(&oct_text, Point::new(2, 30), text_style, Baseline::Top)
        .draw(display)
        .unwrap();
    Text::with_baseline(&chord_text, Point::new(2, 44), text_style, Baseline::Top)
        .draw(display)
        .unwrap();
    Text::with_baseline(&hpf_text, Point::new(64, 2), text_style, Baseline::Top)
        .draw(display)
        .unwrap();
    Text::with_baseline(&lpf_text, Point::new(64, 16), text_style, Baseline::Top)
        .draw(display)
        .unwrap();
    Text::with_baseline(&tof_text, Point::new(64,30), text_style, Baseline::Top)
        .draw(display)
        .unwrap();
    Text::with_baseline(&gate_text, Point::new(64, 44), text_style, Baseline::Top)
        .draw(display)
        .unwrap();

    display.flush().unwrap(); 
}

fn fullscreen_msg(display: &mut Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>, text: String) {
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_8X13)
        .text_color(BinaryColor::On)
        .build();

    display.clear_buffer(); 

    let x: i32 = 64 - (((text.len() as i32) * 8) / 2);
    Text::with_baseline(&text, Point::new(x, 26), text_style, Baseline::Top)
        .draw(display)
        .unwrap();

    display.flush().unwrap(); 

}

fn set_io(int_io: bool, display: &mut Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>) -> bool {
    if int_io {
        let _hp_en= std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=7", "100"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));

        let _spk_dis= std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=8", "0"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));

        fullscreen_msg(display, "I/O External".to_string());
        let _aux_en = std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=78", "on"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));

        let _aux_vol= std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=3", "40"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));

        let _mems_mic_dis = std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=77", "off"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));
        
        let _mems_mic_vol= std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=2", "0"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));
        false
    } else {
        fullscreen_msg(display, "I/O Internal".to_string());
        let _hp_dis= std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=7", "0"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));

        let _spk_en= std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=8", "88"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));

        let _aux_dis= std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=78", "off"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));

        let _aux_vol= std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=3", "0"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));

        let _mems_mic_en= std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=77", "on"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));

        let _mems_mic_vol= std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", "numid=2", "15"])
            .spawn().expect("Failed to launch amixer!");
        sleep(std::time::Duration::from_millis(50));

        true
    }
}
