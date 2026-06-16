# Requirements

### Overview & Goals
The goal of this task is to implement a "Pure Rust" SocketCAN driver for `libcsp-rs`. This approach uses the Rust `socketcan` crate for low-level CAN communication while interfacing with the `libcsp` (C) stack through its generic CAN interface layer. This replaces or complements the previous approach of wrapping the C-based `can_socketcan.c` driver, providing better safety and idiomatic Rust integration.

### Scope
- **In Scope**:
    - Configuration of `libcsp` and `libcsp-sys` to support the `socketcan` feature.
    - Implementation of `CspCanInterface` and its associated context and callbacks in Rust.
    - Integration with `libcsp`'s C-based CAN receive and transmit mechanisms.
- **Out of Scope**:
    - Modification of the C source code of `libcsp`.
    - Implementation of non-CAN interfaces (unless required for consistency).

### User Stories
- As a developer using `libcsp-rs`, I want a native Rust SocketCAN interface so that I can use standard Linux CAN tools and drivers without relying on C-based driver implementations.
- As a developer, I want the CAN driver to be safe and leverage Rust's concurrency and memory safety features.


# Technical Design

### Current Implementation
- `libcsp-rs` (on the `libcsp-2-0` branch) wraps `libcsp` version 2.0.
- Interfaces are added using the `InterfaceBuilder` trait.
- Currently, a ZMQ-based interface is implemented using this pattern.

### Proposed Changes
#### 1. Dependency Update
Modify `libcsp/Cargo.toml` to include:
```toml
[dependencies]
socketcan = { version = "3.3.0", optional = true }

[features]
socketcan = ["std", "libcsp-sys/socketcan", "dep:socketcan"]
```

#### 2. CspCanInterface Implementation
In `libcsp/src/interface.rs`, implement `CspCanInterface` and the `InterfaceBuilder` trait:
- **Structs**:
    - `CspCanInterface`: Public struct for configuring the interface (name, bitrate, promisc).
    - `CspCanContext`: Internal struct to hold the `CanSocket` and a pointer to the `csp_iface_t`, used as `driver_data`.
- **Logic**:
    - `build` method:
        - Opens `CanSocket`.
        - Allocates `csp_iface_t` and `csp_can_interface_data_t` via `Box::into_raw` to ensure they remain alive for the duration of the interface.
        - Calls `csp_can_add_interface` to register the interface with the C library.
        - Spawns a background thread that loops, reading frames from the socket and calling `csp_can_rx`.
- **TX Callback**:
    - `extern "C" fn rust_csp_can_tx_frame`: Takes raw data from `libcsp`, wraps it in a `CanFrame`, and writes it to the `CanSocket`.

#### 3. Error Handling
The implementation will map `socketcan` errors to the appropriate `CspErrorKind` (e.g., `Driver`, `Tx`, or `Inval`) and provide descriptive error messages via the `CspError` struct.

### File Structure
- `libcsp/Cargo.toml`: Add dependencies and features.
- `libcsp/src/interface.rs`: Implementation of the interface and callbacks.


# Delivery Steps

###   Step 1: Verify Environment and Branch
Preparation for implementation.
- Ensure the repository is on the `libcsp-2-0` branch.
- Verify that the Nix environment (if used) is correctly set up with `libsocketcan` as defined in `shell.nix`.


###   Step 2: Add socketcan Dependency and Feature
Add the necessary dependencies to the Rust project.
- Modify `libcsp/Cargo.toml` to add `socketcan = "3.3.0"` as an optional dependency.
- Update the `features` section in `libcsp/Cargo.toml` to include a `socketcan` feature that enables the `socketcan` dependency and the `libcsp-sys/socketcan` feature.


###   Step 3: Implement CspCanInterface and InterfaceBuilder
Implement the core CAN interface logic in Rust.
- Modify `libcsp/src/interface.rs` to add `CspCanInterface` and `CspCanContext` structures.
- Implement the `InterfaceBuilder` trait for `CspCanInterface`.
- Implement the `build` method to:
    - Open and configure a `socketcan::CanSocket`.
    - Allocate and initialize `csp_iface_t` and `csp_can_interface_data_t` using `Box::into_raw` to interface with the C library.
    - Set up the interface parameters (name, address, netmask).
    - Register the interface using `csp_can_add_interface`.
    - Spawn a background thread to handle RX frames and pass them to `csp_can_rx`.


###   Step 4: Implement TX Callback Function
Implement the transmission callback for the CAN interface.
- Add an `extern "C"` function `rust_csp_can_tx_frame` in `libcsp/src/interface.rs`.
- Implement the logic to convert raw CAN ID and data into a `socketcan::CanFrame` and write it to the socket.
- Handle error mapping from `socketcan` results to `libcsp` error codes.


###   Step 5: Integration and Verification
Finalize the integration and verify the changes.
- Update `CspError` usage in the new code to match the existing `CspError` struct and `CspErrorKind` enum.
- (If applicable) Update external `executive` and `tester` components to utilize the new `CspCanInterface`.
- Verify compilation of `libcsp` with the `socketcan` feature enabled.
