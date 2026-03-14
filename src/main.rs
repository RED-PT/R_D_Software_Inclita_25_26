#![no_std]
#![no_main]

use defmt::*; // to use debuger shit
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
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
    let mut led = Output::new(p.PF2, Level::High, Speed::Low);
    let mut slow_led = Output::new(p.PG7, Level::High, Speed::Low);
    let mut config = Config::default();
    config.baudrate = 115200;

    // Initialize the UART peripheral
    let mut serial = Uart::new_blocking(
        p.USART3, p.PD9, // RX pin
        p.PD8, // TX pin
        config,
    )
    .unwrap();

    //Spawning tasks
    spawner.spawn(another_blinker(slow_led)).unwrap();

    serial.blocking_write(b"Hello from STM32!\r\n").unwrap();
    loop {
        serial.blocking_write(b"High!\r\n").unwrap();
        led.set_high();
        Timer::after_millis(300).await;

        serial.blocking_write(b"Low!\r\n").unwrap();

        led.set_low();
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
