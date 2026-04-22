use crate::telemetry::data::{AccelData, DATA_CHANNEL, LATEST_TELEMETRY, LogEvent};
use defmt::{error, info};
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_time::{Duration, Ticker};

// Default ADXL345 I2C address (0x53 if SDO/ALT ADDRESS pin is grounded)
const ADXL345_ADDR: u8 = 0x53;

#[embassy_executor::task]
pub async fn adxl345_task(mut i2c: I2c<'static, Async, embassy_stm32::i2c::Master>) {
    info!("Initializing ADXL345 via I2C...");

    // 1. Wake up the sensor!
    // Write 0x08 to the POWER_CTL register (0x2D) to put it into Measurement Mode.
    if i2c.write(ADXL345_ADDR, &[0x2D, 0x08]).await.is_err() {
        error!("Failed to init ADXL345. Check I2C wiring and SDO pin state.");
        return; // Cleanly exit task if sensor is completely unresponsive
    }

    // Set task to run at 20Hz (every 50ms)
    let mut ticker = Ticker::every(Duration::from_millis(50));
    let mut data_buf = [0u8; 6];

    loop {
        // 2. Atomic Burst Read
        // We write the starting address (0x32), and the sensor automatically increments
        // the register internally to give us all 6 bytes (X, Y, Z axes) in one go.
        match i2c.write_read(ADXL345_ADDR, &[0x32], &mut data_buf).await {
            Ok(_) => {
                // 3. Assemble the Little-Endian bytes into raw i16 integers
                let raw_x = i16::from_le_bytes([data_buf[0], data_buf[1]]);
                let raw_y = i16::from_le_bytes([data_buf[2], data_buf[3]]);
                let raw_z = i16::from_le_bytes([data_buf[4], data_buf[5]]);

                let accel_data = AccelData {
                    raw_x,
                    raw_y,
                    raw_z,
                    timestamp_ms: embassy_time::Instant::now().as_millis() as u32,
                };

                // 5. Send to SD Card via the channel
                let _ = DATA_CHANNEL.send(LogEvent::ACCEL(accel_data)).await;
            }
            Err(_) => error!("ADXL345 read timeout or NACK!"),
        }

        ticker.next().await;
    }
}
