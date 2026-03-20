use crate::mock_data::{DATA_CHANNEL, SensorData};
use core::fmt::Write;
use defmt::{Debug2Format, error, info, println};
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::Spi;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use heapless::String;
// A dummy time source required by embedded-sdmmc
struct DummyTimesource;
impl TimeSource for DummyTimesource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

#[embassy_executor::task]
pub async fn test_raw_read(
    spi_bus: Spi<'static, Blocking, embassy_stm32::spi::mode::Master>,
    cs_pin: Output<'static>,
) {
    info!("Starting SD SPI initialization");

    // Initialize the SD card driver
    // Delay wrapper is sometimes needed, but for basic embedded-hal blocking SPI,
    // we can pass the SPI bus, the CS pin, and a standard delay.
    // Embassy provides a blocking delay via embassy_time::Delay
    let spi_device = ExclusiveDevice::new(spi_bus, cs_pin, Delay).unwrap();

    let sdcard = SdCard::new(spi_device, Delay);

    // Try to get the card size to prove we are talking to it
    info!("Attempting to initialize card...");
    match sdcard.num_bytes() {
        Ok(size) => info!("Card size is {} bytes", size),
        Err(_) => {
            error!("Failed to talk to SD Card. Check wiring!");
            loop {
                embassy_time::Timer::after_secs(1).await;
            } // Park safely
        }
    }

    //By default the VolumeManager will initialize with a maximum number of 4 open directories, files and volumes.
    //This can be customized by specifying the MAX_DIR, MAX_FILES and MAX_VOLUMES generic consts of the VolumeManager:

    //TODO: change this to new_with_limits
    let volume_mgr = VolumeManager::new(sdcard, DummyTimesource);
    let volume0 = match volume_mgr.open_volume(VolumeIdx(0)) {
        Ok(v) => v,
        Err(_) => {
            error!("Failed to open Volume 0. Is it formatted to FAT16/FAT32?");
            loop {
                embassy_time::Timer::after_secs(1).await;
            }
        }
    };
    println!("Volume 0 opened!"); //println!("Volume 0: {:?}", volume0);

    //Read Test
    let root_dir = volume0.open_root_dir().unwrap();
    let my_file = root_dir
        .open_file_in_dir("MY_FILE.TXT", Mode::ReadOnly)
        .unwrap();
    while !my_file.is_eof() {
        let mut buffer = [0u8; 32];
        let _num_read = my_file.read(&mut buffer).unwrap();
    }

    //Write Test
    let my_other_file = root_dir
        .open_file_in_dir("MY_DATA.CSV", embedded_sdmmc::Mode::ReadWriteCreateOrAppend)
        .unwrap();
    my_other_file.write(b"Timestamp,Signal,Value\n").unwrap();
    my_other_file
        .write(b"2025-01-01T00:00:00Z,TEMP,25.0\n")
        .unwrap();
    my_other_file
        .write(b"2025-01-01T00:00:01Z,TEMP,25.1\n")
        .unwrap();
    my_other_file
        .write(b"2025-01-01T00:00:02Z,TEMP,25.2\n")
        .unwrap();

    // Don't forget to flush the file so that the directory entry is updated
    my_other_file.flush().unwrap();

    // Read block 0 (the Master Boot Record
}

type MyVolumeManager<D, T> = VolumeManager<D, T, 4, 4, 4>;

#[embassy_executor::task]
pub async fn sd_logger_task(
    spi_bus: Spi<'static, Blocking, embassy_stm32::spi::mode::Master>,
    cs_pin: Output<'static>,
) {
    info!("Initializing Binary SD Card Logger...");
    info!("alooo");
    let spi_device = ExclusiveDevice::new(spi_bus, cs_pin, Delay).unwrap();
    info!("alooo");

    let sdcard = SdCard::new(spi_device, Delay);
    info!("alooo");

    let volume_mgr = MyVolumeManager::new_with_limits(sdcard, DummyTimesource, 1);
    info!("alooo");

    let volume0 = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    let root_dir = volume0.open_root_dir().unwrap();

    // 1. AUTO-INCREMENTING FILE GENERATOR
    let mut file_num = 1;
    let mut filename: String<16> = String::new();

    loop {
        filename.clear();
        write!(filename, "DATA{}.BIN", file_num).unwrap();

        match root_dir.open_file_in_dir(filename.as_str(), Mode::ReadOnly) {
            Ok(existing_file) => {
                existing_file.close().unwrap_or_default();
                file_num += 1;
            }
            Err(embedded_sdmmc::Error::NotFound) => {
                info!("Found empty slot! Logging to: {}", filename.as_str());
                break;
            }
            Err(e) => {
                error!("Error: {:?}", Debug2Format(&e));
                loop {
                    embassy_time::Timer::after_secs(1).await;
                }
            }
        }
    }

    let mut data_file = root_dir
        .open_file_in_dir(filename.as_str(), Mode::ReadWriteCreateOrAppend)
        .unwrap();
    info!("Starting Binary Data Logging!");

    const BURST_SIZE: u32 = 20;
    let mut burst_counter = 0;

    let mut bin_buffer = [0u8; 128];

    loop {
        //WAIT FOR DATA
        let frame: SensorData = DATA_CHANNEL.receive().await;
        info!("data received from chanel");

        // postcard packs the `frame` directly into the `bin_buffer`
        match postcard::to_slice(&frame, &mut bin_buffer) {
            Ok(serialized_bytes) => {
                // serialized_bytes is exactly the length of the valid data
                if let Err(e) = data_file.write(serialized_bytes) {
                    error!("Write failed! {:?}", Debug2Format(&e));
                }
            }
            Err(_) => error!("Failed to serialize data!"),
        }

        burst_counter += 1;

        if burst_counter >= BURST_SIZE {
            data_file.close().unwrap_or_default();
            burst_counter = 0;
            data_file = root_dir
                .open_file_in_dir(filename.as_str(), Mode::ReadWriteCreateOrAppend)
                .unwrap();
        }
    }
}
