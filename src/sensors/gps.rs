use crate::telemetry::{DATA_CHANNEL, GnggaMessage, GpsFix, LogEvent, UtcTime};
use defmt::{error, info, warn};
use embassy_stm32::usart::Uart;
use embassy_time::Instant;

#[embassy_executor::task]
pub async fn gps_task(mut uart: Uart<'static, embassy_stm32::mode::Async>) {
    // (Usually ~300-500 bytes
    let mut dma_buf = [0u8; 400];

    loop {
        // The CPU halts this task entirely. DMA fills the buffer.
        // It ONLY wakes up when the GPS finishes its sentence burst!
        match uart.read_until_idle(&mut dma_buf).await {
            Ok(bytes_read) => {
                //PROCESS THE COMPLETE BURST
                if let Ok(burst_str) = core::str::from_utf8(&dma_buf[..bytes_read]) {
                    // Rust's .lines() automatically splits the burst by '\n'
                    for line in burst_str.lines() {
                        if line.starts_with("$GNGGA") || line.starts_with("$GPGGA") {
                            parse_and_send_gngga(line).await;
                        }
                    }
                }
            }
            Err(_) => error!("GPS UART Read Error"),
        }
    }
}

async fn parse_and_send_gngga(line: &str) {
    let get_part = |idx: usize| -> Option<&str> { line.split(',').nth(idx) };

    // Parse the raw number, then safely convert it to our Enum
    let raw_fix_num = get_part(6).unwrap_or("0").parse::<u8>().unwrap_or(0);
    let current_fix = GpsFix::from_u8(raw_fix_num);

    // Check our enum
    if current_fix != GpsFix::NoFix && current_fix != GpsFix::Unknown {
        let raw_time = get_part(1).unwrap_or("000000.00");
        let utc_time = parse_utc_time(raw_time);
        let raw_lat = get_part(2).unwrap_or("0.0");
        let lat_dir = get_part(3).unwrap_or("N");
        let raw_lon = get_part(4).unwrap_or("0.0");
        let lon_dir = get_part(5).unwrap_or("E");
        let altitude = get_part(9).unwrap_or("0.0").parse::<f32>().unwrap_or(0.0);

        let latitude = nmea_to_decimal(raw_lat, lat_dir);
        let longitude = nmea_to_decimal(raw_lon, lon_dir);

        let data = GnggaMessage {
            utc_time,
            latitude,
            longitude,
            altitude,
            fix: current_fix,
            timestamp_ms: embassy_time::Instant::now().as_millis() as u32,
        };

        DATA_CHANNEL.send(LogEvent::GPS(data)).await;
    } else {
        warn!("GPS: Waiting for Satellites (No Fix)");
    }
}
/// Converts NMEA DDMM.MMMM format into standard Decimal Degrees
fn nmea_to_decimal(raw: &str, direction: &str) -> f64 {
    let val = raw.parse::<f64>().unwrap_or(0.0);
    if val == 0.0 {
        return 0.0;
    }

    // Extract Degrees (DD) and Minutes (MM.MMMM)
    let degrees = (val / 100.0) as i32 as f64;
    let minutes = val - (degrees * 100.0) as f64;

    let mut decimal = degrees + (minutes / 60.0);

    // South and West are negative coordinates
    if direction == "S" || direction == "W" {
        decimal = -decimal;
    }

    decimal
}

// parse UTCTime
fn parse_utc_time(raw: &str) -> UtcTime {
    // If the string is empty or corrupted, return zeros
    if raw.len() >= 6 {
        // Slice the string by index: HH(0..2) MM(2..4) SS.SS(4..)
        let hours = raw.get(0..2).unwrap_or("0").parse::<u8>().unwrap_or(0);
        let minutes = raw.get(2..4).unwrap_or("0").parse::<u8>().unwrap_or(0);
        let seconds = raw.get(4..).unwrap_or("0.0").parse::<f32>().unwrap_or(0.0);

        UtcTime {
            hours,
            minutes,
            seconds,
        }
    } else {
        UtcTime {
            hours: 0,
            minutes: 0,
            seconds: 0.0,
        }
    }
}
