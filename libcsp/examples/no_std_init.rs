#![no_std]
#![no_main]

use libcsp::{LibCspBuilder, LibCspConfig};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() -> i32 {
    // Initialize LibCSP with address 10
    let config = LibCspConfig::new(10);
    let csp_instance = LibCspBuilder::new(config).build();

    // In a no-std environment, we must manually drive the router
    loop {
        csp_instance.route_work();
        // Here you would also handle other tasks or sleep
    }
}
