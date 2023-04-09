//! Discover Bluetooth devices and list them.

use bluer::{Adapter, AdapterEvent, Address, DeviceEvent, Device};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::{collections::HashSet, env};
use tokio;

const AIRTAG_PREFIX: [u8; 3] = [0x12, 0x19, 0x10];

// static mut LOCK: bool = false;

async fn device_is_airtag(device: &Device) -> bool {
    // Get the Manufacturer Data
    let Ok(Some(manufacturer_data)) = device.manufacturer_data().await else { return false };

    // Get RSSI data to make sure it is available
    let Ok(Some(rssi_data)) = device.manufacturer_data().await else { return false };

    // Check if it's apple
    let Some(apple_data) = manufacturer_data.get(&0x0004C) else { return false };

    // If it is an airtag, we expect the first three bytes to be [0x12, 0x19, 0x10]
    let f3 = &apple_data[0..3];
    println!("Got f3 as {:02X?}", f3);
    if f3.eq(&AIRTAG_PREFIX) {
        return true;
    }

    return false;
}

async fn query_device(adapter: &Adapter, addr: Address, count: &mut i32) -> bluer::Result<()> {
    let device = adapter.device(addr)?;

    if device_is_airtag(&device).await {
        println!("Device added: {addr}");
        // println!("    Address type:       {}", device.address_type().await?);
        // println!("    Name:               {:?}", device.name().await?);
        // println!("    Icon:               {:?}", device.icon().await?);
        // println!("    Class:              {:?}", device.class().await?);
        // println!("    UUIDs:              {:?}", device.uuids().await?.unwrap_or_default());
        // println!("    Paired:             {:?}", device.is_paired().await?);
        // println!("    Connected:          {:?}", device.is_connected().await?);
        // println!("    Trusted:            {:?}", device.is_trusted().await?);
        // println!("    Modalias:           {:?}", device.modalias().await?);
        // println!("    RSSI:               {:?}", device.rssi().await?);
        // println!("    TX power:           {:?}", device.tx_power().await?);
        // println!("    Manufacturer data:  {:02X?}", device.manufacturer_data().await?);
        // println!("    Service data:       {:?}", device.service_data().await?);

        // if LOCK == true {
        //     println!("FOUND AIRTAG! Gonna try and connect");
        // }

        *count += 1;
        if *count != 1 {
            println!("Count is nonzero, wont connect");
            // *count -= 1;
            return Ok(());
        }

        println!("FOUND AIRTAG! Gonna try and connect");

        match device.connect().await {
            Ok(_) => {
                println!("connected");

                if let Ok(s) = device.services().await {
                    // println!("Got services {:?}", s);
                    
                    for svc in s {
                        if svc.id() != 0x5D {
                            println!("Skipping svc id = {:02X?}", svc.id());
                            continue;
                        }

                        println!("Gonna check out svc {:?}, ID = {:02X?}", svc, svc.id());

                        if let Ok(c) = svc.characteristics().await {
                            for ch in c {
                                if ch.id() != 0x5E {
                                    println!("Skipping chr id = {:02X?}", ch.id());
                                    continue;
                                }

                                println!("Found characteristic {:?}, ID = {:02X?}", ch, ch.id());
                                println!("Writing to {:?}", ch);
                                
                                match ch.write_ext(&[0xAF], &bluer::gatt::remote::CharacteristicWriteRequest{
                                    offset: 0,
                                    op_type: bluer::gatt::WriteOp::Request,
                                    prepare_authorize: false,
                                    _non_exhaustive: (),

                                }).await {
                                    Ok(_) => println!("WROTE IT"),
                                    Err(e) => println!("Failed to write: {:?}", e),
                                }
                                
                            }
                        }
                    }
                } else {
                    println!("Fail to get svc");
                }
            },
            Err(e) => {
                println!("fail to connect {:?}", e)
            }
        }

        // *count -= 1;
    }

    Ok(())
}

