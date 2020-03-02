// MAC provided by HCI tool
// sudo hcitool lescan
// C8:47:8C:D1:7F:DC MI SCALE2

extern crate btleplug;

use std::thread;
use std::time::Duration;

use btleplug::api::{BDAddr, Central, Peripheral, UUID};
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};

fn get_central(manager: &Manager) -> ConnectedAdapter {
    let adapters = manager.adapters().unwrap();
    let adapter = adapters.into_iter().nth(0).unwrap();
    adapter.connect().unwrap()
}

fn main() {
    let mac: BDAddr = BDAddr {
        address: [0xDC, 0x7F, 0xD1, 0x8C, 0x47, 0xC8]    };
    let manager = Manager::new().unwrap();
    let central = get_central(&manager);
    central.start_scan().unwrap();

    thread::sleep(Duration::from_secs(2));



    let scale = central
        .peripherals()
        .into_iter()
        .find(|p| p.address() == mac);

    match scale {
        Some(z) => {

            println!("found somethign: {:?}, connecting...", z);
            z.connect().unwrap();
            println!("....Connected");
            println!("Characteristics:  {:?}", z.discover_characteristics().unwrap());
        },
        None => println!("No Device found: {:?}", mac),
    };
    
}
