//! This file will contain all the hw configs necessary: debug  uart, gps uart, sdmmc , spi
//! altimeter, i2c gyroscope, etc...
//! Kinda works like the config.h in the "C" code were used to
//!
#![allow(unreachable_code)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{Config as I2cConfig, I2c};
use embassy_stm32::mode::Blocking;
use embassy_stm32::sdmmc::{Config as SdmmcConfig, Sdmmc};
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::usart::{Config as UsartConfig, Uart};
use embassy_stm32::Peripherals;

pub struct Board<'a> {
    pub debug_uart: Uart<'a, Blocking>, //as of now on blocking mode, maybe change this later
    pub led_mcu_on: Output<'a>,
    pub led_other_function: Output<'a>,
    pub sd_card: Sdmmc<'a>,
    // I2C expects <Lifetime, Mode, MasterMode>
    pub gyro: I2c<'a, Blocking, embassy_stm32::i2c::Master>, // this will prolly work in blocking,
    // as of now were using bno
    pub gps_uart: Uart<'a, Blocking>, // TODO: Figure out circular DMA, later, i want to have this use Async
    // SPI expects <Lifetime, Mode, CommunicationMode>
    pub altimeter: Spi<'a, Blocking, embassy_stm32::spi::mode::Master>, //prollly will keep in
    //blocking mode, as of now were using ms
    pub altimeter_cs: Output<'a>,
}

impl Board<'static> {
    pub fn new(p: Peripherals) -> Self {
        let mut debug_uart_cfg = UsartConfig::default();
        debug_uart_cfg.baudrate = 115200;
        let debug_uart = Uart::new_blocking(p.USART3, p.PD9, p.PD8, debug_uart_cfg).unwrap();

        let led_mcu_on = Output::new(p.PF2, Level::High, Speed::Low);
        let led_other_function = Output::new(p.PB0, Level::High, Speed::Low);

        // GPS UART
        let mut gps_uart_cfg = UsartConfig::default();
        gps_uart_cfg.baudrate = 38400;
        let gps_uart = Uart::new_blocking(p.UART5, p.PB12, p.PB13, gps_uart_cfg).unwrap();

        // GYRO I2C
        let i2c_cfg = I2cConfig::default();
        let gyro = I2c::new_blocking(p.I2C3, p.PA8, p.PB4, i2c_cfg);

        // Altimeter SPI
        let spi_cfg = SpiConfig::default();
        let altimeter = Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, spi_cfg);
        let altimeter_cs = Output::new(p.PC4, Level::High, Speed::High); //initalized
                                                                         //to high -> MS inactive

        // SD CARD SDMMC
        let sdmmc_cfg = SdmmcConfig::default();
        let irq = todo!("Add SDMMC IRQ");
        let dma = todo!("Add SDMMC DMA");
        let sd_card = Sdmmc::new_4bit(
            p.SDIO, irq, dma, p.PC12, p.PD2, p.PC8, p.PC9, p.PC10, p.PC11, sdmmc_cfg,
        );

        Self {
            debug_uart,
            led_mcu_on,
            led_other_function,
            sd_card,
            gyro,
            gps_uart,
            altimeter,
            altimeter_cs,
        }
    }
}
