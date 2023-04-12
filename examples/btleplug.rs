use btleplug::api::Peripheral;
use btleplug::api::{bleuuid::BleUuid, Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;

const AIRTAG_PREFIX: [u8; 3] = [0x12, 0x19, 0x10];

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

const play_sound_characteristic: uuid::Uuid = uuid::uuid!("7dfc9001-7d1c-4951-86aa-8d9728f8d66c");

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {

    let manager = Manager::new().await?;

    // get the first bluetooth adapter
    // connect to the adapter
    let central = get_central(&manager).await;

    // Each adapter has an event stream, we fetch via events(),
    // simplifying the type, this will return what is essentially a
    // Future<Result<Stream<Item=CentralEvent>>>.
    let mut events = central.events().await?;

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;

    // Print based on whatever the event receiver outputs. Note that the event
    // receiver blocks, so in a real program, this should be run in its own
    // thread (not task, as this library does not yet use async channels).
    while let Some(event) = events.next().await {
        match event {
            // CentralEvent::DeviceDiscovered(id) => {
            //     println!("DeviceDiscovered: {:?}", id);
            // }
            // CentralEvent::DeviceConnected(id) => {
            //     println!("DeviceConnected: {:?}", id);
            // }
            // CentralEvent::DeviceDisconnected(id) => {
            //     println!("DeviceDisconnected: {:?}", id);
            // }
            CentralEvent::ManufacturerDataAdvertisement { manufacturer_data, id } => {
                println!("ManufacturerDataAdvertisement: {:02X?}", manufacturer_data);
                if let Some(apple_data) = manufacturer_data.get(&0x004C) {

                    if apple_data[0..3].eq(&AIRTAG_PREFIX) {
                    println!("ManufacturerDataAdvertisement: {:02X?}", apple_data);
                    println!("Found an airtag");

                        central.stop_scan().await?;

                        // Try and connect
                        if let Ok(p) = central.peripheral(&id).await {
                            println!("got peri");
                            if let Ok(_) = p.connect().await {
                                println!("connected");
                                p.discover_services().await;
                                println!("got services");

                                for c in p.characteristics() {
                                    println!("Checking characteristic {:?}", c);
                                    if c.uuid == play_sound_characteristic {
                                        println!("found sound char!");
                                        p.write(&c, &[0xAF], btleplug::api::WriteType::WithResponse).await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // CentralEvent::ServiceDataAdvertisement { id, service_data } => {
            //     println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
            // }
            // CentralEvent::ServicesAdvertisement { id, services } => {
            //     let services: Vec<String> =
            //         services.into_iter().map(|s| s.to_short_string()).collect();
            //     println!("ServicesAdvertisement: {:?}, {:?}", id, services);
            // }
            _ => {}
        }
    }
    Ok(())
}