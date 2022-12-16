use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use crate::args::Cli;
use crate::utils::SendData;
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::{Aes128, Block};
use anyhow::bail;
use btleplug::api::{
    Central, Manager as _, Peripheral as _, PeripheralProperties, ScanFilter, WriteType,
};
use btleplug::api::{CharPropFlags, Characteristic};
use btleplug::platform::{Manager, Peripheral};
use chrono::naive::NaiveTime;
use clap::Parser;
use futures::stream::StreamExt;
use log::{debug, error, info, trace, warn};
use tokio::time;
use uuid::Uuid;

mod args;
mod chars;
mod senders;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = Cli::parse();
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    info!("LynxHR Version v1.0 BETA");
    info!("Starting LynxHR...");
    loop {
        let manager = Manager::new().await.unwrap_or_else(|e| {
            error!("Error creating manager: {}", e);
            std::process::exit(1);
        });

        // Get the first bluetooth adapter available by the Host Device
        // !! : If no adapters is found the program will crash!
        let adapters = manager.adapters().await.unwrap_or_else(|e| {
            error!("Error getting adapters: {}", e);
            std::process::exit(1);
        });
        if adapters.len() == 0 {
            error!("No Bluetooth Adapter Found!");
            return Ok(());
        }
        let central = adapters[0].clone();

        // Start scanning for devices
        info!("Scanning for known devices...");
        let scan_filter = ScanFilter {
            services: vec![create_uuid!("0000fee0-0000-1000-8000-00805f9b34fb")],
        };
        if let Err(_) = central.start_scan(scan_filter).await {
            error!("Failed to start scanning!");
            std::process::exit(1);
        }

        time::sleep(Duration::from_secs(10)).await;
        let devices: Vec<Peripheral>;
        match central.peripherals().await {
            Ok(d) => devices = d,
            Err(_) => {
                error!("Failed to get peripherals!");
                std::process::exit(1);
            }
        }

        info!("Available Devices -> {}", devices.len());
        for device in devices.iter() {
            let properties: Option<PeripheralProperties>;
            match device.properties().await {
                Ok(p) => properties = p,
                Err(_) => {
                    error!("Failed to get properties!");
                    warn!("Skipping device...");
                    continue;
                }
            }
            let is_connected: bool;
            match device.is_connected().await {
                Ok(c) => is_connected = c,
                Err(_) => {
                    error!("Failed to get connection status!");
                    warn!("Skipping device...");
                    continue;
                }
            }
            let local_name = properties
                .unwrap()
                .local_name
                .unwrap_or(String::from("(peripheral name unknown)"));

            if local_name == "(peripheral name unknown)" {
                warn!("Failed to get peripheral name!");
                warn!("Skipping device...");
                continue;
            }

            /* Connect To Device */
            if !is_connected {
                info!("Connecting to peripheral {:?}...", &local_name);
                if let Err(err) = device.connect().await {
                    warn!("Error connecting to peripheral, skipping: {}", err);
                    continue;
                }
            }
            info!("Connected!");

            // Subscribe to Auth Char
            if let Err(_) =
                subscribe_to_characteristic(&device, &chars::CHARS_UUIDS[..3], false).await
            {
                error!("Failed to subscribe to Characteristics!");
                warn!("Skipping device...");
                continue;
            }

            // Authentication Challange
            let auth: Characteristic = create_char!(
                "00000009-0000-3512-2118-0009af100700",
                "0000fee1-0000-1000-8000-00805f9b34fb",
                (NOTIFY, WRITE_WITHOUT_RESPONSE)
            );

            if let Err(_) = authenticate(&device, &auth, &chars::CHARS_UUIDS[3..]).await {
                error!("Failed to Authenticate!");
                warn!("Skipping device...");
                continue;
            }

            // Authentification Success Testing
            hr_control_test(
                Arc::new(device.clone()),
                Arc::new(chars::HR_CONTROL.clone()),
            )
            .await?;
        }
    }
}

async fn subscribe_to_characteristic(
    device: &Peripheral,
    uuids: &[Uuid],
    authenticated: bool,
) -> Result<(), Box<dyn Error>> {
    // Note to Lynix: Use borrows whenever possible (&), Remember Ownership*

    if authenticated == false {
        info!("Discovering Services...");
        device.discover_services().await?;
    }

    debug!("Subscribing to Characteristics...");
    let characteristics = device.characteristics();
    for characteristic in characteristics {
        if uuids.contains(&characteristic.uuid) {
            trace!("Subscribing to characteristic {:?}", characteristic);
            device.subscribe(&characteristic).await?;
        }
    }

    Ok(())
}

