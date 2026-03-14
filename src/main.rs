#![no_std]
#![no_main]

use crate::configs::Board;
use defmt::*; // to use debuger shit
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    i2c::mode::MasterMode,
    spi::mode::CommunicationMode,
    timer::input_capture::InputCapture,
};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};
// This links our chosen panic handler
use embassy_stm32::usart::{Config, Uart};

mod configs;
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

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
