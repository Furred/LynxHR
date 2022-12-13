use btleplug::api::{bleuuid::uuid_from_u16, Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use rand::{Rng, thread_rng};
use std::error::Error;
use std::thread;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

// Created by the Furred Team
// LynxHR is used to connect your heartbeat to VR!
// Version : 0.0.1 ALPHA | 12-13-2022

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await.unwrap();

    // Get the first bluetooth adapter available by the Host Device
    // !! : If no adapters is found the program will crash!
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    // Start scanning for devices
    central.start_scan(ScanFilter::default()).await?;

    time::sleep(Duration::from_secs(2)).await;

    let devices = central;

    println!("LynxHR Version v0.0.1 ALPHA");
    println!("Available Devices -> {}", 0)
    Ok(())
}
