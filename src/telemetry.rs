use defmt::Format;
use defmt::info;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;

use serde::Serialize;
//Mock data struct
#[derive(Format, Serialize)]
pub struct ImuData {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub temperature: f32,
    pub mag_x: f32,
    pub mag_y: f32,
    pub mag_z: f32,
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
    pub lin_accel_n: f32,
    pub lin_accel_e: f32,
    pub lin_accel_d: f32,
    pub timestamp_ms: u32,
}
#[derive(Format, Serialize)]
pub struct AltimeterData {
    pub pressure: f32,
    pub altitude: f32,
    pub temperature: f32,
    pub timestamp_ms: u32,
}

#[derive(Format, Serialize)]
pub struct SensorData {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub temperature: f32,
    pub pressure: f32,
    pub mag_x: f32,
    pub mag_y: f32,
    pub mag_z: f32,
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
    pub lin_accel_n: f32,
    pub lin_accel_e: f32,
    pub lin_accel_d: f32,
    pub lat: f32,
    pub lon: f32,
    pub timestamp_ms: u32,
}

// The wrapping envelope
#[derive(Format, Serialize)]
pub enum LogEvent {
    Imu(ImuData),
    Baro(AltimeterData),
}
//The Channel (Our FreeRTOS StreamBuffer equivalent)
// the channel can hold up to 25 readings before the mock feeder has to wait.
pub static DATA_CHANNEL: Channel<ThreadModeRawMutex, LogEvent, 50> = Channel::new();
