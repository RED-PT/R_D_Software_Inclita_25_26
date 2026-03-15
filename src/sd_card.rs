use defmt::{info, Debug2Format};
use embassy_stm32::sdmmc::sd::{CmdBlock, DataBlock, StorageDevice};
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::time::mhz;

const ALLOW_WRITES: bool = false;
//Initializes the SD card and reads block 0, to verify the hardware and DMA stuff
#[embassy_executor::task]
pub async fn test_raw_read(mut sdmmc: Sdmmc<'static>) {
    info!("Starting SD HW initialization");
    let mut cmd_block = CmdBlock::new();

    let mut storage = StorageDevice::new_sd_card(&mut sdmmc, &mut cmd_block, mhz(24))
        .await
        .unwrap();

    let card = storage.card();

    info!("Card: {:#?}", Debug2Format(&card));

    // Arbitrary block index
    let block_idx = 16;

    // SDMMC uses `DataBlock` instead of `&[u8]` to ensure 4 byte alignment required by the hardware.
    let mut block = DataBlock::new();

    storage.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);

    if !ALLOW_WRITES {
        info!("Writing is disabled.");
        panic!("Writing is disabled")
    }

    info!("Filling block with 0x55");
    block.fill(0x55);
    storage.write_block(block_idx, &block).await.unwrap();
    info!("Write done");

    storage.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);

    info!("Filling block with 0xAA");
    block.fill(0xAA);
    storage.write_block(block_idx, &block).await.unwrap();
    info!("Write done");

    storage.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);
}
