use core::fmt::Write;
use defmt::{error, info, warn};
use embedded_sdmmc::{BlockDevice, Error as SdError, File, Mode};
use heapless::String;

#[derive(Debug)]
pub struct MaxAttemptsReached;

pub fn find_next_available_slot<
    D,
    T,
    const NB_DIRS: usize,
    const NB_FILES: usize,
    const NB_VOLUMES: usize,
>(
    root_dir: &mut embedded_sdmmc::Directory<'_, D, T, NB_DIRS, NB_FILES, NB_VOLUMES>,
    filename: &str,
) -> Result<u8, MaxAttemptsReached>
where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
    <D as BlockDevice>::Error: defmt::Format,
{
    let mut full_filename: String<32> = String::new();
    for file_num in 1..=100 {
        full_filename.clear();
        if write!(full_filename, "{}{}", filename, file_num).is_err() {
            continue;
        }

        match root_dir.open_file_in_dir(full_filename.as_str(), Mode::ReadOnly) {
            Ok(file) => {
                // The file already exists. Close it and let the loop check the next number.
                file.close().unwrap_or_default();
            }
            Err(embedded_sdmmc::Error::NotFound) => {
                info!("Found empty slot! Logging to: {}", full_filename.as_str()); //We found a
                //file num that hasnt been used yet
                return Ok(file_num); // Return the number wrapped in Ok()
            }
            Err(e) => {
                error!("Error opening file: {:?}", e);
            }
        }
    }
    // If we exit the loop, it means files 1 through 100 ALL existed
    // throw the error
    Err(MaxAttemptsReached)
}

pub fn open_file_with_retry<
    'dir,
    D,
    T,
    const NB_DIRS: usize,
    const NB_FILES: usize,
    const NB_VOLUMES: usize,
>(
    root_dir: &'dir mut embedded_sdmmc::Directory<'_, D, T, NB_DIRS, NB_FILES, NB_VOLUMES>,
    filename: &str,
    mode: Mode,
) -> Result<File<'dir, D, T, NB_DIRS, NB_FILES, NB_VOLUMES>, SdError<D::Error>>
where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
{
    const MAX_RETRIES: u8 = 5;

    // will always hit one of return statements inside.
    for attempt in 1..=MAX_RETRIES {
        match root_dir.open_file_in_dir(filename, mode) {
            Ok(file) => {
                return Ok(file); // Success
            }
            Err(e) => {
                if attempt >= MAX_RETRIES {
                    // Exhausted all retries, return the actual SD card error
                    return Err(e);
                }
                warn!(
                    "Failed to open {} (Attempt {}/{}). Retrying...",
                    filename, attempt, MAX_RETRIES
                );
            }
        }
    }
    unreachable!();
}
