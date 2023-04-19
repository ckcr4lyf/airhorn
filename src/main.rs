use btleplug::api::{Manager as _, Central as _, CentralEvent, Peripheral};
use btleplug::platform::{Manager, Adapter};
use futures::StreamExt;

mod airtag;

#[tokio::main(flavor = "current_thread")]
async fn main() {

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
    if let Err(e) = ble_adapter.start_scan(btleplug::api::ScanFilter { services: vec![airtag::constants::AIRTAG_SOUND_SERVICE] }).await {
        panic!("Unable to start scan! {:?}", e);
    }

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::ManufacturerDataAdvertisement { id, manufacturer_data } => {
                if airtag::airtag::is_airtag(&manufacturer_data) == false {
                    continue;
                }

                // Found an airtag, stop current scan
                // TBD Do we need to stop
                if let Err(e) = ble_adapter.stop_scan().await {
                    println!("Failed to stop scan: {:?}", e);
                    continue;
                }

                // Get the peripheral
                let peripheral = match ble_adapter.peripheral(&id).await {
                    Ok(p) => p,
                    Err(e) => {
                        println!("Failed to get peripheral! {:?}", e);
                        continue;
                    }
                };

                // Connect to it
                if let Err(e) = peripheral.connect().await {
                    println!("Failed to connect to device! {:?}", e);
                    continue;
                }

                // Discover services
                if let Err(e) = peripheral.discover_services().await {
                    println!("Failed to discover services! {:?}", e);
                    continue;
                }

                // Loop over characteristics, try and find play sound characteristic
                for characteristic in peripheral.characteristics() {
                    if characteristic.uuid == airtag::constants::AIRTAG_SOUND_CHARACTERISTIC {
                        if let Err(e) = peripheral.write(&characteristic, &airtag::constants::AIRTAG_PLAY_SOUND, btleplug::api::WriteType::WithResponse).await {
                            println!("Failed to write characteristic! {:?}", e);
                            continue;
                        }
                    }
                }

                // Success!                
            }
            _ => println!("Got an event!"),
        }
    }
}
