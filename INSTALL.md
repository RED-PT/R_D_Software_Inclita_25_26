
**1. Install Rust**

If you don't have Rust, the official way is via [rustup](https://rustup.rs/) . This manages your compiler versions and targets.

Windows: Download `rustup-init.exe` from the [rustup](https://rustup.rs/) and run it .

Linux/macOS: Just run the following in the terminal:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
**2. Install the Target Architecture**

Standard Rust compiles for your computer (e.g., x86 or ARM64). We need to add the target for the STM32F413ZH (Cortex-M4F with hardware floating point):
```bash
rustup target add thumbv7em-none-eabihf
```
**3. Install Embedded Helper Tools**

We use several specialized "Cargo" sub-commands to handle the hardware:

`probe-rs`: The all-in-one tool to flash the chip and see logs (`defmt`) over the same USB cable.

`cargo-binutils`: For inspecting the binary size and sections.

```bash
# Install probe-rs (this may take a few minutes as it compiles)
cargo install probe-rs-tools

# Install llvm-tools for binary inspection
rustup component add llvm-tools-preview
cargo install cargo-binutils
```

**4. Running the Software**

You don't need to manually move binaries. The project is configured (via `.cargo/config.toml``) to use probe-rs as the runner. Just execute:

```bash
cargo run --release
```

**What happens when you run this?**

1. Compilation: Cargo compiles the code into an ELF file.

2. Optimization: The `--release` flag ensures the code is small enough for flash.

3. Flashing: `probe-rs` detects your the st-link and flashes the program.

4. Logging: The terminal will automatically start showing `defmt` logs (e.g., Starting SD HW initialization).

**If you are use to  C and STM32CubeIDE, here is how our Rust setup compares:**

|C Tooling   |   Rust / Embassy Equivalent  |
|--------------- | --------------- |
| `printf` ( AKA the HAL_Uart_Transmit())   | `defmt::info!()` (much faster, uses RTT)   |
| `STM32Cube Programmer`  | `probe-rs`   |
| `static` globals   |  `embassy_executor::task` with `'static` lifetimes  |
|  the IOC file and `config.h`   | `config.rs`  |




