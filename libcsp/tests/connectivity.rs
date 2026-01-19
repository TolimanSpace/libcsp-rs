use libcsp::{
    CspConnAddress, CspConnPriority, LibCspBuilder, LibCspConfig, CspPort
};
use std::{thread, time::Duration};

#[test]
fn test_loopback_connectivity() {
    let address = 1;
    let port = 10;
    let csp_instance = LibCspBuilder::new(LibCspConfig::new(address)).build();

    thread::scope(|s| {
        // Server
        s.spawn(|| {
            let socket = csp_instance.open_server_socket(CspPort::port(port)).unwrap();
            if let Some(conn) = socket.accept_timeout(Duration::from_secs(2)) {
                if let Some(packet) = conn.iter_packets(Duration::from_secs(1)).next() {
                    let data = String::from_utf8_lossy(packet.as_slice());
                    let data = data.trim_end_matches('\0');
                    assert_eq!(data, "Hello from test");
                } else {
                    panic!("No packet received");
                }
            } else {
                panic!("No connection received");
            }
        });

        // Client
        s.spawn(|| {
            thread::sleep(Duration::from_millis(100));
            let client = csp_instance.client();
            let connection = client
                .connect(
                    CspConnAddress::new(address, port),
                    CspConnPriority::Normal,
                    Duration::from_secs(1),
                )
                .unwrap();

            connection
                .send_packet(b"Hello from test")
                .unwrap();
        });
    });
}
