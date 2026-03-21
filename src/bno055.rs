use crate::mock_data::{DATA_CHANNEL, ImuData, LogEvent};
use bno055::{BNO055OperationMode, Bno055};
use defmt::{Debug2Format, error, info};
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Blocking;
use embassy_time::{Delay, Duration, Instant, Ticker, Timer};

#[embassy_executor::task]
pub async fn bno055_logger_task(i2c_bus: I2c<'static, Blocking, embassy_stm32::i2c::Master>) {
    info!("Starting BNO055 IMU Task...");

    // The BNO055 crate needs a standard embedded-hal Delay to wait during bootup
    let mut delay = Delay;
    let mut imu = Bno055::new(i2c_bus);
    let mut ticker = Ticker::every(Duration::from_millis(100)); //ticker that fires at a 100Hz freq

    //Boot up the sensor
    if let Err(e) = imu.init(&mut delay) {
        error!("Failed to initialize BNO055! {:?}", Debug2Format(&e));
        loop {
            Timer::after_secs(1).await;
        }
    }

    //Set to NDOF mode (Sensor Fusion ON)
    if let Err(e) = imu.set_mode(BNO055OperationMode::NDOF, &mut delay) {
        error!("Failed to set BNO055 mode! {:?}", Debug2Format(&e));
        loop {
            Timer::after_secs(1).await;
        }
    }

    info!("BNO055 initialized and fused! Starting data loop (100Hz)...");

    let mut timestamp_ms: u32 = 0;

    loop {
        timestamp_ms = Instant::now().as_millis() as u32;

        // (If any read fails, we just use 0.0 or the last known value to keep logging alive
        let (yaw, pitch, roll) = match imu.euler_angles() {
            Ok(euler) => (euler.c, euler.a, euler.b), //TODO:check the AXIS
            Err(_) => (0.0, 0.0, 0.0),
        };
        let lin_accel = imu.linear_acceleration().unwrap_or(bno055::mint::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let gyro = imu.gyro_data().unwrap_or(bno055::mint::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        let mag = imu.mag_data().unwrap_or(bno055::mint::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });

        let data = ImuData {
            yaw,
            pitch,
            roll,

            temperature: imu.temperature().unwrap_or(0) as f32,

            mag_x: mag.x,
            mag_y: mag.y,
            mag_z: mag.z,

            gyro_x: gyro.x,
            gyro_y: gyro.y,
            gyro_z: gyro.z,

            lin_accel_n: lin_accel.x,
            lin_accel_e: lin_accel.y,
            lin_accel_d: lin_accel.z,

            timestamp_ms,
        };
        info!("{:?}", data);
        // Send to the SD Card
        DATA_CHANNEL.send(LogEvent::Imu(data)).await;

        // Yield back to Embassy
        ticker.next().await;
    }
}
