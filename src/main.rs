use btleplug::api::{Characteristic, CharPropFlags};
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Manager, Peripheral};
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;
use aes::{Aes128, Block};
use aes::cipher::{KeyInit, BlockEncrypt};

macro_rules! create_uuid {
    ($a:expr) => {Uuid::parse_str($a).unwrap()};
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {                                       

    println!("LynxHR Version v0.0.1 ALPHA");
    
    let chars: Vec<Uuid> = vec![
        create_uuid!("00000009-0000-3512-2118-0009af100700"), // Authentication

        // No Authentication Required
        create_uuid!("00000006-0000-3512-2118-0009af100700"), // Battery (Used for NeosVR Only)
        create_uuid!("00002a2b-0000-1000-8000-00805f9b34fb"), // Current Time (Used for NeosVR Only / Logs)

        // Authentication Required
        create_uuid!("00002a37-0000-1000-8000-00805f9b34fb"), // HR Mesure
        create_uuid!("00002a39-0000-1000-8000-00805f9b34fb") // HR Control
    ];

    loop {
        let manager = Manager::new().await.unwrap();

        // Get the first bluetooth adapter available by the Host Device
        // !! : If no adapters is found the program will crash!
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().nth(0).unwrap();

        // Start scanning for devices
        println!("Scanning for known devices...");
        let scanf = ScanFilter{services: vec![Uuid::parse_str("0000fee0-0000-1000-8000-00805f9b34fb").unwrap()]};
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
                subscribe_to_characteristic(&device, &chars[..3]).await?;

                // Authentication Challange
                let auth: Characteristic = Characteristic { 
                    uuid: Uuid::parse_str("00000009-0000-3512-2118-0009af100700").unwrap(), 
                    service_uuid: Uuid::parse_str("0000fee1-0000-1000-8000-00805f9b34fb").unwrap(), 
                    properties: CharPropFlags::NOTIFY | CharPropFlags::WRITE_WITHOUT_RESPONSE | CharPropFlags::NOTIFY
                    // "READ | WRITE_WITHOUT_RESPONSE | NOTIFY" 
                };

                authenticate(&device, &auth).await?;
            }
        }
    }

}

async fn subscribe_to_characteristic(device: &Peripheral, uuids: &[Uuid]) -> Result<(), Box<dyn Error>> {        
    // Note to Lynix: Use borrows whenever possible (&), Remember Ownership*

    println!("Info: Discovering Services...");
    device.discover_services().await?;

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
async fn authenticate(device: &Peripheral, auth_char: &Characteristic) -> Result<(), Box<dyn Error>> {         

    let auth_key = 0xc9a57d375d8d96ffd3331b73b123d43bu128.to_le_bytes(); // Auth Key as Hex

    println!("Starting Authentication...");

    // Ask for Authentication
    device.write(auth_char,&[0x02, 0x00], WriteType::WithoutResponse).await?;

    // Get data from authentication notification
    let mut notification_stream = device.notifications().await?;
    while let Some(data) = notification_stream.next().await {
        println!("Awaiting for Data...");
        
        println!(
            "Received data from [{:?}]: {:?}",
            data.uuid, data.value
        );


        // This means we received authentication information
        if data.uuid == auth_char.uuid {
            let random_number = &data.value[3..];

            println!("Number Received : {:?}", random_number);
            
            println!("Encrypting...");
            let cipher = Aes128::new_from_slice(&auth_key).unwrap();

            // Create a block
            let mut blk = Block::clone_from_slice(random_number);
            cipher.encrypt_block(&mut blk);

            println!("Number Encrypted : {:?}", blk);

            let encrypted_number: &[u8] = blk.as_slice();
            

            // We are sending the encrypted key
            device.write(auth_char, &[[0x03,0x00].as_slice(), encrypted_number].concat(), WriteType::WithoutResponse).await?;
        }
    }

    Ok(())
}