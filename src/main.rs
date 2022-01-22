// MAC provided by HCI tool
// sudo hcitool lescan
// C8:47:8C:D1:7F:DC MI SCALE2
// https://github.com/oliexdev/openScale/wiki/Xiaomi-Bluetooth-Mi-Scale

//extern crate btleplug;

use std::convert::TryFrom;
use std::vec;
use std::{error::Error, str::FromStr};
use btleplug::api::bleuuid::BleUuid;
use log::{info, error, debug, trace};
use time::{OffsetDateTime, UtcOffset};

use btleplug::api::{bleuuid, Central, Manager as _, Peripheral as _, BDAddr, ScanFilter, CentralEvent};
use btleplug::platform::{Adapter, Manager, Peripheral};
use dotenv;
use simple_logger::SimpleLogger;
use influx_db_client::{Client, Point, Value, point};
use tokio_stream::StreamExt;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().with_utc_timestamps().init().unwrap();
    let mac_str = dotenv::var("ADDRESS").unwrap();
    let mac = BDAddr::from_str(&mac_str).unwrap();
    println!("Configured scale address: {:?}", mac_str);
    let manager = Manager::new().await.unwrap();
    let central = get_central(&manager).await;
    let mut events = central.events().await?;

    // let scale = central.peripheral(mac);
    // start scanning for devices
    let mut uuid = uuid::Uuid::parse_str("0000181d-0000-1000-8000-00805f9b34fb").unwrap(); 
    let mut v = vec![uuid];
    let mut filter = ScanFilter::default();
    filter.services.append(&mut v);
    central.start_scan(filter).await.unwrap();

    // Print based on whatever the event receiver outputs. Note that the event
    // receiver blocks, so in a real program, this should be run in its own
    // thread (not task, as this library does not yet use async channels).
    while let Some(event) = events.next().await {
        match event {
            CentralEvent::ServiceDataAdvertisement {
                id,
                service_data,
            } => {
                // if mac == address {

                    //service data advertisement id: PeripheralId(DeviceId { object_path: Path("/org/bluez/hci0/dev_C8_47_8C_D1_7F_DC\u{0}") }) data: [130, 232, 3, 178, 7, 1, 1, 22, 8, 45]
                    // let w: u16 = (service_data[2]) << 8 | service_data[1];
                    // log::debug!("Weight: {:?} kg", (w as f32 / 200.00));
                    log::debug!("service data advertisement id: {:?} data: {:?}",id,  service_data.values());
                // }
            },
            CentralEvent::DeviceDiscovered(id) => {log::debug!("Device Discovered: {:?}", id)},
            CentralEvent::ManufacturerDataAdvertisement{id, manufacturer_data } => {log::debug!("id: {:?}  manu data: {:?}", id, manufacturer_data)}
            _ => { log::debug!("Non handled event") }
        }
    }
    // match scale {
    //     Some(scale) => {

    //         println!("found something: {:?}, connecting...", scale);
    //         scale.connect().unwrap();
    //         println!("....Connected");
    //         let characteristics = scale.discover_characteristics().unwrap();

    //         let characteristic = characteristics.iter().find(|c| c.uuid == UUID::B16(0x2a9d));
    //         match scale.subscribe(&characteristic.unwrap()){
    //             Ok(_) => println!("Subscribed"),
    //             Err(_) => println!("During subscription something went wrong")
    //         }
    //         let time_characteristic = characteristics.iter().find(|c| c.uuid == UUID::B16(0x2A2B));
    //         let time = scale.read(&time_characteristic.unwrap());
    //         match time {
    //             Ok(time) => {parse_time(&time)},
    //             Err(_) => { println!("Can't read time characteristics") }
    //         }
    //         let weight_history_characteristic = characteristics.iter().find(|c| c.uuid == UUID::B128([0x00, 0x00, 0x2A, 0x2F, 0x00, 0x00, 0x35, 0x12, 0x21, 0x18, 0x00, 0x09,0xAF, 0x10, 0x07, 0x00]));
    //         match weight_history_characteristic {
    //             Some(weight_history) => {
    //                 scale.subscribe(weight_history).unwrap();
    //                 let cmd: [u8;5]  = [0x01, 0xFF, 0xFF, 0xFF, 0xFF];
    //                 match  scale.command(weight_history,&cmd){
    //                     Ok(_) => {println!("Successfully send history weight command {:?}", cmd)},
    //                     Err(e) => {println!("Cannot send weight history command: {:?}. Error: {:?}", cmd,e)}
    //                 }
    //             },
    //             None => {println!("Cannot find weight history characteristics")}
    //         }
    //     },

    //     None => println!("No Device found: {:?}", mac),
    // };
    Ok(())
}

fn store_weight(weight: f32) -> (){
    let user = dotenv::var("INFLUXDB_USERNAME").unwrap();
    let passwd = dotenv::var("INFLUXDB_PASSWORD").unwrap();
    let client = Client::default().set_authentication(user, passwd);
    let point =  point!("ddd"); 
    // Point::new("weight").add_field(field, value)
    //client.write_point(point);
    ()
}

fn handler() {
    println!("Got Notification");
}

fn parse_time(time: &Vec<u8>) {
    println!("time vec :{:?}", time)
}
