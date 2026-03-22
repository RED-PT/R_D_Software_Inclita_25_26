#![no_std]
#![no_main]
#![allow(unused_assignments)]
mod hardware_cfg;
mod sensors;
mod storage;
mod telemetry;
use crate::hardware_cfg::Board;
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
        .spawn(storage::sd_card::sd_logger_task(board.sd_spi, board.sd_cs))
        .unwrap();
    #[cfg(not(feature = "mock-sensors"))]
    {
        info!("Compiling in FLIGHT MODE");
        spawner
            .spawn(sensors::bno055::bno055_logger_task(board.imu))
            .unwrap();
        spawner
            .spawn(sensors::ms5611::ms5611_task(
                board.altimeter,
                board.altimeter_cs,
            ))
            .unwrap();
    }

    // --- TEST MODE: Mock Sensors ---
    #[cfg(feature = "mock-sensors")]
    {
        info!("Compiling in TEST MODE");
        spawner.spawn(sensors::mock::mock_imu_task()).unwrap();
        spawner.spawn(sensors::mock::mock_baro_task()).unwrap();
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
