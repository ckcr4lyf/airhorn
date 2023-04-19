use btleplug::api::{Manager as _, Central as _, CentralEvent};
use btleplug::platform::{Manager, Adapter};
use futures::StreamExt;

mod airtag;

const AIRTAG_SOUND_SERVICE: uuid::Uuid = uuid::uuid!("7dfc9000-7d1c-4951-86aa-8d9728f8d66c");

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
    if let Err(e) = ble_adapter.start_scan(btleplug::api::ScanFilter { services: vec![AIRTAG_SOUND_SERVICE] }).await {
        panic!("Unable to start scan! {:?}", e);
    }

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::ManufacturerDataAdvertisement { id, manufacturer_data } => {

                if airtag::airtag::is_airtag(&manufacturer_data) == false {
                    continue;
                }

                // Found an airtag, stop current scan
                if let Err(e) = ble_adapter.stop_scan().await {
                    println!("Failed to stop scan: {:?}", e);
                    continue;
                }

                // Try and connect to it
                let peripheral = match ble_adapter.peripheral(&id).await {
                    Ok(p) => p,
                    Err(e) => {
                        println!("Failed to get peripheral! {:?}", e);
                        continue;
                    }
                };

                




                
            }
            _ => println!("Got an event!"),
        }
    }
}
