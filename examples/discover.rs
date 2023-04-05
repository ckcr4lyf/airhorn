//! Discover Bluetooth devices and list them.

use bluer::{Adapter, AdapterEvent, Address, DeviceEvent, Device};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::{collections::HashSet, env};
use tokio;

async fn addr_is_airtag(device: Device) -> bool {

    // Get the Manufacturer Data & RSSSI
    let Ok(Some(manufacturer_data)) = device.manufacturer_data().await else { return false };
    let Ok(Some(rssi_data)) = device.manufacturer_data().await else { return false };


    return true;




}

async fn query_device(adapter: &Adapter, addr: Address) -> bluer::Result<()> {
    let device = adapter.device(addr)?;

    if let (Ok(m_data), Ok(rssi_data)) = (device.manufacturer_data().await, device.rssi().await) {
        if let (Some(m_d), Some(r_d)) = (m_data, rssi_data) {
            if m_d.contains_key(&0x004c) {

                let mdata =m_d.get(&0x004c).expect("bruvva");

                if mdata[2] != 0x10 {
                    return Ok(())
                }

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

                println!("Gonna try and connect");
                match device.connect().await {
                    Ok(_) => {
                        println!("connected");

                        if let Ok(s) = device.services().await {
                            // println!("Got services {:?}", s);
                            
                            for svc in s {
                                // print!("Gonna check out svc {:?}", svc);
                                if let Ok(c) = svc.characteristics().await {
                                    // println!("Found characteristic {:?}", c);
                                    for ch in c {
                                        if let Ok(chu) = ch.uuid().await {
                                            if chu.to_string() == "7dfc9001-7d1c-4951-86aa-8d9728f8d66c" {
                                                println!("Gottem {:?}", chu);
                                                println!("Writing to {:?}", ch.uuid().await?);
                                                
                                                match ch.write(&[175]).await {
                                                    Ok(_) => println!("WROTE IT"),
                                                    Err(e) => println!("Failed to write: {:?}", e),
                                                }
                                            }
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
            }
        }
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

    let device_events = adapter.discover_devices().await?;
    pin_mut!(device_events);

    let mut all_change_events = SelectAll::new();

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
                            query_device(&adapter, addr).await
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
            }
            Some((addr, DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {

                

                let device = adapter.device(addr)?;

                if let (Ok(m_data), Ok(rssi_data)) = (device.manufacturer_data().await, device.rssi().await) {
                    if let (Some(m_d), Some(r_d)) = (m_data, rssi_data) {
                        if m_d.contains_key(&0x004c) {
            
                            println!("Device changed: {addr}");
                            println!("    {property:?}");
                            // println!("Device added: {addr}");
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
            
                            println!("Gonna try and connect");
                            match device.connect().await {
                                Ok(_) => {
                                    println!("connected");
            
                                    if let Ok(s) = device.services().await {
                                        // println!("Got services {:?}", s);
                                        
                                        for svc in s {
                                            // print!("Gonna check out svc {:?}", svc);
                                            if let Ok(c) = svc.characteristics().await {
                                                // println!("Found characteristic {:?}", c);
                                                for ch in c {
                                                    if let Ok(chu) = ch.uuid().await {
                                                        if chu.to_string() == "7dfc9001-7d1c-4951-86aa-8d9728f8d66c" {
                                                            println!("Gottem {:?}", chu);
                                                            println!("Writing to {:?}", ch.uuid().await?);
                                                            
                                                            match ch.write(&[175]).await {
                                                                Ok(_) => (),
                                                                Err(e) => println!("Failed to write: {:?}", e),
                                                            }
                                                        }
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
                        }
                    }
                }
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
