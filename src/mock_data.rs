use defmt::Format;
use defmt::info;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;

use serde::Serialize;
//Mock data struct
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

//The Channel (Our FreeRTOS StreamBuffer equivalent)
// the channel can hold up to 25 readings before the mock feeder has to wait.
pub static DATA_CHANNEL: Channel<ThreadModeRawMutex, SensorData, 25> = Channel::new();

#[embassy_executor::task]
pub async fn mock_sensor_task() {
    info!("Starting Mock Sensor Task (100Hz)");
    let mut timestamp = 0;

    loop {
        timestamp += 10; // 10ms = 100Hz

        // Generate the frame
        let data = SensorData {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            temperature: 0.0,
            pressure: 0.0,
            mag_x: 0.0,
            mag_y: 0.0,
            mag_z: 0.0,
            accel_x: 0.0,
            accel_y: 0.0,
            accel_z: 0.0,
            gyro_x: 0.0,
            gyro_y: 0.0,
            gyro_z: 0.0,
            lin_accel_n: 0.0,
            lin_accel_e: 0.0,
            lin_accel_d: 0.0,
            lat: 0.0,
            lon: 0.0,

            timestamp_ms: timestamp,
            // Belas!
        };

        // Push to the channel
        DATA_CHANNEL.send(data).await;
        info!("pushed to channel");

        // Sleep until the next 10ms tick
        embassy_time::Timer::after_millis(10).await;
    }
}
