mod chars;
mod utils;

use aes::cipher::{BlockEncrypt, KeyInit};
use aes::{Aes128, Block};
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::api::{CharPropFlags, Characteristic};
use btleplug::platform::{Manager, Peripheral};
use chrono::naive::NaiveTime;
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("LynxHR Version v0.0.1 ALPHA");

    loop {
        let manager = Manager::new().await.unwrap();

        // Get the first bluetooth adapter available by the Host Device
        // !! : If no adapters is found the program will crash!
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().nth(0).unwrap();

        // Start scanning for devices
        println!("Scanning for known devices...");
        let scanf = ScanFilter {
            services: vec![create_uuid!("0000fee0-0000-1000-8000-00805f9b34fb")],
        };
        central.start_scan(scanf).await?;

        time::sleep(Duration::from_secs(10)).await;

        let devices = central.peripherals().await?;

        println!("Available Devices -> {}", devices.len());

        for device in devices.iter() {
            let properties = device.properties().await?;
            let is_connected = device.is_connected().await?;
            let local_name = properties
                .unwrap()
                .local_name
                .unwrap_or(String::from("(peripheral name unknown)"));

            /* Connect To Device */
            if !is_connected {
                println!("Connecting to peripheral {:?}...", &local_name);
                if let Err(err) = device.connect().await {
                    eprintln!("Error connecting to peripheral, skipping: {}", err);
                    continue;
                }
            }

            let is_connected = device.is_connected().await?;

            if is_connected {
                println!("Connected!");
                println!("Warning! Authentication Key is required to access your device!");

                // Subscribe to Auth Char
                subscribe_to_characteristic(&device, &chars::CHARS_UUIDS[..3], false).await?;

                // Authentication Challange
                let auth: Characteristic = create_char!(
                    "00000009-0000-3512-2118-0009af100700",
                    "0000fee1-0000-1000-8000-00805f9b34fb",
                    (NOTIFY, WRITE_WITHOUT_RESPONSE)
                );

                authenticate(&device, &auth, &chars::CHARS_UUIDS[3..]).await?;

                // Authentification Success Testing
            }
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
        println!("Info: Discovering Services...");
        device.discover_services().await?;
    }

    println!("Info: Subscribing to Chars...");
    let characteristics = device.characteristics();
    for characteristic in characteristics {
        if uuids.contains(&characteristic.uuid) {
            println!("Subscribing to characteristic {:?}", characteristic);
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
) -> Result<(), Box<dyn Error>> {
    let auth_key: &[u8] = &0xc9a57d375d8d96ffd3331b73b123d43bu128.to_be_bytes();

    println!("Starting Authentication...");

    // Ask for Authentication
    device
        .write(auth_char, &[0x02, 0x00], WriteType::WithoutResponse)
        .await?;

    // Get data from authentication notification
    let mut notification_stream = device.notifications().await?;
    while let Some(data) = notification_stream.next().await {
        //println!("Awaiting for Data..."); Not needed for now 
        //println!("Received data from [{:?}]: {:?}", data.uuid, data.value); Not needed for now

        // This means we received authentication information
        if data.uuid == auth_char.uuid {
            println!("Debug : {:?}", &data.value[..3]);

            if &data.value[..3] == [0x10, 0x02, 0x01] {
                let random_number = &data.value[3..];

                println!("Number Received : {:?}", random_number);

                println!("Encrypting...");
                let cipher = Aes128::new_from_slice(&auth_key).unwrap();

                // Create a block
                let mut blk = Block::clone_from_slice(&random_number);
                cipher.encrypt_block(&mut blk);

                println!("Number Encrypted : {:?}", blk);

                let encrypted_number: &[u8] = blk.as_slice();

                // We are sending the encrypted key
                device
                    .write(
                        auth_char,
                        &[[0x03, 0x00].as_slice(), encrypted_number].concat(),
                        WriteType::WithoutResponse,
                    )
                    .await?;

                println!("info: sent AUTHENTIFICATION_CHALLANGE")
            }

            if &data.value[..3] == [0x10, 0x03, 0x08] {
                println!("error: AUTHENTICATION_FAILED");
            }

            if &data.value[..3] == [0x10, 0x03, 0x01] {
                println!("info: AUTHENTICATION_SUCCESS");

                // Initialize
                subscribe_to_characteristic(&device, chars, true).await?;

                hr_control_test(&device, &chars::HR_CONTROL).await?;
            }
        }
    }

    Ok(())
}

// Get Heart Rate
async fn hr_control_test(device: &Peripheral, char: &Characteristic) -> Result<(), Box<dyn Error>> {
    loop {
        // Force Heartrate
        device
            .write(char, &[0x15, 0x02, 0x00], WriteType::WithResponse)
            .await?;
        device
            .write(char, &[0x15, 0x01, 0x00], WriteType::WithResponse)
            .await?;

        // Read Heartrate
        let mut notification_stream = device.notifications().await?;
        while let Some(data) = notification_stream.next().await {
            println!("Awaiting for Data...");

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
                println!("Data Package : {:?}", datapackage);
            }
        }

        device
            .write(char, &[0x15, 0x01, 0x00], WriteType::WithResponse)
            .await?;

        time::sleep(Duration::from_secs(1)).await;

        // Request HR Monitoring each 12s
        device
            .write(char, &[0x16], WriteType::WithoutResponse)
            .await?;
    }
}

#[derive(Default, Debug)]
pub struct SendData {
    pub hr: u8,
    pub battery_percentage: u8,
    pub charging: bool,
    pub time: NaiveTime,
}
