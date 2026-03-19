use defmt::{error, info, println};
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::Spi;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::Timestamp;
use embedded_sdmmc::{Error, Mode, SdCard, SdCardError, TimeSource, VolumeIdx, VolumeManager};

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
    let root_dir = volume0.open_root_dir().unwrap();
    let mut my_file = root_dir
        .open_file_in_dir("MY_FILE.TXT", Mode::ReadOnly)
        .unwrap();
    while !my_file.is_eof() {
        let mut buffer = [0u8; 32];
        let num_read = my_file.read(&mut buffer).unwrap();
    }

    // Read block 0 (the Master Boot Record
}
