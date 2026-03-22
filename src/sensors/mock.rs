use crate::telemetry::{AltimeterData, DATA_CHANNEL, ImuData, LogEvent};
use defmt::info;
use embassy_time::{Duration, Instant, Ticker};

#[embassy_executor::task]
pub async fn mock_imu_task() {
    info!("Starting MOCK IMU Task (100Hz)...");
    let mut ticker = Ticker::every(Duration::from_millis(10));

    loop {
        ticker.next().await;

        let data = ImuData {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            temperature: 25.0,
            mag_x: 0.0,
            mag_y: 0.0,
            mag_z: 0.0,
            gyro_x: 0.0,
            gyro_y: 0.0,
            gyro_z: 0.0,
            lin_accel_n: 0.0,
            lin_accel_e: 0.0,
            lin_accel_d: 0.0,
            timestamp_ms: Instant::now().as_millis() as u32,
        };

        DATA_CHANNEL.send(LogEvent::Imu(data)).await;
    }
}

#[embassy_executor::task]
pub async fn mock_baro_task() {
    info!("Starting MOCK Altimeter Task (50Hz)...");
    let mut ticker = Ticker::every(Duration::from_millis(20));
    let pressure = 1013.25;

    loop {
        ticker.next().await;

        let data = AltimeterData {
            pressure,
            altitude: 100.0,
            temperature: 22.5,
            timestamp_ms: Instant::now().as_millis() as u32,
        };

        DATA_CHANNEL.send(LogEvent::Baro(data)).await;
    }
}
