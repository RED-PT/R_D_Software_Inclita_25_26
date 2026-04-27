use crate::telemetry::data::{AccelData, DATA_CHANNEL, LATEST_TELEMETRY, LogEvent};
use defmt::{error, info};
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Blocking;
use embassy_time::{Duration, Ticker};
use lh_adxl345 as adxl; // Brings in the .accel_raw() method

#[embassy_executor::task]
pub async fn adxl343_task(i2c: I2c<'static, Blocking, embassy_stm32::i2c::Master>) {
    info!("Initializing ADXL345 via lh-adxl345 crate at 0x1D...");

    // 1. Wrap the Embassy I2C bus in the crate's I2C struct
    // This is where we solve the SDO pull-up address issue!
    let adxl_bus = adxl::AdxlBusI2c {
        i2c,
        addr: 0x1D, // Your custom address because SDO is pulled HIGH
    };

    // 2. Initialize the device
    let mut accelerometer = adxl::Adxl345::new(adxl_bus);

    // 3. Write default configurations (Measurement mode, standard ranges, etc.)
    if let Err(_) = accelerometer.init_defaults() {
        error!("Failed to initialize lh-adxl345 crate! Check wiring.");
        return;
    }

    // Set task to run at 20Hz (every 50ms)
    let mut ticker = Ticker::every(Duration::from_millis(50));

    loop {
        // 4. Read the raw acceleration via the accelerometer trait
        // This blocks the CPU just long enough to pull the 6 bytes over I2C.
        match accelerometer.read_axis() {
            Ok((x, y, z)) => {
                let accel_data = AccelData {
                    raw_x: x,
                    raw_y: y,
                    raw_z: z,
                    timestamp_ms: embassy_time::Instant::now().as_millis() as u32,
                };

                // Update the global telemetry Mutex
                {
                    let mut guard = LATEST_TELEMETRY.lock().await;
                    //guard.accel = Some(accel_data.clone());
                }

                // Send to the SD Card logger
                // let _ = DATA_CHANNEL.send(LogEvent::Accel(accel_data)).await;
            }
            Err(_) => error!("Failed to read axis from lh-adxl345 crate"),
        }

        // 7. Yield back to the Embassy executor
        ticker.next().await;
    }
}
