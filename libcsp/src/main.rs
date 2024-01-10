use libcsp::{CspDebugChannel, LibCspBuilder, LibCspConfig, LibCspInstance};
use libcsp_sys::*;
use std::{ptr, thread, time::Duration};

// Server port, the port the server listens on for incoming connections from the client.
const MY_SERVER_PORT: u16 = 10;

// Server task - handles requests from clients
unsafe fn server_task(instance: &LibCspInstance) {
    instance
        .server_socket_builder()
        .unwrap()
        .bind_port(MY_SERVER_PORT as u8, |conn| {
            for packet in conn.iter_packets() {
                let data = std::str::from_utf8_unchecked(packet);
                println!("Packet received on MY_SERVER_PORT: {:?}", data);
            }
        })
        .run();
}

// Client task sending requests to server task
unsafe fn client_task() {
    let address = 1;

    loop {
        // Simulate some workload or delay
        thread::sleep(Duration::from_millis(100));

        // Example: send a ping to the server
        let result = csp_ping(address, 1000, 100, CSP_O_NONE as u8);
        println!("Ping result: {}", result);

        // Example: send a packet to the server
        let conn: *mut csp_conn_t = csp_connect(
            csp_prio_t_CSP_PRIO_NORM as u8,
            address,
            MY_SERVER_PORT as u8,
            1000,
            CSP_O_NONE,
        );
        if conn.is_null() {
            // If connection failed, continue loop
            println!("Connection failed");
            continue;
        }

        let packet: *mut csp_packet_t = csp_buffer_get(100) as *mut csp_packet_t;
        if packet.is_null() {
            // If getting a packet buffer failed, continue loop
            println!("Failed to get CSP buffer");
            csp_close(conn);
            continue;
        }

        let msg = "Hello world from Rust";

        ptr::copy_nonoverlapping(
            msg.as_ptr(),
            &(*packet).__bindgen_anon_1.data as *const _ as *mut u8,
            msg.len(),
        );
        (*packet).length = msg.len() as u16;
        csp_send(conn, packet, 1000);
        csp_close(conn);
    }
}

// unsafe fn router_task() {
//     loop {
//         csp_route_work(1000);
//     }
// }

fn main() {
    let address: u8 = 1; // Choose sensible defaults here

    // let zmq_device = "localhost";
    // let zmq_device = std::ffi::CString::new(zmq_device).unwrap();

    let csp_instance = LibCspBuilder::new(LibCspConfig::new(address))
        .debug_channels(CspDebugChannel::up_to_info())
        .build();

    csp_instance.print_conn_table();
    csp_instance.print_iflist();
    csp_instance.print_rtable();

    thread::scope(|s| {
        // Start server and client tasks in separate threads
        s.spawn(|| unsafe {
            server_task(&csp_instance);
        });

        s.spawn(|| unsafe {
            client_task();
        });
    });
}
