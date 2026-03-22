use defmt::Format;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use heapless::String;
use serde::Serialize;
//Mock data struct
#[derive(Format, Serialize)]
pub struct ImuData {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub temperature: f32,
    pub mag_x: f32,
    pub mag_y: f32,
    pub mag_z: f32,
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
    pub lin_accel_n: f32,
    pub lin_accel_e: f32,
    pub lin_accel_d: f32,
    pub timestamp_ms: u32,
}
#[derive(Format, Serialize)]
pub struct AltimeterData {
    pub pressure: f32,
    pub altitude: f32,
    pub temperature: f32,
    pub timestamp_ms: u32,
}

#[derive(Format, Serialize, Clone, Copy, PartialEq)]
pub enum GpsFix {
    NoFix = 0,
    Standard = 1,   // Standard GPS Fix
    Dgps = 2,       // Differential GPS (More accurate)
    Pps = 3,        // Precise Positioning Service
    RtkInteger = 4, // Real-Time Kinematic (Highest accuracy)
    RtkFloat = 5,
    Estimated = 6,
    Manual = 7,
    Simulation = 8,
    Unknown, // Safety fallback for corrupted data
}

impl GpsFix {
    // A clean way to convert the raw u8 from the GPS into our type-safe enum
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => GpsFix::NoFix,
            1 => GpsFix::Standard,
            2 => GpsFix::Dgps,
            3 => GpsFix::Pps,
            4 => GpsFix::RtkInteger,
            5 => GpsFix::RtkFloat,
            6 => GpsFix::Estimated,
            7 => GpsFix::Manual,
            8 => GpsFix::Simulation,
            _ => GpsFix::Unknown,
        }
    }
}
#[derive(Format, Serialize)]
pub struct GnggaMessage {
    /// UTC Time (HHMMSS.SS)
    pub utc_time: UtcTime,
    /// Latitude in decimal degrees (Positive = North, Negative = South)
    pub latitude: f64,
    /// Longitude in decimal degrees (Positive = East, Negative = West)
    pub longitude: f64,
    /// Quality of the fix
    pub fix: GpsFix,

    /// Altitude above mean sea level (Meters)
    pub altitude: f32,

    pub timestamp_ms: u32,
}

#[derive(Format, Serialize)]
pub struct UtcTime {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: f32, // f32 because NMEA includes milliseconds (e.g., 14.50s)
}
// The wrapping envelope
#[derive(Format, Serialize)]
pub enum LogEvent {
    Imu(ImuData),
    Baro(AltimeterData),
    GPS(GnggaMessage),
}
//The Channel (Our FreeRTOS StreamBuffer equivalent)
// the channel can hold up to 25 readings before the mock feeder has to wait.
pub static DATA_CHANNEL: Channel<ThreadModeRawMutex, LogEvent, 50> = Channel::new();
