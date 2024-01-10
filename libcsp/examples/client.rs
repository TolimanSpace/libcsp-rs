use libcsp::{
    CspConnPriority, CspDebugChannel, CspZmqInterface, LibCspBuilder, LibCspConfig, LibCspInstance,
    Route,
};

use std::{thread, time::Duration};

// Server port, the port the server listens on for incoming connections from the client.
const MY_SERVER_PORT: u16 = 10;

// Client task sending requests to server task
fn client_task(instance: &LibCspInstance) {
    let address = 27;
    let client = instance.client();

    loop {
        // Simulate some workload or delay
        thread::sleep(Duration::from_millis(10));

        // Example: send a ping to the server
        // let result = client.ping(address);
        // println!("Ping result: {:?}", result);

        let connection = client
            .connect(
                address,
                CspConnPriority::Normal,
                MY_SERVER_PORT as u8,
                Duration::from_secs(1),
            )
            .unwrap();

        connection
            .send_packet(Duration::from_secs(1), b"Hello world from Rust")
            .unwrap();

        println!("Packet sent");
    }
}

fn main() {
    let address: u8 = 1;

    let zmq_device = "localhost";

    let csp_instance = LibCspBuilder::new(LibCspConfig::new(address))
        .debug_channels(CspDebugChannel::up_to_info())
        .build();

    csp_instance
        .add_interface_route(
            Route::default_address(),
            CspZmqInterface::new_basic(zmq_device, 0),
        )
        .unwrap();

    csp_instance.print_conn_table();
    csp_instance.print_iflist();
    csp_instance.print_rtable();

    client_task(&csp_instance);
}
