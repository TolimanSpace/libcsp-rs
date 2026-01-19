use libcsp::{
    CspConnAddress, CspConnPriority, CspDebugChannel, LibCspBuilder, LibCspConfig, LibCspInstance,
};

use std::{thread, time::Duration};

// Server port, the port the server listens on for incoming connections from the client.
const MY_SERVER_PORT: u16 = 10;

// Server task - handles requests from clients
fn server_task(instance: &LibCspInstance) {
    instance
        .server_sync_socket_builder()
        .unwrap()
        .bind_port(MY_SERVER_PORT as u8, |conn| {
            for packet in conn.iter_packets(Duration::from_secs(1)) {
                let data = String::from_utf8_lossy(packet.as_slice());
                println!("Packet received on MY_SERVER_PORT: {:?}", data);
            }
        })
        .run_sync();
}

// Client task sending requests to server task
fn client_task(instance: &LibCspInstance) {
    let address = 1u16;
    let client = instance.client();

    loop {
        // Simulate some workload or delay
        thread::sleep(Duration::from_millis(10));

        // Example: send a ping to the server
        // let result = client.ping(address).unwrap();
        // println!("Ping result: {}", result);

        let connection = client
            .connect(
                CspConnAddress::new(address, MY_SERVER_PORT as u8),
                CspConnPriority::Normal,
                Duration::from_secs(1),
            )
            .unwrap();

        connection
            .send_packet(b"Hello world from Rust")
            .unwrap();

        println!("Packet sent");
    }
}

fn main() {
    let address: u16 = 1; // Choose sensible defaults here

    let csp_instance = LibCspBuilder::new(LibCspConfig::new(address))
        .debug_channels(CspDebugChannel::up_to_info())
        .build();

    csp_instance.print_conn_table();
    csp_instance.print_iflist();
    csp_instance.print_rtable();

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
