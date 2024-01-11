use std::time::Duration;

use libcsp::{
    interface::CspZmqInterface, CspDebugChannel, LibCspBuilder, LibCspConfig, LibCspInstance, Route,
};

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

fn main() {
    let address: u8 = 27;

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

    server_task(&csp_instance);
}