async fn query_all_device_properties(adapter: &Adapter, addr: Address) -> bluer::Result<()> {
    // let device = adapter.device(addr)?;
    // let props = device.all_properties().await?;
    // for prop in props {
    //     println!("    {:?}", &prop);
    // }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    let with_changes = env::args().any(|arg| arg == "--changes");
    let all_properties = env::args().any(|arg| arg == "--all-properties");
    let filter_addr: HashSet<_> = env::args()
        .filter_map(|arg| arg.parse::<Address>().ok())
        .collect();

    // env_logger::init();
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    println!(
        "Discovering devices using Bluetooth adapter {}\n",
        adapter.name()
    );
    adapter.set_powered(true).await?;

    let device_events = adapter.discover_devices_with_changes().await?;
    pin_mut!(device_events);

    let mut all_change_events = SelectAll::new();
    let mut count = 0;

    loop {
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        if !filter_addr.is_empty() && !filter_addr.contains(&addr) {
                            continue;
                        }

                        // println!("Device added: {addr}");
                        let res = if all_properties {
                            query_all_device_properties(&adapter, addr).await
                        } else {
                            query_device(&adapter, addr, &mut count).await
                        };
                        if let Err(err) = res {
                            println!("    Error: {}", &err);
                        }

                        if with_changes {
                            let device = adapter.device(addr)?;
                            let change_events = device.events().await?.map(move |evt| (addr, evt));
                            all_change_events.push(change_events);
                        }
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
                        println!("Device removed: {addr}");
                    }
                    _ => (),
                }
                // println!();
            },
            Some((addr, DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {


                

                
                

                let device = adapter.device(addr)?;

                if device_is_airtag(&device).await {
                    println!("Device changed: {addr}");
                    println!("    {property:?}");
                    // property.get
                }

                // if let (Ok(m_data), Ok(rssi_data)) = (device.manufacturer_data().await, device.rssi().await) {
                //     if let (Some(m_d), Some(r_d)) = (m_data, rssi_data) {
                //         if m_d.contains_key(&0x004c) {
            
                //             println!("Device changed: {addr}");
                //             println!("    {property:?}");
                //             // println!("Device added: {addr}");
                //             // println!("    Address type:       {}", device.address_type().await?);
                //             // println!("    Name:               {:?}", device.name().await?);
                //             // println!("    Icon:               {:?}", device.icon().await?);
                //             // println!("    Class:              {:?}", device.class().await?);
                //             // println!("    UUIDs:              {:?}", device.uuids().await?.unwrap_or_default());
                //             // println!("    Paired:             {:?}", device.is_paired().await?);
                //             // println!("    Connected:          {:?}", device.is_connected().await?);
                //             // println!("    Trusted:            {:?}", device.is_trusted().await?);
                //             // println!("    Modalias:           {:?}", device.modalias().await?);
                //             // println!("    RSSI:               {:?}", device.rssi().await?);
                //             // println!("    TX power:           {:?}", device.tx_power().await?);
                //             // println!("    Manufacturer data:  {:02X?}", device.manufacturer_data().await?);
                //             // println!("    Service data:       {:?}", device.service_data().await?);
            
                //             println!("Gonna try and connect");
                //             match device.connect().await {
                //                 Ok(_) => {
                //                     println!("connected");
            
                //                     if let Ok(s) = device.services().await {
                //                         // println!("Got services {:?}", s);
                                        
                //                         for svc in s {
                //                             // print!("Gonna check out svc {:?}", svc);
                //                             if let Ok(c) = svc.characteristics().await {
                //                                 // println!("Found characteristic {:?}", c);
                //                                 for ch in c {
                //                                     if let Ok(chu) = ch.uuid().await {
                //                                         if chu.to_string() == "7dfc9001-7d1c-4951-86aa-8d9728f8d66c" {
                //                                             println!("Gottem {:?}", chu);
                //                                             println!("Writing to {:?}", ch.uuid().await?);
                                                            
                //                                             match ch.write(&[175]).await {
                //                                                 Ok(_) => (),
                //                                                 Err(e) => println!("Failed to write: {:?}", e),
                //                                             }
                //                                         }
                //                                     }
                                                    
                //                                 }
                //                             }
                //                         }
                //                     } else {
                //                         println!("Fail to get svc");
                //                     }
                //                 },
                //                 Err(e) => {
                //                     println!("fail to connect {:?}", e)
                //                 }
                //             }
                //         }
                //     }
                // }
                // let device = adapter.device(addr)?;

                // if let (Ok(m_data), Ok(rssi_data)) = (device.manufacturer_data().await, device.rssi().await) {
                //     if let (Some(m_d), Some(r_d)) = (m_data, rssi_data) {
                //         if m_d.contains_key(&0x004c) {

                //             println!("Device changed: {addr}");
                //             println!("    {property:?}");
                //             println!("    Address type:       {}", device.address_type().await?);
                //             println!("    Name:               {:?}", device.name().await?);
                //             println!("    Icon:               {:?}", device.icon().await?);
                //             println!("    Class:              {:?}", device.class().await?);
                //             println!("    UUIDs:              {:?}", device.uuids().await?.unwrap_or_default());
                //             println!("    Paired:             {:?}", device.is_paired().await?);
                //             println!("    Connected:          {:?}", device.is_connected().await?);
                //             println!("    Trusted:            {:?}", device.is_trusted().await?);
                //             println!("    Modalias:           {:?}", device.modalias().await?);
                //             println!("    RSSI:               {:?}", device.rssi().await?);
                //             println!("    TX power:           {:?}", device.tx_power().await?);
                //             println!("    Manufacturer data:  {:02X?}", device.manufacturer_data().await?);
                //             println!("    Service data:       {:?}", device.service_data().await?);
                //         }
                //     }
                // }

            }
            else => break
        }
    }

    Ok(())
}
