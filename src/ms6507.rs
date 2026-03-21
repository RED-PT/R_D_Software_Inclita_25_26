use crate::mock_data::{AltimeterData, DATA_CHANNEL, LogEvent};

use defmt::{error, info};
use embassy_stm32::gpio::Output;
use embassy_stm32::spi::Spi;
use embassy_time::{Delay, Instant, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use ms5611_rs::{Ms5611, Oversampling};

#[embassy_executor::task]
pub async fn ms6507_task(
    // Notice we are demanding an Async SPI bus here, not Blocking!
    spi_bus: Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Master>,
    cs_pin: Output<'static>,
) {
    info!("Starting MS5607 Altimeter Task (50Hz)...");

    let spi_device = ExclusiveDevice::new(spi_bus, cs_pin, Delay).unwrap();
    let mut sensor = Ms5611::new_spi(spi_device);
    let mut delay = Delay;

    // Read the factory calibration PROM from the sensor
    if let Err(_) = sensor.init(&mut delay).await {
        error!("Failed to initialize MS5607!");
        loop {
            Timer::after_secs(1).await;
        } // Halt task on failure
    }

    info!("MS5607 Calibrated! Starting conversion loop...");

    loop {
        // Osr4096 is max resolution. The .await here allows Embassy to pause
        // THIS task for 9ms and go run the BNO055, Zero wasted CPU.
        match sensor.measure(Oversampling::Osr4096, &mut delay).await {
            Ok(measurement) => {
                let data = AltimeterData {
                    timestamp_ms: Instant::now().as_millis() as u32,
                    pressure: measurement.pressure_mbar,
                    temperature: measurement.temperature_c,
                    altitude: 0.0,
                };
                info!("{:?}", data);
                // Wrap it and send it!
                DATA_CHANNEL.send(LogEvent::Baro(data)).await;
            }
            Err(_) => error!("MS5607 read failed!"),
        }
        // Run at 50Hz (every 20ms)
        Timer::after_millis(20).await;
    }
}
