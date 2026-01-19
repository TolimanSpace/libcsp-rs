use libcsp::{
    CspConnAddress, CspConnPriority, CspDebugChannel, LibCspBuilder, LibCspConfig, LibCspInstance,
};

use std::{
    io::{Read, Write},
    thread,
    time::Duration,
};

// Server port, the port the server listens on for incoming connections from the client.
const MY_SERVER_PORT: u16 = 10;

// Server task - handles requests from clients
fn server_task(instance: &LibCspInstance, stream: &[u8], buffer_sizes: &[usize]) {
    let mut buffer_sizes_index = 0;

    instance
        .server_sync_socket_builder()
        .unwrap()
        .bind_port(MY_SERVER_PORT as u8, |conn| {
            if buffer_sizes_index >= buffer_sizes.len() {
                println!("WTF");
            }

            let buffer_size = buffer_sizes[buffer_sizes_index];
            println!("Recieving with buffer size: {}", buffer_size);

            let mut reader = conn.into_reader(Duration::from_secs(1));
            let mut buffer = vec![0; buffer_size];

            let mut read_result = Vec::new();

            // Read until connection is closed
            loop {
                let bytes_read = reader.read(&mut buffer).unwrap();
                if bytes_read == 0 {
                    println!("Connection closed");
                    break;
                } else {
                    // println!("Read {} bytes", bytes_read);
                    read_result.extend_from_slice(&buffer[0..bytes_read]);
                }
            }

            // Check that the read result is the same as the stream
            assert_eq!(read_result.len(), stream.len());
            assert_eq!(read_result, stream);
            println!("Read result matches stream");

            buffer_sizes_index += 1;
            if buffer_sizes_index >= buffer_sizes.len() {
                println!("Sucess! All buffer sizes tested");
            }
        })
        .run_sync();
}

// Client task sending requests to server task
fn client_task(instance: &LibCspInstance, stream: &[u8], buffer_sizes: &[usize]) {
    let address = 1;
    let client = instance.client();

    for size in buffer_sizes {
        // Wait for a ping
        loop {
            let result = client.ping(address);
            println!("Ping result: {:?}", result);
            if result.is_ok() {
                break;
            }
        }

        let connection = client
            .connect(
                CspConnAddress::new(address, MY_SERVER_PORT as u8),
                CspConnPriority::Normal,
                Duration::from_secs(1),
            )
            .unwrap();

        let mut writer = connection.into_writer();
        for chunk in stream.chunks(*size) {
            writer.write_all(chunk).unwrap();
            thread::sleep(Duration::from_millis(10));
        }

        println!("Packet sent");
    }
}

fn main() {
    let address: u8 = 1; // Choose sensible defaults here

    let csp_instance = LibCspBuilder::new(
        LibCspConfig::new(address as u16),
    )
    .debug_channels(CspDebugChannel::up_to_info())
    .build();

    csp_instance.print_conn_table();
    csp_instance.print_iflist();
    csp_instance.print_rtable();

    // Make a binary stream with arbitrary bytes
    let mut stream = Vec::new();
    for i in 0..10000 {
        stream.push(i as u8);
    }

    // Choose the write sizes for each attempt
    let write_sizes = vec![100, 1000, 10000, 10000];

    thread::scope(|s| {
        // Start server and client tasks in separate threads
        s.spawn(|| {
            server_task(&csp_instance, &stream, &write_sizes);
        });

        s.spawn(|| {
            client_task(&csp_instance, &stream, &write_sizes);
        });
    });
}
