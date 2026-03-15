#![no_std]
#![no_main]
#![allow(unused_assignments)]
use crate::configs::Board;
use defmt::*; // to use debuger shit
use embassy_executor::Spawner;
use embassy_stm32::gpio::Output;
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

mod configs;
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    //we need to set the clock because of the sdmmc, which uses PLL1_P
    // TODO: move timer configs outside of main
    let mut config = Config::default();

    {
        use embassy_stm32::rcc::*;

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
    }
    let p = embassy_stm32::init(config);

    info!("Setting stuff ");

    let mut board = Board::new(p);
    //Spawning tasks
    spawner
        .spawn(another_blinker(board.led_other_function))
        .unwrap();

    board
        .debug_uart
        .blocking_write(b"Hello from STM32!\r\n")
        .unwrap();
    loop {
        board.debug_uart.blocking_write(b"High!\r\n").unwrap();
        board.led_mcu_on.set_high();
        Timer::after_millis(300).await;

        board.debug_uart.blocking_write(b"Low!\r\n").unwrap();

        board.led_mcu_on.set_low();
        Timer::after_millis(300).await;
    }
}

#[embassy_executor::task]
async fn another_blinker(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(1000).await;
        led.set_low();
        Timer::after_millis(1000).await;
    }
}
