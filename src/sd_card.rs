use defmt::{error, info, println};
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::Spi;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{
    Error, Mode, SdCard, SdCardError, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};

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

    let mut sdcard = SdCard::new(spi_device, Delay);

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
    let mut volume0 = match volume_mgr.open_volume(VolumeIdx(0)) {
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
    let mut my_file = root_dir
        .open_file_in_dir("MY_FILE.TXT", Mode::ReadOnly)
        .unwrap();
    while !my_file.is_eof() {
        let mut buffer = [0u8; 32];
        let num_read = my_file.read(&mut buffer).unwrap();
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
