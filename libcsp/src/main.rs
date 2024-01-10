use libcsp::{
    CspConnPriority, CspDebugChannel, CspZmqInterface, LibCspBuilder, LibCspConfig, LibCspInstance,
    Route,
};

use std::{thread, time::Duration};

// Server port, the port the server listens on for incoming connections from the client.
const MY_SERVER_PORT: u16 = 10;

// Server task - handles requests from clients
fn server_task(instance: &LibCspInstance) {
    instance
        .server_socket_builder()
        .unwrap()
        .bind_port(MY_SERVER_PORT as u8, |conn| {
            for packet in conn.iter_packets() {
                let data = String::from_utf8_lossy(packet);
                println!("Packet received on MY_SERVER_PORT: {:?}", data);
            }
        })
        .run();
}

// Client task sending requests to server task
unsafe fn client_task(instance: &LibCspInstance) {
    let address = 1;
    let client = instance.client();

    loop {
        // Simulate some workload or delay
        thread::sleep(Duration::from_millis(10));

        // Example: send a ping to the server
        // let result = client.ping(address).unwrap();
        // println!("Ping result: {}", result);

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

        // // Example: send a packet to the server
        // let conn: *mut csp_conn_t = csp_connect(
        //     csp_prio_t_CSP_PRIO_NORM as u8,
        //     address,
        //     MY_SERVER_PORT as u8,
        //     1000,
        //     CSP_O_NONE,
        // );
        // if conn.is_null() {
        //     // If connection failed, continue loop
        //     println!("Connection failed");
        //     continue;
        // }

        // let packet: *mut csp_packet_t = csp_buffer_get(256) as *mut csp_packet_t;
        // if packet.is_null() {
        //     // If getting a packet buffer failed, continue loop
        //     println!("Failed to get CSP buffer");
        //     csp_close(conn);
        //     continue;
        // }

        // let msg = "Hello world from Rust";

        // ptr::copy_nonoverlapping(
        //     msg.as_ptr(),
        //     &(*packet).__bindgen_anon_1.data as *const _ as *mut u8,
        //     msg.len(),
        // );
        // (*packet).length = msg.len() as u16;
        // csp_send(conn, packet, 1000);
        // csp_close(conn);
    }
}

fn main() {
    let address: u8 = 1; // Choose sensible defaults here

    // let zmq_device = "localhost";
    // let zmq_device = std::ffi::CString::new(zmq_device).unwrap();

    let csp_instance = LibCspBuilder::new(LibCspConfig::new(address))
        .debug_channels(CspDebugChannel::up_to_info())
        .build();

    csp_instance
        .add_interface_route(
            Route::default_address(),
            CspZmqInterface::new_basic("localhost", 0),
        )
        .unwrap();

    csp_instance.print_conn_table();
    csp_instance.print_iflist();
    csp_instance.print_rtable();

    thread::scope(|s| {
        // Start server and client tasks in separate threads
        s.spawn(|| unsafe {
            server_task(&csp_instance);
        });

        s.spawn(|| unsafe {
            client_task(&csp_instance);
        });
    });
}
