use serde::{Deserialize, Serialize};
use vl53l1x::{Vl53l1x, Vl53l1xRangeStatus, CalibrationData, CustomerNvmManaged, AdditionalOffsetCalData, OpticalCentre, GainCalibrationData, CalPeakRateMap};
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
    io::stdin().read_line(&mut "".to_string()).expect("Failed to read line"); 
    tof.perform_ref_spad_management().expect("failed SPAD calibration!");
    
    println!("Ensure calibration card is 600mm from sensor and press ENTER to preform offset calibration");
    io::stdin().read_line(&mut "".to_string()).expect("Failed to read line"); 
    tof.perform_single_target_xtalk_calibration(140).expect("failed cross-talk calibration!");
    
    println!("Ensure calibration card is 140mm from sensor and press ENTER to preform cross-talk calibration");
    io::stdin().read_line(&mut "".to_string()).expect("Failed to read line"); 
    tof.perform_offset_simple_calibration(600).expect("failed offset calibration!");
    
    let mut cal_data: CalibrationData = CalibrationData::new();
    io::stdin().read_line(&mut "".to_string()).expect("Failed to read line"); 
    tof.get_calibration_data(&mut cal_data);

    let ron_calib = ron::to_string(&cal_data).expect("failed to serialize calibration data!");
    fs::write("calibration.ron", ron_calib);
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "CalibrationData")]
#[repr(C)]
pub struct CalibrationDataRem {
	struct_version: u32,
    #[serde(with = "CustomerNvmManagedRem")]
	customer: CustomerNvmManaged,
    #[serde(with = "AdditionalOffsetCalDataRem")]
    add_off_cal_data: AdditionalOffsetCalData,
    #[serde(with = "OpticalCentreRem")]
	optical_centre: OpticalCentre,
    #[serde(with = "GainCalibrationDataRem")]
	gain_cal: GainCalibrationData,
    #[serde(with = "CalPeakRateMapRem")]
	cal_peak_rate_map: CalPeakRateMap,

}

// impl CalibrationData {
//     pub fn new() -> Self {
//         Self {
//             struct_version: 0,
//             customer: CustomerNvmManaged {
//                global_config__spad_enables_ref_0: 0,
//                global_config__spad_enables_ref_1: 0,
//                global_config__spad_enables_ref_2: 0,
//                global_config__spad_enables_ref_3: 0,
//                global_config__spad_enables_ref_4: 0,
//                global_config__spad_enables_ref_5: 0,
//                global_config__ref_en_start_select: 0,
//                ref_spad_man__num_requested_ref_spads: 0,
//                ref_spad_man__ref_location: 0,
//                algo__crosstalk_compensation_plane_offset_kcps: 0,
//                algo__crosstalk_compensation_x_plane_gradient_kcps: 0,
//                algo__crosstalk_compensation_y_plane_gradient_kcps: 0,
//                ref_spad_char__total_rate_target_mcps: 0,
//                algo__part_to_part_range_offset_mm: 0,
//                mm_config__inner_offset_mm: 0,
//                mm_config__outer_offset_mm: 0,
//             },
//             add_off_cal_data: AdditionalOffsetCalData {
//                result__mm_inner_actual_effective_spads: 0,
//                result__mm_outer_actual_effective_spads: 0,
//                result__mm_inner_peak_signal_count_rtn_mcps: 0,
//                result__mm_outer_peak_signal_count_rtn_mcps: 0,
//             },
//             optical_centre: OpticalCentre {
//                 x_centre: 0,
//                 y_centre: 0,
//             },
//             gain_cal: GainCalibrationData {
//                 standard_ranging_gain_factor: 0,
//             },
//             cal_peak_rate_map: CalPeakRateMap {
//                cal_distance_mm: 0,
//                max_samples: 0,
//                width: 0,
//                height: 0,
//                peak_rate_mcps: [0; VL53L1_NVM_PEAK_RATE_MAP_SAMPLES],
//             },
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "CustomerNvmManaged")]
#[repr(C)]
pub struct CustomerNvmManagedRem {
    global_config__spad_enables_ref_0: u8,
    global_config__spad_enables_ref_1: u8,
    global_config__spad_enables_ref_2: u8,
    global_config__spad_enables_ref_3: u8,
    global_config__spad_enables_ref_4: u8,
    global_config__spad_enables_ref_5: u8,
    global_config__ref_en_start_select: u8,
    ref_spad_man__num_requested_ref_spads: u8,
    ref_spad_man__ref_location: u8,
    algo__crosstalk_compensation_plane_offset_kcps: u32,
    algo__crosstalk_compensation_x_plane_gradient_kcps: i16,
    algo__crosstalk_compensation_y_plane_gradient_kcps: i16,
    ref_spad_char__total_rate_target_mcps: u16,
    algo__part_to_part_range_offset_mm: i16,
    mm_config__inner_offset_mm: i16,
    mm_config__outer_offset_mm: i16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "AdditionalOffsetCalData")]
#[repr(C)]
struct AdditionalOffsetCalDataRem {
    result__mm_inner_actual_effective_spads: u16,
    result__mm_outer_actual_effective_spads: u16,
    result__mm_inner_peak_signal_count_rtn_mcps: u16,
    result__mm_outer_peak_signal_count_rtn_mcps: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "OpticalCentre")]
#[repr(C)]
struct OpticalCentreRem {
    x_centre: u8,
    y_centre: u8,
} 

#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "GainCalibrationData")]
#[repr(C)]
struct GainCalibrationDataRem {
	standard_ranging_gain_factor: u16,
}


const VL53L1_NVM_PEAK_RATE_MAP_SAMPLES: usize = 25;

#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "CalPeakRateMap")]
#[repr(C)]
struct CalPeakRateMapRem {
    cal_distance_mm: i16,
    max_samples: u16,
    width: u16,
    height: u16, 
    peak_rate_mcps: [u16; VL53L1_NVM_PEAK_RATE_MAP_SAMPLES],
}