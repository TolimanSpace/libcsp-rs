use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use libcsp::{
    CspConnAddress, CspConnPriority, LibCspBuilder, LibCspConfig, CspPort, interface::CspZmqInterface, Route
};

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn test_c_server_interop() {
    // 1. Compile ZMQ Hub
    Command::new("gcc")
        .args(["tests/c_src/zmq_hub.c", "-o", "../target/zmq_hub", "-lzmq"])
        .status()
        .expect("Failed to compile ZMQ Hub");

    // 2. Compile C server
    Command::new("gcc")
        .args(["tests/c_src/simple_server.c", "-o", "../target/c_server", "-lcsp", "-lzmq"])
        .status()
        .expect("Failed to compile C server");

    // 3. Spawn ZMQ Hub
    let _zmq_hub = ChildGuard(Command::new("../target/zmq_hub").spawn().expect("Failed to spawn ZMQ Hub"));
    thread::sleep(Duration::from_secs(1));

    // 4. Spawn C process
    let mut c_proc = Command::new("../target/c_server").spawn().expect("Failed to spawn C server");
    thread::sleep(Duration::from_secs(1));

    // 5. Connect using Rust wrapper
    let address = 1;
    let server_address = 10;
    let server_port = 10;
    
    let csp_instance = LibCspBuilder::new(LibCspConfig::new(address)).build();

    // Default route to ZMQ
    csp_instance.add_interface_route(
        Route::default_address(),
        CspZmqInterface::WithEndpoints {
            publish_endpoint: "tcp://127.0.0.1:6000",
            subscribe_endpoint: "tcp://127.0.0.1:7000",
            zmq_flags: 0,
        }
    ).expect("Failed to add ZMQ interface");

    // Give ZMQ time to subscribe
    thread::sleep(Duration::from_secs(2));

    let client = csp_instance.client();
    let connection = client
        .connect(
            CspConnAddress::new(server_address, server_port),
            CspConnPriority::Normal,
            Duration::from_secs(5),
        )
        .expect("Failed to connect to C server");

    connection
        .send_packet(Duration::from_secs(1), b"Hello from Rust")
        .expect("Failed to send packet");

    // 6. Wait for C process to finish
    let status = c_proc.wait().expect("C process failed");
    assert!(status.success());
}
