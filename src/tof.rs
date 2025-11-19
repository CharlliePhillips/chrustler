use vl53l1x::{Vl53l1x, Vl53l1xRangeStatus};
use rppal::{gpio::{Event, Gpio, Trigger}, i2c::I2c};
use std::{env, fs, io, sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU16}}, thread::sleep, time::Duration};

pub enum FilterType {
    HPF,
    LPF,
}
pub type ROIRight = AtomicBool;

pub fn init_eq() {
    let _amix_en = std::process::Command::new("amixer")
        .args(vec!["-c", "1", "cset", "numid=9", "on"])
        .spawn().expect("Failed to launch amixer!");
    sleep(std::time::Duration::from_millis(50));
    for freq in 10..15 {
        let numid_string = format!("numid={}", freq);
        let numid= numid_string.as_str();
        let _amix = std::process::Command::new("amixer")
            .args(vec!["-c", "1", "cset", numid, "12"])
            .spawn().expect("Failed to launch amixer!");
    }
}

fn set_eq(freq: u8, level: i8) {
    if freq > 5 || freq == 0 || level < 0 {
        return;
    }
    let numid_string = format!("numid={}", (freq + 9));
    let numid= numid_string.as_str();
    let lev_string = level.to_string();
    let lev = lev_string.as_str();
    let _amix = std::process::Command::new("amixer")
        .args(vec!["-q", "-c", "1", "cset", numid, lev])
        .output();
    // pray this doesn't cause any issues...

}

pub fn init_tof() -> Vl53l1x {
    let mut tof_sensor = Vl53l1x::new(1, None).expect("Failed to create TOF sensor struct");
    tof_sensor.soft_reset().expect("Failed to reset TOF sensor");
    tof_sensor.init().expect("Failed to init TOF sensor");
    tof_sensor.set_measurement_timing_budget(20000).expect("failed to set measurement timing");
    tof_sensor.set_inter_measurement_period(24).expect("failed to set inter-measurement timing");

    tof_sensor.set_user_roi(8, 15, 15, 0).expect("failed to set ROI Right");
    
    println!("initilized TOF sensor");
    return tof_sensor;
}

pub fn tof_eq_int(_event: Event, tof_sensor: Arc<Mutex<Vl53l1x>>, cur_roi: &ROIRight, cur_hpf: Arc<AtomicU16>, cur_lpf: Arc<AtomicU16>, enabled: &Arc<AtomicBool>) {
    //println!("TOF interrupt");
    let mut sensor = tof_sensor.lock().expect("failed to acquire sensor lock");
    let sample = sensor.read_sample().expect("failed to get right sample");
    //println!("sampled: {}mm ({:#?})", sample.distance, sample.status);
    if enabled.load(std::sync::atomic::Ordering::SeqCst) {
        match sample.status {
            Vl53l1xRangeStatus::Ok => {
                let filter_strength: i8 = if sample.distance < 156 {
                    (sample.distance/13).try_into().unwrap()
                } else {
                    12
                };
                if cur_roi.load(std::sync::atomic::Ordering::SeqCst) {
                    set_filter(FilterType::LPF, filter_strength, cur_hpf, cur_lpf);
                    cur_roi.store(false, std::sync::atomic::Ordering::SeqCst);
                    sensor.set_user_roi(0, 15, 3, 0).expect("failed to set ROI Left during interrupt");
                } else {
                    set_filter(FilterType::HPF, filter_strength, cur_hpf, cur_lpf);
                    cur_roi.store(true, std::sync::atomic::Ordering::SeqCst);
                    sensor.set_user_roi(12, 15, 15, 0).expect("failed to set ROI Right during interrupt");
                }
            }
            _ => {}
        }
    }
}

fn set_filter(filter: FilterType, strength: i8, cur_hpf: Arc<AtomicU16>, cur_lpf: Arc<AtomicU16>) {
    match filter {
        FilterType::LPF => {
            set_eq(1, strength);
            if strength < 12 {
                set_eq(2, strength/2);
            } else {
                set_eq(2, strength);
            }
            cur_lpf.store(strength as u16, std::sync::atomic::Ordering::SeqCst);
        },
        FilterType::HPF => {
            set_eq(4, strength);
            if strength < 12 {
                set_eq(5, strength/2);
            } else {
                set_eq(5, strength);
            }
            cur_hpf.store(strength as u16, std::sync::atomic::Ordering::SeqCst);
        }
    }

    // cur_eq3.fetch_update(std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst, |cur_strength| {
    //     if cur_strength > (strength) as u16 {
    //         Some(strength as u16)
    //     } else {
    //         Some(cur_strength)
    //     }
    // }).expect("failed to set eq3 strength");

    // let eq3: i8 = cur_eq3.load(std::sync::atomic::Ordering::SeqCst).try_into().unwrap(); 
    // set_eq(3, eq3);
}

pub fn calibration(tof: Vl53l1x) {
    println!("Ensure TOF sensor is clear and press ENTER to preform SPAD calibration");
    io::stdin().read_line(&mut [0u8]).expect("Failed to read line"); 
    tof.preform_ref_spad_managment().expect("failed SPAD calibration!");
    
    println!("Ensure calibration card is 600mm from sensor and press ENTER to preform offset calibration");
    io::stdin().read_line(&mut [0u8]).expect("Failed to read line"); 
    tof.perform_single_target_xtalk_calibration(140).expect("failed cross-talk calibration!");
    
    println!("Ensure calibration card is 140mm from sensor and press ENTER to preform cross-talk calibration");
    io::stdin().read_line(&mut [0u8]).expect("Failed to read line"); 
    tof.perform_offset_simple_calibration(600).expect("failed offset calibration!");
    
    let mut cal_data: CalibrationData = CalibrationData::new();
    io::stdin().read_line(&mut [0u8]).expect("Failed to read line"); 
    tof.get_calibration_data(&mut cal_data);

    let ron_calib = ron::to_string(&cal_data).expect("failed to serialize calibration data!");
    fs::write("calibration.ron", ron_calib);
}