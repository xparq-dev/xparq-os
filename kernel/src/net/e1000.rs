// XPARQ OS - Phase 4: Device Drivers Integration
// E1000 Networking Wrapper

use xparq_hal as hal;

pub fn init() {
    // Enable the E1000 driver through HAL
    let mut e1000 = hal::x86_64::e1000::E1000_DRIVER.lock();
    use hal::connectivity::ConnectivityDriver;
    let _ = e1000.set_enabled(true);
    
    let mac = e1000.get_info().mac_address;
    // Here we could register the MAC with the upper networking stack
    let _ = mac; // Silences unused warning for now
}
