//! This file will contain all the hw configs necessary: debug  uart, gps uart, sdmmc , spi
//! altimeter, i2c gyroscope, etc...
//! Kinda works like the config.h in the "C" code were used to
//!

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{Config as I2cConfig, I2c};
use embassy_stm32::mode::{Async, Blocking};
use embassy_stm32::sdmmc::{Config as SdmmcConfig, Sdmmc};
use embassy_stm32::usart::{Config as UsartConfing, Uart};
use embassy_stm32::Peripherals;
use embassy_stm32::{spi, Config as SpiConfig};
