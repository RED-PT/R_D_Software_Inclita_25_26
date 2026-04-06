use crate::storage::sd_card_utils;
use crate::telemetry::data::{DATA_CHANNEL, LogEvent};
use core::fmt::Write;
use defmt::{error, info};
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::Spi;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use heapless::{String, Vec};

struct DummyTimesource;
impl TimeSource for DummyTimesource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 200,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

type MyVolumeManager<D, T> = VolumeManager<D, T, 4, 4, 6>;

#[embassy_executor::task]
pub async fn sd_logger_task(
    spi_bus: Spi<'static, Blocking, embassy_stm32::spi::mode::Master>,
    cs_pin: Output<'static>,
) {
    info!("Initializing Binary SD Card Logger...");
    let spi_device = ExclusiveDevice::new(spi_bus, cs_pin, Delay).unwrap();
    let sdcard = SdCard::new(spi_device, Delay);

    // Initialize the card and open volume
    match sdcard.num_bytes() {
        Ok(size) => info!("Card size is {} bytes", size),
        Err(_) => {
            // Using your updated util logger
            error!("Failed to talk to SD Card. Check wiring!");
            loop {
                embassy_time::Timer::after_secs(1).await;
            }
        }
    }

    let volume_mgr = MyVolumeManager::new_with_limits(sdcard, DummyTimesource, 1);
    let volume0 = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    let mut root_dir = volume0.open_root_dir().unwrap();

    // 1. Find the next available run/session number
    let run_num = match sd_card_utils::find_next_available_slot(&mut root_dir, "IMU") {
        Ok(num) => num,
        Err(_) => {
            error!("No empty file slots available! (1 to 100 all full). Please format SD Card.");
            // Spin forever safely
            loop {
                embassy_time::Timer::after_secs(1).await;
            }
        }
    };

    info!("Starting logging session #{}", run_num);

    // 2. Generate unified filenames for this session
    let mut imu_filename: String<32> = String::new();
    let mut baro_filename: String<32> = String::new();
    let mut gps_filename: String<32> = String::new();
    let mut mag_filename: String<32> = String::new();

    write!(imu_filename, "IMU_{}", run_num).unwrap();
    write!(baro_filename, "BARO_{}", run_num).unwrap();
    write!(gps_filename, "GPS_{}", run_num).unwrap();
    write!(mag_filename, "MAG_{}", run_num).unwrap();

    // This allows us to hold several readings before bothering the SD card.
    let mut imu_buf: Vec<u8, 512> = Vec::new();
    let mut baro_buf: Vec<u8, 512> = Vec::new();
    let mut gps_buf: Vec<u8, 512> = Vec::new();
    let mut mag_buf: Vec<u8, 512> = Vec::new();

    // A small temporary array just for serializing a single event
    let mut temp_encode_buf = [0u8; 64];

    info!("Starting Multi-File Buffered Data Logging!");

    // 4. The Macro that handles Buffering -> Opening -> Writing -> Closing
    macro_rules! write_buffered {
        ($data:expr, $buffer:expr, $filename:expr) => {
            // Serialize the struct into bytes
            if let Ok(encoded_bytes) = postcard::to_slice(&$data, &mut temp_encode_buf) {

                // If adding these new bytes would overflow our RAM buffer, it's time to flush to the SD card!
                if $buffer.len() + encoded_bytes.len() > $buffer.capacity() {

                    // 1. OPEN the specific file safely
                    match sd_card_utils::open_file_with_retry(&mut root_dir, $filename.as_str(), Mode::ReadWriteCreateOrAppend) {
                        Ok(mut file) => {
                            // 2. WRITE the whole chunk at once
                            if let Err(e) = file.write(&$buffer) {
                                error!("Write Failed on {}: {:?}", $filename.as_str(), defmt::Debug2Format(&e));
                            }
                            // 3. CLOSE the file (This releases the mutable borrow on root_dir!)
                            file.close().unwrap_or_default();
                        }
                        Err(e) => error!("Failed to open {} for flushing: {:?}", $filename.as_str(), defmt::Debug2Format(&e)),
                    }

                    // Clear the buffer after writing
                    $buffer.clear();
                }

                // Append the new bytes to the buffer
                let _ = $buffer.extend_from_slice(encoded_bytes);
            }
        };
    }

    // 5. The Event Loop
    loop {
        let frame: LogEvent = DATA_CHANNEL.receive().await;

        match frame {
            LogEvent::Mag(mag_data) => {
                write_buffered!(mag_data, mag_buf, mag_filename);
            }
            LogEvent::GPS(gps_data) => {
                write_buffered!(gps_data, gps_buf, gps_filename);
            }
            LogEvent::Imu(imu_data) => {
                write_buffered!(imu_data, imu_buf, imu_filename);
            }
            LogEvent::Baro(baro_data) => {
                write_buffered!(baro_data, baro_buf, baro_filename);
            }
        }
    }
}
