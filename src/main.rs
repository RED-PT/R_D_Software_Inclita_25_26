#![no_std]
#![no_main]
#![allow(unused_assignments)]
mod bno055;
mod configs;
mod mock_data;
mod sd_card;
use crate::configs::Board;
use defmt::*; // to use debuger shit
use embassy_executor::Spawner;
use embassy_stm32::gpio::Output;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = Board::set_clock();
    let p = embassy_stm32::init(config);

    info!("Setting stuff ");

    let mut board = Board::new(p);
    //Spawning tasks
    spawner
        .spawn(another_blinker(board.led_other_function))
        .unwrap();

    //spawner
    //  .spawn(sd_card::test_raw_read(board.sd_spi, board.sd_cs))
    //.unwrap();
    board
        .debug_uart
        .blocking_write(b"Hello from STM32!\r\n")
        .unwrap();
    //spawner.spawn(mock_data::mock_sensor_task()).unwrap();
    spawner
        .spawn(sd_card::sd_logger_task(board.sd_spi, board.sd_cs))
        .unwrap();
    spawner
        .spawn(bno055::bno055_logger_task(board.imu))
        .unwrap();
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
