use btleplug::api::{Manager as _, Central as _, CentralEvent, Peripheral};
use btleplug::platform::{Manager, Adapter};
use futures::StreamExt;
use tokio;
use env_logger;

mod airtag;

async fn must_start_scan(adapter: &Adapter) {
// if let Err(e) = adapter.start_scan(btleplug::api::ScanFilter { services: vec![airtag::constants::AIRTAG_SOUND_SERVICE] }).await {
if let Err(e) = adapter.start_scan(btleplug::api::ScanFilter::default()).await {
        panic!("Unable to start scan! {:?}", e);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {

    env_logger::init();

    let Ok(ble_manager) = Manager::new().await else { panic!("Unable to create BLE manager") };
    
    let ble_adapter: Adapter = match ble_manager.adapters().await {
        Ok(adapters) => match adapters.into_iter().nth(0) {
            Some(adapter) => adapter,
            None => panic!("No BLE adapter found!"),
        },
        Err(e) => panic!("Unable to get list of BLE adapters! {:?}", e),
    };

    let Ok(mut events) = ble_adapter.events().await else { panic!("Unable to register event handler") };

    // Note: As per btleplug docs, ScanFilter is platform dependent.
    // In some cases, empty ScanFilter may cause problems, but supplying a 
    // ScanFilter doesn't guarantee results would actually match the filter...
    // So we still need to have other logic to check device, services etc.
    // if let Err(e) = ble_adapter.start_scan(btleplug::api::ScanFilter { services: vec![airtag::constants::AIRTAG_SOUND_SERVICE] }).await {
    if let Err(e) = ble_adapter.start_scan(btleplug::api::ScanFilter::default()).await {
        panic!("Unable to start scan! {:?}", e);
    }

    log::info!("Starting scan...");

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::ManufacturerDataAdvertisement { id, manufacturer_data } => {
                log::debug!("ManufacturerDataAdvertisement: {:02X?}", manufacturer_data);
                if airtag::airtag::is_airtag(&manufacturer_data) == false {
                    continue;
                }

                // Found an airtag, stop current scan
                log::info!("Found airtag! Stopping scan...");

                // TBD Do we need to stop the scan really?
                if let Err(e) = ble_adapter.stop_scan().await {
                    log::error!("Failed to stop scan: {:?}", e);
                    continue;
                }

                // Get the peripheral
                let peripheral = match ble_adapter.peripheral(&id).await {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("Failed to get peripheral! {:?}", e);
                        must_start_scan(&ble_adapter).await;
                        continue;
                    }
                };

                // Connect to it
                if let Err(e) = peripheral.connect().await {
                    log::error!("Failed to connect to device! {:?}", e);
                    continue;
                }

                log::info!("Connected to device!");

                // Discover services
                if let Err(e) = peripheral.discover_services().await {
                    log::error!("Failed to discover services! {:?}", e);
                    must_start_scan(&ble_adapter).await;
                    continue;
                }

                // Loop over characteristics, try and find play sound characteristic
                for characteristic in peripheral.characteristics() {
                    if characteristic.uuid == airtag::constants::AIRTAG_SOUND_CHARACTERISTIC {
                        if let Err(e) = peripheral.write(&characteristic, &airtag::constants::AIRTAG_PLAY_SOUND, btleplug::api::WriteType::WithResponse).await {
                            log::error!("Failed to write characteristic! {:?}", e);
                            must_start_scan(&ble_adapter).await;
                            continue;
                        }

                        log::info!("Playing sound...");
                    }
                }

                // Success!
                if let Err(e) = peripheral.disconnect().await {
                    log::error!("Failed to disconnect! {:?}", e);
                    must_start_scan(&ble_adapter).await;
                    continue;
                }

                log::info!("Starting scan again");
                must_start_scan(&ble_adapter).await;
            },
            _ => (),
        }
    }
}
