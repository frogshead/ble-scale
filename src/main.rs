mod config;
mod influxdb;
mod scale;

use std::{error::Error, str::FromStr, time::Duration};

use btleplug::api::{BDAddr, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Manager, Peripheral};
use log::{error, info, warn};
use simple_logger::SimpleLogger;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use uuid::Uuid;

use config::Config;
use influxdb::InfluxClient;
use scale::{parse_advertisement, SessionAction, SessionTracker};

const BODY_COMPOSITION_UUID: &str = "0000181d-0000-1000-8000-00805f9b34fb";
const CURRENT_TIME_UUID: &str = "00002a2b-0000-1000-8000-00805f9b34fb";
const SESSION_SILENCE_SECS: u64 = 30;
const TICK_INTERVAL_SECS: u64 = 5;
const RESTART_DELAY_SECS: u64 = 30;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().with_utc_timestamps().init()?;
    dotenv::dotenv().ok();

    let config = Config::from_env()?;
    info!("Configured scale address: {}", config.scale_address);

    let influx = InfluxClient::new(&config);

    loop {
        match run_scan_loop(&config, &influx).await {
            Ok(()) => break,
            Err(e) => {
                error!(
                    "Scan loop failed: {}. Restarting in {}s...",
                    e, RESTART_DELAY_SECS
                );
                sleep(Duration::from_secs(RESTART_DELAY_SECS)).await;
            }
        }
    }

    Ok(())
}

async fn set_scale_time(peripheral: Peripheral) -> Result<(), Box<dyn Error + Send + Sync>> {
    peripheral.connect().await?;

    let char_uuid = Uuid::from_str(CURRENT_TIME_UUID)?;
    let chars = peripheral.characteristics();
    let current_time_char = chars
        .iter()
        .find(|c| c.uuid == char_uuid)
        .ok_or("Current Time characteristic (0x2A2B) not found")?
        .clone();

    let now = time::OffsetDateTime::now_utc();
    let year = now.year() as u16;
    let payload = [
        year.to_le_bytes()[0],
        year.to_le_bytes()[1],
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        now.weekday().number_from_monday(),
        0, // fractions256
        0, // adjust reason
    ];

    peripheral
        .write(&current_time_char, &payload, WriteType::WithResponse)
        .await?;
    peripheral.disconnect().await?;
    info!("Scale clock synced to {}", now);
    Ok(())
}

async fn run_scan_loop(config: &Config, influx: &InfluxClient) -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters
        .into_iter()
        .next()
        .ok_or("No Bluetooth adapter found")?;

    let mut events = central.events().await?;

    let service_uuid = Uuid::from_str(BODY_COMPOSITION_UUID)?;
    let filter = ScanFilter {
        services: vec![service_uuid],
    };
    central.start_scan(filter).await?;
    info!("BLE scan started, waiting for MI Scale 2 advertisements...");

    let _scale_addr = BDAddr::from_str(&config.scale_address)?;

    let mut tracker = SessionTracker::new(Duration::from_secs(SESSION_SILENCE_SECS));
    let mut tick_interval = tokio::time::interval(Duration::from_secs(TICK_INTERVAL_SECS));
    let mut time_synced = false;

    loop {
        tokio::select! {
            event = events.next() => {
                let Some(event) = event else { break };
                if let CentralEvent::ServiceDataAdvertisement { id, service_data } = event {
                    let Some(data) = service_data.get(&service_uuid) else { continue };
                    let Some(adv) = parse_advertisement(data) else { continue };
                    if !time_synced {
                        time_synced = true;
                        if let Ok(peripheral) = central.peripheral(&id).await {
                            tokio::spawn(async move {
                                if let Err(e) = set_scale_time(peripheral).await {
                                    warn!("Time sync failed: {}", e);
                                }
                            });
                        }
                    }
                    match tracker.process(adv) {
                        SessionAction::Store(adv) => {
                            info!("Storing weight: {:.2} kg", adv.weight_kg);
                            if let Err(e) = influx.write_weight(&adv).await {
                                warn!("InfluxDB write failed: {}", e);
                            }
                        }
                        SessionAction::SessionEnded | SessionAction::Ignore => {}
                    }
                }
            }
            _ = tick_interval.tick() => {
                if let SessionAction::SessionEnded = tracker.tick() {
                    info!("Weighing session ended, tracker reset");
                    time_synced = false;
                }
            }
        }
    }

    Ok(())
}
