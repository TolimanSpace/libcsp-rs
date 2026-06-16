#[cfg(feature = "socketcan")]
use libcsp::{
    interface::CspCanInterface, CspConnAddress, CspConnPriority, CspDebugChannel, CspPort,
    LibCspBuilder, LibCspConfig, LibCspInstance, Route,
};

#[cfg(feature = "socketcan")]
use std::{thread, time::Duration};

// Server port, the port the server listens on for incoming connections from the client.
#[cfg(feature = "socketcan")]
const MY_SERVER_PORT: u16 = 10;

// Server task - handles requests from clients
#[cfg(feature = "socketcan")]
fn server_task(instance: &LibCspInstance) {
    println!("Server task started. Listening on port {}...", MY_SERVER_PORT);
    let socket = instance
        .open_server_socket(CspPort::port(MY_SERVER_PORT as u8))
        .unwrap();

    loop {
        if let Some(conn) = socket.accept_timeout(Duration::from_secs(2)) {
            println!("Connection accepted!");
            for packet in conn.iter_packets(Duration::from_secs(1)) {
                let data = String::from_utf8_lossy(packet.as_slice());
                let data = data.trim_end_matches('\0');
                println!("Packet received on MY_SERVER_PORT: {:?}", data);
            }
        }
    }
}

// Client task sending requests to server task
#[cfg(feature = "socketcan")]
fn client_task(instance: &LibCspInstance) {
    let address = 1u16;
    let client = instance.client();

    loop {
        // Wait for server to start and for some time between packets
        thread::sleep(Duration::from_millis(1000));

        println!("Connecting to address {} on port {}...", address, MY_SERVER_PORT);
        let connection = client.connect(
            CspConnAddress::new(address, MY_SERVER_PORT as u8),
            CspConnPriority::Normal,
            Duration::from_secs(1),
        );

        match connection {
            Ok(conn) => {
                println!("Connected! Sending packet...");
                if let Err(e) = conn.send_packet(b"Hello world from Rust CAN") {
                    println!("Failed to send packet: {:?}", e);
                } else {
                    println!("Packet sent successfully");
                }
            }
            Err(e) => {
                println!("Failed to connect: {:?}", e);
            }
        }
    }
}

fn main() {
    #[cfg(feature = "socketcan")]
    {
        let address: u16 = 1;

        println!("Initializing LibCSP with address {}...", address);
        let csp_instance = LibCspBuilder::new(LibCspConfig::new(address))
            .debug_channels(CspDebugChannel::up_to_info())
            .build();

        // Add CAN interface and route traffic for address 1 via CAN
        // This will override the default loopback route for address 1
        println!("Adding CAN interface vcan0 and routing address {} through it...", address);
        let can_if = CspCanInterface::new("vcan0", 1000000, false);
        csp_instance
            .add_interface_route(Route::new(address), can_if)
            .expect("Failed to add CAN route");

        println!("Interface list:");
        csp_instance.print_iflist();
        println!("Routing table:");
        csp_instance.print_rtable();

        println!("Starting server and client tasks...");
        thread::scope(|s| {
            // Start server and client tasks in separate threads
            s.spawn(|| {
                server_task(&csp_instance);
            });

            s.spawn(|| {
                client_task(&csp_instance);
            });
        });
    }

    #[cfg(not(feature = "socketcan"))]
    {
        println!("This example requires the 'socketcan' feature.");
        println!("Run with: cargo run --example can_loopback --features socketcan");
    }
}
