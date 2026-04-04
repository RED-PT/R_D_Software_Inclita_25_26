use core::fmt::Write;
use defmt::{error, info};
use embedded_sdmmc::Mode;
use heapless::String;

pub fn open_file_with_retry<
    D,
    T,
    const NB_DIRS: usize,
    const NB_FILES: usize,
    const NB_VOLUMES: usize,
>(
    root_dir: &mut embedded_sdmmc::Directory<'_, D, T, NB_DIRS, NB_FILES, NB_VOLUMES>,
    filename: &str,
    mode: Mode,
) -> Result<(), ()>
where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
{
    let mut file_num = 1;
    let mut full_filename: String<32> = String::new();
    loop {
        full_filename.clear();
        // Now works because core::fmt::Write is in scope
        if write!(full_filename, "{}{}", filename, file_num).is_err() {
            return Err(());
        }

        match root_dir.open_file_in_dir(full_filename.as_str(), mode) {
            Ok(_) => return Ok(()),
            Err(embedded_sdmmc::Error::NotFound) => {
                info!("Found empty slot! Logging to: {}", full_filename.as_str());
                break;
            }
            Err(e) => {
                error!("Error opening file: {:?}", defmt::Debug2Format(&e));
                file_num += 1;
                if file_num > 100 {
                    break;
                } // Safety break
            }
        }
    }

    Err(())
}

pub fn log_error<T>(err: &T, message: &str)
where
    T: core::fmt::Debug,
{
    error!("{}, Error: {:?}", message, defmt::Debug2Format(err));
}
