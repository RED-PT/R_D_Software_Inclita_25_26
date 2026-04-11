# We using Rust! How can you Compile and Rust this software?

Please read [INSTALL.md](/INSTALL.md)  for instructions, from installing rust, to flashing to the board :).
Also, for more details on  how to use the debugger and related stuff go see  [DEBUG.md](/DEBUG.md) !

This repo contains the pilot experiment for Inclita 25/26, were the avionics software is written in Rust, in order to evaluate Rust as a primary language for embedded flight software sistems.
The [Embassy Framework](https://github.com/embassy-rs/embassy)  was chosen, aiming to achieve high-performance concurrency with significantly lower power consumption and higher memory safety than traditional C-based FreeRTOS approach.

Rust was chosen also because of being easy to deploy testing and mocks, to see how testing is being implemented, please see [TESTING.md](/TESTING.md)

## Key Features

- The software is built for the STM32F413ZH (Cortex-M4F). Unlike traditional "Super Loop" or threaded RTOS architectures, this project uses an [Async/Wait](https://rust-lang.github.io/async-book/) [Executor](https://embassy.dev/book/#_embassy_executor) , allowing peripherals to wait for hardware events without blocking the CPU.

- **Dual Thread-Safe Data Pipelines:**
  - **Lossless SD Logging:** Uses `embassy_sync::Channel` to safely queue high-speed telemetry (`LogEvent`) from sensor tasks into a slower SD card write buffer.
  - **Stateful Radio Telemetry:** Uses a globally locked `Mutex<ThreadModeRawMutex, RefCell<DownlinkPacket>>` to maintain the latest state of the rocket, allowing the radio to broadcast snapshots without halting sensor loops.
- **Optimized LoRa Bandwidth (DTO Pattern):** Radio payloads strip redundant timing data into lightweight structs (e.g., `ImuTx`, `GpsTx`) and inject a single transmission timestamp, enabling high-speed 5Hz telemetry bursts over the RFM95 module.
- Serialization: Uses [postcard](https://docs.rs/postcard/latest/postcard/)  and [Serde](https://crates.io/crates/serde)  to pack data structs directly into raw bytes (no_std compatible).

- File Generation: Scans the SD card on boot and creates sequential, non-destructive log files (e.g., DATA1.BIN, DATA2.BIN).

## Project Structure (as of now)

```
── hardware_cfg.rs  // Board Peripheral and Clock configs, works like the .IOC file
├── main.rs //entry point of the flight software: initializes the STM32 hardware and uses the Embassy executor to spawn all asynchronous tasks.
├── sensors
│   ├── bno055.rs
│   ├── gps.rs //Circular DMA and UART IDLE Line Detection  tocapture and parse 5Hz NMEA $GNGGA bursts.
│   ├── mock.rs //For future use. I would like to use SIL
│   ├── mod.rs
│   └── ms5611.rs //50Hz asynchronous loop over SPI to read pressure and temperature
├── storage
│   ├── mod.rs
│   ├── sd_card_utils.rs // Functions to be used with the sd card operations, aiming at a DRY approach.
│   └── sd_card.rs //Uses SPI to interface with the SD card's FAT file system. It finds the empty file slots (e.g., IMU_1.BIN, BARO_1.BIN) and acts as a mail-sorter, grabbing LogEvent packets from the telemetry channel, serializing them into binary using postcard, and safely executing burst-writes.
└── telemetry
    ├── data.rs ////All serde-derivable data structures, the LogEvent enum, the asynchronous Channel to pass data safely between the sensor tasks and SD card task, and the refcell wrapped mutex, to pass data between sensor tasks and the lora task
    ├── lora.rs
    └── mod.rs
```

## Roadmap

- [x] Basic async executor and GPIO blinking.

- [x] SD CARD use via SPI
- [ ] Switch from SPI to SDMMC

- [ ] Async GPS data (TBT)

- [x] Implement high-level file system access on the SD card.

- [x] Get IMU data via polling.

- [x] Store IMU and Altimeter data in the SD Card.

- [ ] Lora Comms  to the GS and think of  a better protocol(TBT)

# VSCode extensions

- [Rust](https://marketplace.visualstudio.com/vscode/item?itemName=rust-lang.rust) (official Rust extension for syntax highlighting and basic support)
- [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer)  (for code completion, inline documentation, and error checking)
- [Even Better TOML](https://marketplace.visualstudio.com/vscode/item?itemName=tamasfe.even-better-toml)  (for editing Cargo.toml files)
- [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)  (for debugging Rust code with LLDB)
- [Error Lens](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens)  (to highlight Rust compiler errors directly in the code)
- [Crate Graph](https://marketplace.visualstudio.com/items?itemName=alexdima.crate-graph)  (to visualize the dependency graph of Rust crates in the project)
- [Debugger for probe-rs](https://marketplace.visualstudio.com/vscode/item?itemName=probe-rs.probe-rs-debugger) (for debugging embedded Rust code with probe-rs)
- [Dependi](https://marketplace.visualstudio.com/vscode/item?itemName=fill-labs.dependi) (for visualizing and managing Rust dependencies in Cargo.toml)
- [Path Intellisense](https://marketplace.visualstudio.com/vscode/item?itemName=christian-kohler.path-intellisense) (for autocompleting file paths in Rust code)
- [Serial Monitor](https://marketplace.visualstudio.com/vscode/item?itemName=ms-vscode.vscode-serial-monitor) (for monitoring serial output from embedded Rust applications)

# crates (to install)

- bacon
- cargo-binstall
- cargo-nextest
- coreutils

- use clippy!

- unwrap might be okay for startup. its easier to deal with than error handling. during routines / loops, it should use error handling with logging, ideally telemtetry too. But if not, expect is a good middle ground, which panics with error mesages!

- for drivers crates, see https://vaishnav.world/Hayasen/