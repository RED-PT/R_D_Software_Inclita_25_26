//! Drivers for IIS"MDC magnetometer
//! Libs used
use crate::telemetry::{DATA_CHANNEL, LogEvent, MagnetometerData};
use defmt::{error, info};
use embassy_stm32::i2c::{Error as I2cError, I2c};
use embassy_stm32::mode::Blocking;
use embassy_time::{Delay, Duration, Instant, Ticker};
use iis2mdc::Iis2mdc;

#[embassy_executor::task]
pub async fn iis2mdc_logger_task(mut i2c_bus: I2c<'static, Blocking, embassy_stm32::i2c::Master>) {
    info!("Starting II2MDC magnetometer Task...");

    let mut mag = match Iis2mdc::new(&mut i2c_bus) {
        Ok(s) => s,
        Err(_) => {
            error!("Failed to initialize IIS2MDC! Check wiring or I2C address.");
            // Spin forever to avoid crashing the rest of the system
            loop {
                embassy_time::Timer::after_secs(1).await;
            }
        }
    };
    let mut ticker = Ticker::every(Duration::from_millis(100)); //ticker that fires at a 100Hz freq
    // 2. Configure the Sensor
    // Reboot the memory to clear any garbage state
    let _ = mag.cfg_reg_a.set_reboot(&mut i2c_bus, true);

    // Set Output Data Rate (ODR) to 100Hz to match your IMU
    let _ = mag
        .cfg_reg_a
        .set_data_rate(&mut i2c_bus, iis2mdc::cfg_reg_a::Odr::Hz100);

    // Set to Continuous Measurement Mode
    let _ = mag
        .cfg_reg_a
        .set_mode(&mut i2c_bus, iis2mdc::cfg_reg_a::Mode::Continuous);

    info!("Iis2mdc magnetometer configured!");

    loop {
        //Yield back to Embassy
        ticker.next().await;
        match mag.get_measurements(&mut i2c_bus) {
            Ok(reading) => {
                let data = MagnetometerData {
                    mag_x: reading[0] as f32,
                    mag_y: reading[1] as f32,
                    mag_z: reading[2] as f32,
                    timestamp_ms: Instant::now().as_millis() as u32,
                };
                // Send to the SD card
                DATA_CHANNEL.send(LogEvent::Mag(data)).await;
            }
            Err(e) => {
                match e {
                    I2cError::Nack => {
                        // The sensor didn't respond to its address.
                        // loose wire, unpowered sensor, or wrong address.
                        error!("IIS2MDC Critical: NACK! Sensor disconnected?");
                    }
                    I2cError::Timeout => {
                        // The STM32 gave up waiting. The I2C bus lines might be shorted to ground.
                        error!("IIS2MDC Warning: I2C Bus Timeout!");
                    }
                    I2cError::Overrun => {
                        // The MCU couldn't read the data fast enough
                        error!("IIS2MDC Warning: I2C Data Overrun!");
                    }
                    // Catch any other hardware errors (Bus error, Arbitration loss, etc)
                    other => {
                        error!("IIS2MDC Unknown I2C Hardware Error: {}", other);
                    }
                }
            }
        }
    }
}
