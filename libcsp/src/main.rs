use libcsp_sys::*;
use std::{os::raw::c_void, ptr, thread, time::Duration};

// Server port, the port the server listens on for incoming connections from the client.
const MY_SERVER_PORT: u16 = 10;

// Address to use for the server in this example
const SERVER_ADDRESS: u8 = 255;

// Server task - handles requests from clients
unsafe fn server_task() {
    loop {
        // Create socket and listen for connections
        let socket = csp_socket(CSP_SO_NONE);
        csp_bind(socket, CSP_ANY as u8);
        csp_listen(socket, 10);

        println!("Server listening on port {}", MY_SERVER_PORT);

        // Wait for connections and process packets
        loop {
            let conn = csp_accept(socket, 1000);
            if conn.is_null() {
                // If there's no connection, just continue the loop
                continue;
            }

            println!(
                "connection: source={}:{} dest={}:{}",
                csp_conn_src(conn),
                csp_conn_sport(conn),
                csp_conn_dst(conn),
                csp_conn_dport(conn)
            );

            // Process packets on the connection
            loop {
                let packet = csp_read(conn, 100);
                if packet.is_null() {
                    break;
                }

                let dport = csp_conn_dport(conn);

                if dport == MY_SERVER_PORT as i32 {
                    let data = &(*packet).__bindgen_anon_1.data as *const _ as *const u8;
                    let length = (*packet).length;
                    let slice = std::slice::from_raw_parts(data, length as usize);

                    // Convert to string
                    let data = std::str::from_utf8_unchecked(slice);

                    println!("Packet received on MY_SERVER_PORT: {:?}", data);

                    // Free the packet buffer
                    unsafe { csp_buffer_free(packet as *mut std::os::raw::c_void) };
                } else {
                    // Call the default CSP service handler
                    unsafe { csp_service_handler(conn, packet) };
                }
            }

            // Close the connection
            unsafe { csp_close(conn) };
        }
    }
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
    let zmq_device = "localhost";
    let zmq_device = std::ffi::CString::new(zmq_device).unwrap();

    unsafe {
        for i in 0..3 {
            csp_debug_set_level(i, true);
        }

        let mut csp_conf = csp_conf_get_defaults();
        csp_conf.address = address;
        let error = csp_init(&csp_conf);
        if error != 0 {
            eprintln!("Failed to initialize CSP, error: {}", error);
            std::process::exit(1);
        }

        csp_route_start_task(500, 0);

        // let mut default_iface = std::ptr::null_mut();

        // Initialize ZMQ interface
        // let error = csp_zmqhub_init(
        //     csp_get_address(),
        //     zmq_device.as_ptr() as *const i8,
        //     0,
        //     &mut default_iface,
        // );
        // if error != 0 {
        //     eprintln!(
        //         "Failed to add ZMQ interface [{}], error: {}",
        //         zmq_device.to_str().unwrap(),
        //         error
        //     );
        //     std::process::exit(1);
        // }

        // let error = csp_rtable_set(0, 0, default_iface, libcsp_sys::CSP_NO_VIA_ADDRESS as u8);
        // if error != 0 {
        //     eprintln!("Failed to add route, error: {}", error);
        //     std::process::exit(1);
        // }

        // Print connection table, interfaces, and route table
        csp_conn_print_table();
        csp_iflist_print();
        csp_rtable_print();
    }

    // Start server and client tasks in separate threads
    let server_handle = thread::Builder::new()
        .name("SERVER".to_string())
        .spawn(|| unsafe {
            server_task();
        })
        .unwrap();

    let client_handle = thread::Builder::new()
        .name("CLIENT".to_string())
        .spawn(|| unsafe {
            client_task();
        })
        .unwrap();

    // let router_handle = thread::spawn(|| unsafe {
    //     router_task();
    // });

    // Here we just join on the server and client threads
    // In a real application, you would handle threads differently
    let _ = server_handle.join();
    let _ = client_handle.join();
    // let _ = router_handle.join();
}
