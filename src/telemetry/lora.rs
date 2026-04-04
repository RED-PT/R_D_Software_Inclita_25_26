use crate::telemetry::data;
use defmt::{error, info};
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::Spi;
use embassy_time::{Delay, Duration, Ticker};
use sx127x_lora::LoRa;
#[embassy_executor::task]

pub async fn lora_task(
    spi: Spi<'static, Blocking, embassy_stm32::spi::mode::Master>,
    cs: Output<'static>,
    reset: Output<'static>,
) {
    info!("Initializing RFM95 LoRa Module...");
    const FREQUENCY: i64 = 868;

    let mut lora = LoRa::new(spi, cs, reset, FREQUENCY, Delay).unwrap();

    // 1. Set Maximum Bandwidth (500 kHz)
    // Valid options for RFM95: 125_000, 250_000, or 500_000 Hz.
    let _ = lora.set_signal_bandwidth(500_000);

    // 2. Set Lowest Spreading Factor (SF7)
    // Default is often SF9 or SF10. SF7 is the fastest standard LoRa setting.
    // Every step down roughly halves your Time on Air!
    let _ = lora.set_spreading_factor(7);

    // 3. Set the lowest Coding Rate (4/5)
    // This is the error-correction overhead. 4/5 means for every 4 bits of data,
    // 1 parity bit is sent. It's the lowest overhead setting.
    let _ = lora.set_coding_rate_4(5);

    // 17 dBm - max power lol
    let _ = lora.set_tx_power(17, 1);

    let mut tx_buffer = [0u8; 255];

    // Create a Ticker that fires exactly 5 times per second (every 200ms)
    let mut ticker = Ticker::every(Duration::from_hz(5));

    loop {
        // 1. Grab a snapshot of the latest data instantly
        let packet = data::LATEST_TELEMETRY.lock(|t| t.borrow().clone());

        // 2. Serialize to bytes
        if let Ok(payload_len) = postcard::to_slice(&packet, &mut tx_buffer).map(|b| b.len()) {
            // 3. Transmit the packet!
            match lora.transmit_payload(tx_buffer, payload_len) {
                Ok(size) => info!("Transmitted {} bytes at 5Hz", size),
                Err(_) => error!("LoRa Transmission Failed!"),
            }
        }

        // 5. Wait for the exact remainder of the 200ms window
        ticker.next().await;
    }
}
