use btleplug::api::{Manager as _, Central as _};
use btleplug::platform::{Manager, Adapter};
use futures::StreamExt;

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

    if let Err(e) = ble_adapter.start_scan(btleplug::api::ScanFilter { services: vec![AIRTAG_SOUND_SERVICE] }).await {
        panic!("Unable to start scan! {:?}", e);
    }

    while let Some(event) = events.next().await {
        match event {
            _ => println!("Got an event!"),
        }
    }
}