// Authentication Function
async fn authenticate(
    device: &Peripheral,
    auth_char: &Characteristic,
    chars: &[Uuid],
) -> anyhow::Result<()> {
    let auth_key: &[u8] = &0xc9a57d375d8d96ffd3331b73b123d43bu128.to_be_bytes();

    info!("Starting Authentication...");

    // Ask for Authentication
    device
        .write(auth_char, &[0x02, 0x00], WriteType::WithoutResponse)
        .await?;

    // Get data from authentication notification
    let mut notification_stream = device.notifications().await?;
    while let Some(data) = notification_stream.next().await {
        debug!("Awaiting for Data...");

        trace!("Received data from [{:?}]: {:?}", data.uuid, data.value);

        // This means we received authentication information
        if data.uuid == auth_char.uuid {
            trace!("CMD : {:?}", &data.value[..3]);

            if &data.value[..3] == [0x10, 0x02, 0x01] {
                let random_number = &data.value[3..];

                trace!("Number Received : {:?}", random_number);

                trace!("Encrypting number...");
                let cipher = Aes128::new_from_slice(&auth_key).unwrap();

                // Create a block
                let mut blk = Block::clone_from_slice(&random_number);
                cipher.encrypt_block(&mut blk);

                trace!("Number Encrypted : {:?}", blk);

                let encrypted_number: &[u8] = blk.as_slice();

                // We are sending the encrypted key
                device
                    .write(
                        auth_char,
                        &[[0x03, 0x00].as_slice(), encrypted_number].concat(),
                        WriteType::WithoutResponse,
                    )
                    .await?;

                debug!("sent AUTHENTIFICATION_CHALLANGE")
            }

            if &data.value[..3] == [0x10, 0x03, 0x08] {
                error!("AUTHENTICATION_FAILED");
                bail!("Authentication Failed!");
            }

            if &data.value[..3] == [0x10, 0x03, 0x01] {
                info!("AUTHENTICATION_SUCCESS");

                // Initialize
                if let Err(_) = subscribe_to_characteristic(&device, chars, true).await {
                    error!("Failed to subscribe to Characteristics!");
                    bail!("Failed to subscribe to Characteristics!");
                }
                return Ok(());
            }
        }
    }

    Ok(())
}

// Get Heart Rate
async fn hr_control_test(device: Arc<Peripheral>, char: Arc<Characteristic>) -> anyhow::Result<()> {
    // Force Heartrate
    device
        .write(&*char, &[0x15, 0x02, 0x00], WriteType::WithResponse)
        .await?;
    device
        .write(&*char, &[0x15, 0x01, 0x00], WriteType::WithResponse)
        .await?;
    // Request HR Monitoring each 12s, using the start_ping function, spawning it in a new thread
    tokio::spawn(start_ping(device.clone(), char.clone()));

    loop {
        // Read Heartrate
        let mut notification_stream = device.notifications().await?;
        while let Some(data) = notification_stream.next().await {
            debug!("Awaiting for Data...");

            trace!("Received data from [{:?}]: {:?}", data.uuid, data.value);

            let mut datapackage = SendData::default();
            let hr_mesure_uuid = create_uuid!("00002a37-0000-1000-8000-00805f9b34fb");

            // Battery
            let bat_data = device.read(&chars::BATTERY).await?;

            // Time
            let time_data = device.read(&chars::TIME).await?;

            if data.uuid == hr_mesure_uuid {
                // Parse Data into Datapack
                datapackage.hr = data.value[1];
                datapackage.battery_percentage = bat_data[1];
                datapackage.charging = bat_data[2] == 0x01;
                let tmp_current_hour = time_data[4];
                let tmp_current_minute = time_data[5];
                datapackage.time =
                    NaiveTime::from_hms_opt(tmp_current_hour as u32, tmp_current_minute as u32, 0)
                        .unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                info!("Data Package : {:?}", datapackage);
            }
        }

        time::sleep(Duration::from_millis(500)).await;
    }
}

async fn start_ping(device: Arc<Peripheral>, char: Arc<Characteristic>) {
    loop {
        if let Err(_) = device
            .write(&*char, &[0x16], WriteType::WithoutResponse)
            .await
        {
            error!("Failed to send ping!");
            return;
        };
        debug!("Sent ping!");

        debug!("Writing Force Command...");
        if let Err(_) = device
            .write(&*char, &[0x15, 0x01, 0x01], WriteType::WithResponse)
            .await
        {
            error!("Error Writing Force Command!");
            return;
        };
        time::sleep(Duration::from_secs(30)).await;
    }
}
