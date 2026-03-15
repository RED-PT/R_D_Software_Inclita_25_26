# Debugging with `defmt`

In this project, we use `defmt` ("deferred formatting") for logging. It is significantly more efficient than `printf` because it doesn't transmit full strings; it only sends small IDs that the PC decodes using the project's ELF file.


### 1. How to log
You can use various log levels in your code:
 
```rust
info!("Starting SD HW initialization"); // Standard information
debug!("Calculated PLL: {}", val);       // Debugging detail
error!("Failed to initialize I2C");     // Error states
```

### 2. Viewing the Logs

When you run `cargo run`, the `probe-rs` tool automatically opens an RTT (Real-Time Transfer) channel. Your terminal will look like this:
```
0.000000 INFO  Setting stuff
0.000120 INFO  Starting SD HW initialization
0.005321 INFO  Card: CardInfo { ... }
```
### 3. Formatting Complex Types
`defmt` allows for sophisticated formatting without the overhead:

- Hex Arrays: 
```rust 
info!("Read: {=[u8]:X}", block[..8])
```

- Debug Structures:
```rust
info!("Card: {:#?}", Debug2Format(&card));
```

- Binary: 
```rust
info!("Bits: {:b}", 0b1010);
```


### Visual Debugging (VS Code)

For those who prefer a GUI over the command line, this project is configured for one-click debugging using the `probe-rs` extension.

(or NeoVim, for those who use NeoVim, my NeoVim configs also account for this debugger, and works really nice, go check my repo at [Nvim_setup](https://github.com/sofiavldd2005/Nvim_setup) :))

#### 1. Install the Extension
Go to the VS Code Extensions marketplace and install:

Name: probe-rs

Identifier: probe-rs.probe-rs-debugger

#### 2. Launching a Session
Connect your board.

Go to the Run and Debug tab in VS Code (Ctrl+Shift+D).

Select "Embassy Debug (probe-rs)" from the dropdown.

Press F5.

#### 3. Features available:
Breakpoints: Click to the left of line numbers to set breakpoints in main.rs or your drivers.

Variable Inspection: View local variables and the call stack when a breakpoint is hit.

Integrated Console: defmt logs will appear directly in the VS Code "Debug Console" tab.

[!TIP]
The configuration is currently pointed at your debug binary:
`target/thumbv7em-none-eabihf/debug/r_d_inclita_sofware`
If you want to debug an optimized build, change the path to `/release/` in `.vscode/launch.json`.


