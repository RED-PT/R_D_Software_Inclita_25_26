use crate::storage::sd_card_utils;
use crate::telemetry::{DATA_CHANNEL, LogEvent};
use core::fmt::Write;
use defmt::{info, println};
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::Spi;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use heapless::String;

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
            sd_card_utils::log_error(&(), "Failed to talk to SD Card. Check wiring!");
            loop {
                embassy_time::Timer::after_secs(1).await;
            }
        }
    }

    let volume_mgr = MyVolumeManager::new_with_limits(sdcard, DummyTimesource, 1);
    let mut volume0 = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    let mut root_dir = volume0.open_root_dir().unwrap();

    // 1. Find the next available run/session number
    let mut run_num = 1;
    let mut temp_filename: String<32> = String::new();
    loop {
        temp_filename.clear();
        write!(temp_filename, "IMU_{}", run_num).unwrap();
        // Just checking if the IMU file for this session exists
        match root_dir.open_file_in_dir(temp_filename.as_str(), Mode::ReadOnly) {
            Ok(file) => {
                // File exists, bump the number and close it
                file.close().unwrap_or_default();
                run_num += 1;
            }
            Err(embedded_sdmmc::Error::NotFound) => break, // Found an empty slot!
            Err(_) => {
                run_num += 1;
                if run_num > 100 {
                    break;
                } // Safety escape
            }
        }
    }

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

    // 3. Open the files
    let mut imu_file = root_dir
        .open_file_in_dir(imu_filename.as_str(), Mode::ReadWriteCreateOrAppend)
        .unwrap();
    let mut baro_file = root_dir
        .open_file_in_dir(baro_filename.as_str(), Mode::ReadWriteCreateOrAppend)
        .unwrap();
    let mut gps_file = root_dir
        .open_file_in_dir(gps_filename.as_str(), Mode::ReadWriteCreateOrAppend)
        .unwrap();
    let mut mag_file = root_dir
        .open_file_in_dir(mag_filename.as_str(), Mode::ReadWriteCreateOrAppend)
        .unwrap();

    info!("Starting Binary Data Logging!");

    const BURST_SIZE: u32 = 30;
    let mut imu_burst_counter = 0;
    let mut baro_burst_counter = 0;
    let mut gps_burst_counter = 0;
    let mut mag_burst_counter = 0;
    let mut bin_buffer = [0u8; 128];

    // DRY MACRO: Handles the repetitive serialization, writing, and burst-refresh logic
    macro_rules! write_sensor_data {
        ($data:expr, $file:expr, $filename:expr, $counter:expr, $err_msg:expr) => {
            if let Ok(bytes) = postcard::to_slice(&$data, &mut bin_buffer) {
                if let Err(_e) = $file.write(bytes) {
                    sd_card_utils::log_error(&_e, $err_msg);
                }
            }
            $counter += 1;

            if $counter >= BURST_SIZE {
                $file.close().unwrap_or_default();
                // Re-open to flush FAT table updates to the SD card
                $file = root_dir
                    .open_file_in_dir($filename.as_str(), Mode::ReadWriteCreateOrAppend)
                    .unwrap();
                $counter = 0;
            }
        };
    }

    loop {
        // WAIT FOR DATA
        let frame: LogEvent = DATA_CHANNEL.receive().await;

        match frame {
            LogEvent::Mag(mag_data) => {
                write_sensor_data!(
                    mag_data,
                    mag_file,
                    mag_filename,
                    mag_burst_counter,
                    "Mag Write Failed"
                );
            }
            LogEvent::GPS(gps_data) => {
                write_sensor_data!(
                    gps_data,
                    gps_file,
                    gps_filename,
                    gps_burst_counter,
                    "GPS Write Failed"
                );
            }
            LogEvent::Imu(imu_data) => {
                write_sensor_data!(
                    imu_data,
                    imu_file,
                    imu_filename,
                    imu_burst_counter,
                    "IMU Write Failed"
                );
            }
            LogEvent::Baro(baro_data) => {
                write_sensor_data!(
                    baro_data,
                    baro_file,
                    baro_filename,
                    baro_burst_counter,
                    "Baro Write Failed"
                );
            }
        }
    }
}
