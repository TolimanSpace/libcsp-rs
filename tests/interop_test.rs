use std::process::Command;
use std::thread;
use std::time::Duration;
use libcsp::{
    CspConnAddress, CspConnPriority, LibCspBuilder, LibCspConfig, Route,
    interface::CspZmqInterface,
};

#[test]
#[ignore]
fn test_c_server_interop() {
    // 1. Compile C code
    // We assume libcsp and zmq are available in the environment (nix-shell)
    let mut c_src_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    c_src_path.pop(); // go to root
    c_src_path.push("tests");
    let c_src_file = c_src_path.join("interop_server.c");
    let c_bin_file = c_src_path.join("interop_server");

    let status = Command::new("gcc")
        .args([
            c_src_file.to_str().unwrap(),
            "-o", c_bin_file.to_str().unwrap(),
            "-lcsp", "-lzmq"
        ])
        .status()
        .expect("Failed to compile C server");
    
    assert!(status.success(), "C compilation failed");

    // 2. Spawn C process
    let mut c_proc = Command::new(c_bin_file)
        .spawn()
        .expect("Failed to spawn C server");

    // Give the C server some time to start and bind to ZMQ
    thread::sleep(Duration::from_millis(500));

    // 3. Initialize Rust CSP instance (Address 1)
    let address = 1;
    let csp_instance = LibCspBuilder::new(LibCspConfig::new(address)).build();

    // Add ZMQ interface and route to C server (address 2)
    let zmq_if = CspZmqInterface::new_basic("localhost", 0);
    csp_instance.add_interface_route(Route::default_address(), zmq_if).unwrap();

    // 4. Connect to C server
    let client = csp_instance.client();
    let conn_result = client.connect(
        CspConnAddress::new(2, 10),
        CspConnPriority::Normal,
        Duration::from_secs(2),
    );

    match conn_result {
        Ok(connection) => {
            println!("Rust client connected to C server");
            connection
                .send_packet(Duration::from_secs(1), b"Hello from Rust!")
                .unwrap();
            
            // Wait for C process to exit
            let status = c_proc.wait().expect("C process failed");
            assert!(status.success(), "C process exited with error");
        }
        Err(e) => {
            c_proc.kill().ok();
            panic!("Failed to connect to C server: {:?}", e);
        }
    }
}
