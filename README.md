# We using Rust! How can you Compile and Rust this software?
Please read [INSTALL.md](/INSTALL.md)  for instructions, from installing rust, to flashing to the board :).
Also, for more details on  how to use the debugger and related stuff go see  [DEBUG.md](/DEBUG.md) !


This repo contains the pilot experiment for Inclita 25/26, were the avionics software is written in Rust, in order to evaluate Rust as a primary language for mission-critical embedded software sistems. 
The [Embassy Framework](https://github.com/embassy-rs/embassy)  was chosen, aiming to achieve high-performance concurrency with significantly lower power consumption and higher memory safety than traditional C-based FreeRTOS approach.

Rust was chosen also because of being easy to deploy testing and mocks, to see how testing is being implemented, please see [TESTING.md](/TESTING.md) 



## Key Features

- The software is built for the STM32F413ZH (Cortex-M4F). Unlike traditional "Super Loop" or threaded RTOS architectures, this project uses an [Async/Wait](https://rust-lang.github.io/async-book/) [Executor](https://embassy.dev/book/#_embassy_executor) , allowing peripherals to wait for hardware events without blocking the CPU.

- Thread-Safe Data Pipelines: Uses [embassy_sync::Chanel](https://docs.embassy.dev/embassy-sync/0.7.2/default/channel/struct.Channel.html) to safely pass data between the sensor polling task and the slower SD card writing task.

- Serialization: Uses [postcard](https://docs.rs/postcard/latest/postcard/)  and [Serde](https://crates.io/crates/serde)  to pack data structs directly into raw bytes (no_std compatible).

- File Generation: Scans the SD card on boot and creates sequential, non-destructive log files (e.g., DATA1.BIN, DATA2.BIN).

## Hardware:
| Peripheral | Component | Protocol | MCU Pins (Example) |
| :--- | :--- | :--- | :--- |
| **Debug Console** | UART to USB | USART3 | TX: PD8, RX: PD9 |
| **IMU** | BNO055 | I2C1 (Blocking) | SCL: PA8, SDA: PB4 |
| **Storage** | MicroSD Card | SPI2 | SCK: PD3, MOSI: PC3, MISO: PC2, CS: PG5 |
| **Altimeter** | MS5611 (Planned) | SPI1 | SCK: PA5, MOSI: PA7, MISO: PA6, CS: PC4 |
| **GPS** | NEO-6M (Planned) | UART5 | TX: PB12, RX: PB13 |


## Project Structure (as of now)

```

├── hardware_cfg.rs
├── main.rs //entry point of the flight software: initializes the STM32 hardware and uses the Embassy executor to spawn all asynchronous tasks.
├── sensors //Hardware Abstraction Layer (HAL) configs: setting up the system clocks, binding interrupts, initializing the peripherals (SPI, I2C, UART, DMA)
│   ├── bno055.rs //(IMU) driver. Runs a 100Hz asynchronous loop over I2C
│   ├── gps.rs //Circular DMA and UART IDLE Line Detection  tocapture and parse 5Hz NMEA $GNGGA bursts.
│   ├── mock.rs // For future use in SIL (software In the Loop)
│   ├── mod.rs
│   └── ms5611.rs //50Hz asynchronous loop over SPI to read pressure and temperature
├── storage
│   ├── mod.rs
│   └── sd_card.rs //Uses SPI to interface with the SD card's FAT file system. It finds the empty file slots (e.g., IMU_1.BIN, BARO_1.BIN) and acts as a mail-sorter, grabbing LogEvent packets from the telemetry channel, serializing them into binary using postcard, and safely executing burst-writes.
└── telemetry.rs //All serde-derivable data structures, the LogEvent enum, the asynchronous Channel to pass data safely between the sensor tasks and SD card task.
```
## Roadmap
- [x] Basic async executor and GPIO blinking.

- [x] SD CARD use via SPI
- [ ] Switch from SPI to SDMMC

- [ ] Transition GPS UART to Circular DMA with Async support.

- [x] Implement high-level file system access on the SD card.

- [x] Get IMU data via polling.

- [ ] Store flight data in the SD Card.
 
