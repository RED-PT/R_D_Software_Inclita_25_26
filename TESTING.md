# Testing in the Software

In embedded systems, concepts like **SIL** (Software-In-The-Loop) and **HIL** (Hardware-In-The-Loop) are use to validate the software ( ex the logging for the sd card, the FSM) without needing to do an test flight.

In rust, we can easily switch between "testing software" and "flight software", using [Cargo Features](https://doc.rust-lang.org/cargo/reference/features.html). There is no need to manualy "comment" or "un-comment" code, to run diferent software, like we would probabily do in a Lazy C approach (or we would use macros, anyways, both approaches are ugly and can lead to errors).

Cargo Features  allows us to conditionally compile either the real sensor drivers or the mock generators.

### Example using cargo features

1. In Cargo.toml and add a [features] section at the bottom.

```toml
[features]
default = []
mock-sensors = [] # We will use this flag to build the mock version
```
2. To tell the compiler which tasks to compile, we use Rust's #[cfg(...)] macro. In the `main.rs`:
```Rust
// SD Card task always runs
    spawner.spawn(storage::sd_card::sd_logger_task(board.sd_spi, board.sd_cs)).unwrap();

    // --- FLIGHT MODE: Real Sensors ---
    #[cfg(not(feature = "mock-sensors"))]
    {
        info!("Compiling in FLIGHT MODE");
        spawner.spawn(sensors::bno055::bno055_logger_task(board.imu)).unwrap();
        spawner.spawn(sensors::ms5611::ms5611_task(board.altimeter, board.altimeter_cs)).unwrap();
    }

    // --- TEST MODE: Mock Sensors ---
    #[cfg(feature = "mock-sensors")]
    {
        info!("Compiling in TEST MODE");
        spawner.spawn(sensors::mock::mock_imu_task()).unwrap();
        spawner.spawn(sensors::mock::mock_baro_task()).unwrap();
    }
```

When you just run `cargo run`, it compiles the real I2C/SPI drivers.
When you want to test the SD card without the sensors attached, you run:
`cargo run --features mock-sensors` !
