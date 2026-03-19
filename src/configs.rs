//! This file will contain all the hw configs necessary: debug  uart, gps uart, sdmmc , spi
//! altimeter, i2c gyroscope, etc...
//! Kinda works like the config.h in the "C" code were used to
//!
#![allow(unreachable_code)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use embassy_stm32::Peripherals;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{Config as I2cConfig, I2c};
use embassy_stm32::interrupt::typelevel::DMA2_STREAM3;
use embassy_stm32::mode::Blocking;
use embassy_stm32::pac;
use embassy_stm32::rcc::*;
use embassy_stm32::sdmmc::{Config as SdmmcConfig, Sdmmc};
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::usart::{Config as UsartConfig, Uart};
use embassy_stm32::{Config, bind_interrupts, dma, peripherals, sdmmc};
// TODO: sdmmc.rs or something, this is messy

//NVIC and DMA
bind_interrupts!(struct Irqs {
    SDIO => sdmmc::InterruptHandler<peripherals::SDIO>;
});

pub struct Board<'a> {
    pub debug_uart: Uart<'a, Blocking>, //as of now on blocking mode, maybe change this later
    pub led_mcu_on: Output<'a>,
    pub led_other_function: Output<'a>,
    // I2C expects <Lifetime, Mode, MasterMode>
    pub gyro: I2c<'a, Blocking, embassy_stm32::i2c::Master>, // this will prolly work in blocking,
    // as of now were using bno
    pub gps_uart: Uart<'a, Blocking>, // TODO: Figure out circular DMA, later, i want to have this use Async
    // SPI expects <Lifetime, Mode, CommunicationMode>
    pub altimeter: Spi<'a, Blocking, embassy_stm32::spi::mode::Master>, //prollly will keep in
    //blocking mode, as of now were using ms
    pub altimeter_cs: Output<'a>,
    pub sd_spi: Spi<'a, Blocking, embassy_stm32::spi::mode::Master>,
    pub sd_cs: Output<'a>,
}

impl Board<'static> {
    //we need this because of the sd card
    pub fn set_clock() -> Config {
        let mut config = Config::default();

        config.rcc.hse = None; // We dont have an external oscilator
        config.rcc.hsi = true;

        config.rcc.pll_src = PllSource::HSI;

        // math to suply the sdio with 48mhz
        config.rcc.pll = Some(Pll {
            // MATH: HSI is 16 MHz
            prediv: PllPreDiv::DIV8, // 16 MHz / 8 = 2 MHz (Optimal VCO input)
            mul: PllMul::MUL192,     // 2 MHz * 192 = 384 MHz (Internal VCO frequency)
            divp: Some(PllPDiv::DIV4), // 384 MHz / 4 = 96 MHz (Main CPU Clock - Safe!)
            divq: Some(PllQDiv::DIV8), // 384 MHz / 8 = 48 MHz (Exact SDMMC requirement!)
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;

        config
    }
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

        // SD CARD SPI
        let mut sd_spi_cfg = SpiConfig::default();
        // SD cards usually need to start slow (e.g., 400kHz) for initialization,
        // the embedded-sdmmc crate handles this, but we'll set a moderate default.
        sd_spi_cfg.frequency = embassy_stm32::time::mhz(1);

        let sd_spi = Spi::new_blocking(
            p.SPI2, p.PD3, // SCK
            p.PC3, // MOSI
            p.PC2, // MISO
            sd_spi_cfg,
        );

        // Chip Select must start HIGH (deselected)
        let sd_cs = Output::new(p.PG5, Level::High, Speed::High);
        Self {
            debug_uart,
            led_mcu_on,
            led_other_function,
            gyro,
            gps_uart,
            altimeter,
            altimeter_cs,
            sd_spi,
            sd_cs,
        }
    }
}
