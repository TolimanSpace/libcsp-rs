use libcsp_sys::*;
use std::{ptr, thread, time::Duration};

// Server port, the port the server listens on for incoming connections from the client.
const MY_SERVER_PORT: u16 = 10;

// Address to use for the server in this example
const SERVER_ADDRESS: u8 = 0;

// Server task - handles requests from clients
fn server_task() {
    loop {
        // Create socket and listen for connections
        let mut socket: csp_socket_t = unsafe { std::mem::zeroed() };
        unsafe {
            csp_bind(&mut socket, CSP_ANY as u8);
            csp_listen(&mut socket, 10);
        }

        // Wait for connections and process packets
        loop {
            let conn: *mut csp_conn_t = unsafe { csp_accept(&mut socket, 10000) };
            if conn.is_null() {
                // If there's no connection, just continue the loop after a small delay
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            // Process packets on the connection
            loop {
                let packet = unsafe { csp_read(conn, 50) };
                if packet.is_null() {
                    break;
                }

                let dport = unsafe { csp_conn_dport(conn) };

                if dport == MY_SERVER_PORT as i32 {
                    let data = unsafe { (*packet).__bindgen_anon_2.data };
                    let length = unsafe { (*packet).length };
                    // Convert to string
                    let data = unsafe { std::str::from_utf8_unchecked(&data[0..length as usize]) };

                    println!("Packet received on MY_SERVER_PORT: {:?}", data);

                    // Free the packet buffer
                    unsafe { csp_buffer_free(packet as *mut std::os::raw::c_void) };
                } else {
                    // Call the default CSP service handler
                    unsafe { csp_service_handler(packet) };
                }
            }

            // Close the connection
            unsafe { csp_close(conn) };
        }
    }
}

// Client task sending requests to server task
fn client_task() {
    loop {
        // Simulate some workload or delay
        thread::sleep(Duration::from_millis(1000));

        // Example: send a ping to the server
        let result = unsafe { csp_ping(SERVER_ADDRESS.into(), 1000, 100, CSP_O_NONE as u8) };
        println!("Ping result: {}", result);

        // Example: send a packet to the server
        let conn: *mut csp_conn_t = unsafe {
            csp_connect(
                csp_prio_t_CSP_PRIO_NORM as u8,
                SERVER_ADDRESS.into(),
                MY_SERVER_PORT as u8,
                1000,
                CSP_O_NONE,
            )
        };
        if conn.is_null() {
            // If connection failed, continue loop
            println!("Connection failed");
            continue;
        }

        let packet: *mut csp_packet_t = unsafe { csp_buffer_get(100) };
        if packet.is_null() {
            // If getting a packet buffer failed, continue loop
            println!("Failed to get CSP buffer");
            unsafe { csp_close(conn) };
            continue;
        }

        let msg = "Hello world from Rust";
        unsafe {
            ptr::copy_nonoverlapping(
                msg.as_ptr(),
                (*packet).__bindgen_anon_2.data.as_mut_ptr(),
                msg.len(),
            );
            (*packet).length = msg.len() as u16;
            csp_send(conn, packet);
            csp_close(conn);
        }
    }
}

fn router_task() {
    loop {
        unsafe {
            csp_route_work();
        }
    }
}

fn main() {
    let address: u8 = 0; // Choose sensible defaults here
    let zmq_device = "localhost";
    let zmq_device = std::ffi::CString::new(zmq_device).unwrap();

    unsafe {
        csp_init();

        let mut int = std::ptr::null_mut();

        // Initialize ZMQ interface
        let error = csp_zmqhub_init(
            address as u16,
            zmq_device.as_ptr() as *const i8,
            0,
            &mut int,
        );
        if error != 0 {
            eprintln!(
                "Failed to add ZMQ interface [{}], error: {}",
                zmq_device.to_str().unwrap(),
                error
            );
            std::process::exit(1);
        }
        (*int).is_default = 1;

        csp_rtable_set(0, 0, int, libcsp_sys::CSP_NO_VIA_ADDRESS as u16);

        // Print connection table, interfaces, and route table
        csp_conn_print_table();
        csp_iflist_print();
        csp_rtable_print();
    }

    // Start server and client tasks in separate threads
    let server_handle = thread::spawn(|| {
        server_task();
    });

    let client_handle = thread::spawn(|| {
        client_task();
    });

    let router_handle = thread::spawn(|| {
        router_task();
    });

    // Here we just join on the server and client threads
    // In a real application, you would handle threads differently
    let _ = server_handle.join();
    let _ = client_handle.join();
    let _ = router_handle.join();
}
