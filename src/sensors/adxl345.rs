use crate::telemetry::data::{AccelData, DATA_CHANNEL, LATEST_TELEMETRY, LogEvent};
use adxl343::Adxl343;
use adxl343::accelerometer::RawAccelerometer; // Required trait to access .accel_raw()
use defmt::{error, info};
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Blocking;
use embassy_time::{Duration, Ticker};

#[embassy_executor::task]
pub async fn adxl343_task(i2c: I2c<'static, Blocking, embassy_stm32::i2c::Master>) {
    info!("Initializing ADXL343 via adxl343 crate...");

    // 1. Pass the blocking I2C bus into the crate
    let mut accelerometer = match Adxl343::new(i2c) {
        Ok(accel) => accel,
        Err(_) => {
            error!("Failed to initialize ADXL343 crate!");
            return;
        }
    };

    // Set task to run at 20Hz (every 50ms)
    let mut ticker = Ticker::every(Duration::from_millis(50));

    loop {
        // 2. Read the raw acceleration
        // This is a BLOCKING call. The CPU will wait right here until the I2C
        // transaction is finished, but Embassy handles this gracefully.
        match accelerometer.accel_raw() {
            Ok(accel_vec) => {
                // The crate returns a vector struct with x, y, z fields.
                // We keep them as raw i16 integers for maximum LoRa efficiency!
                let accel_data = AccelData {
                    raw_x: accel_vec.x,
                    raw_y: accel_vec.y,
                    raw_z: accel_vec.z,
                    timestamp_ms: embassy_time::Instant::now().as_millis() as u32,
                };
                info!("{:?}", accel_data);

                // 3. Update global telemetry mutex
                {
                    let mut guard = LATEST_TELEMETRY.lock().await;
                    //guard.accel = Some(accel_data.clone());
                }

                // 4. Send to SD Card via the channel
                //let _ = DATA_CHANNEL.send(LogEvent::Accel(accel_data)).await;
            }
            Err(_) => error!("Failed to read from ADXL343 crate"),
        }

        // 5. Yield back to the executor so your LoRa and GPS tasks can run!
        ticker.next().await;
    }
}
