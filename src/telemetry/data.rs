use core::cell::RefCell;
use defmt::Format;
use embassy_sync::blocking_mutex::{Mutex, raw::ThreadModeRawMutex};
use embassy_sync::channel::Channel;
use serde::Serialize;

///Magnetometer data structure
#[derive(Format, Serialize, Clone)]
pub struct MagnetometerData {
    pub mag_x: f32,
    pub mag_y: f32,
    pub mag_z: f32,
    pub timestamp_ms: u32,
}

/// IMUN DAta struct to use
#[derive(Format, Serialize, Clone)]
pub struct ImuData {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
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
#[derive(Format, Serialize, Clone)]
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
#[derive(Format, Serialize, Clone)]
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

#[derive(Format, Serialize, Clone)]
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
    Mag(MagnetometerData),
}
//The Channel (Our FreeRTOS StreamBuffer equivalent)
// the channel can hold up to 25 readings before the mock feeder has to wait.
pub static DATA_CHANNEL: Channel<ThreadModeRawMutex, LogEvent, 100> = Channel::new();

// LoRa smaller structs

#[derive(Format, Serialize, Clone)]
pub struct ImuTx {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub mag_x: f32,
    pub mag_y: f32,
    pub mag_z: f32,
    pub lin_accel_n: f32,
    pub lin_accel_e: f32,
    pub lin_accel_d: f32,
}

// Quick converter from SD struct to the LoRa struct
impl From<ImuData> for ImuTx {
    fn from(d: ImuData) -> Self {
        Self {
            yaw: d.yaw,
            pitch: d.pitch,
            roll: d.roll,
            mag_x: d.mag_x,
            mag_y: d.mag_y,
            mag_z: d.mag_z,
            lin_accel_n: d.lin_accel_n,
            lin_accel_e: d.lin_accel_e,
            lin_accel_d: d.lin_accel_d,
        }
    }
}

#[derive(Format, Serialize, Clone)]
pub struct AltimeterTx {
    pub pressure: f32,
    pub altitude: f32,
    pub temperature: f32,
}

impl From<AltimeterData> for AltimeterTx {
    fn from(d: AltimeterData) -> Self {
        Self {
            pressure: d.pressure,
            altitude: d.altitude,
            temperature: d.temperature,
        }
    }
}

#[derive(Format, Serialize, Clone)]
pub struct GpsTx {
    pub latitude: f64,
    pub longitude: f64,
    pub fix: GpsFix,
    pub altitude: f32,
}

impl From<GnggaMessage> for GpsTx {
    fn from(d: GnggaMessage) -> Self {
        Self {
            latitude: d.latitude,
            longitude: d.longitude,
            fix: d.fix,
            altitude: d.altitude,
        }
    }
}

// This struct groups the latest data.
// Option allows us to transmit even if some sensors haven't fired yet.
#[derive(Format, Serialize, Clone)]
pub struct DownlinkPacket {
    pub imu: Option<ImuData>,
    pub baro: Option<AltimeterData>,
    pub gps: Option<GnggaMessage>,
}

// why not making the DownlinkPacket an enum?
pub enum DownlinkEvent {
    Imu(ImuData),
    Baro(AltimeterData),
    GPS(GnggaMessage),
}

// A global, thread-safe variable to hold the latest state
// INFO: We use RefCell cause, in a no std environment, Mutex does not implement interior mutablity
// embassy_sync has signal. it seems more appropriate here! all my homies hate RefCell in no_std! use embassy_sync instead! theres always a sync primitive there you can use! 
// plus it has async support, so you can await on it!
pub static LATEST_TELEMETRY: Mutex<ThreadModeRawMutex, RefCell<DownlinkPacket>> =
    Mutex::new(RefCell::new(DownlinkPacket {
        imu: None,
        baro: None,
        gps: None,
    }));
