# We using Rust! How can you Compile and Rust this software?
See the INSTALL.md file for instructions :).


This repo contains the pilot experiment for Inclita 25/26, were the avionics software is written in Rust, in order to evaluate Rust as a primary language for mission-critical embedded software sistems. 
The [Embassy Framework](https://github.com/embassy-rs/embassy)  was chosen, aiming to achieve high-performance concurrency with significantly lower power consumption and higher memory safety than traditional C-based FreeRTOS approach.



## The hardware

The software is built for the STM32F413ZH (Cortex-M4F). Unlike traditional "Super Loop" or threaded RTOS architectures, this project uses an [Async/Wait](https://rust-lang.github.io/async-book/) [Executor](https://embassy.dev/book/#_embassy_executor) , allowing peripherals to wait for hardware events without blocking the CPU.

### Hardware Drivers we are aiming for:
1. SDMMC: 4-bit wide bus configuration.
2. UART: Debug logging via USART3 (algo we can just for using the debuger, and get debug messages via the `info!()` macro) and GPS integration via UART5.
3. I2C: Configured for IMU data acquisition.
4. SPI: Dedicated bus for Altimeter sensor.



## Project Structure (as of now)
`main.rs`: The async entry point. It handles the Embassy executor initialization and spawns concurrent tasks like blinkers and sensor polling.

`configs.rs`: Our "Hardware Abstraction Layer" (HAL) wrapper. It acts similarly to a C config.h, centralizing pin assignments, clock trees, and peripheral ownership in a Board struct.

`sd_card.rs`: Low-level SDMMC test function to verify DMA transfers and raw block storage.

## Roadmap
[x] Basic async executor and GPIO blinking.

[x] SDMMC Raw block read/write verification.

[ ] Transition GPS UART to Circular DMA with Async support.

[ ] Implement embedded-fatfs for high-level file system access on the SD card.

[ ] Get IMU data via polling.

[ ] Store flight data in the SD Card.

