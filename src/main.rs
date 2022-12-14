use btleplug::api::{bleuuid::uuid_from_u16, Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use rand::{Rng, thread_rng};
use std::error::Error;
use std::thread;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

macro_rules! create_uuid {
    ($a:expr) => {Uuid::parse_str($a).unwrap()};
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {                                       

    println!("LynxHR Version v0.0.1 ALPHA");
    
    let chars: Vec<Uuid> = vec![
        create_uuid!("00000009-0000-3512-2118-0009af100700"), // Authentication
        //create_uuid!("00002a37-0000-1000-8000-00805f9b34fb"), // HR Mesure
        //create_uuid!("00002a39-0000-1000-8000-00805f9b34fb"), // HR Control
        //create_uuid!("00000006-0000-3512-2118-0009af100700"), // Battery (Used for NeosVR Only)
        //create_uuid!("00002a2b-0000-1000-8000-00805f9b34fb") // Current Time (Used for NeosVR Only / Logs)
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
    
                // Subscribe to Chars
                subscribe_to_characteristic(&device, &chars).await?; // Not working <============
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
async fn authenticate(device: &Peripheral) {      

    let AUTH_KEY: u128 = 0xc9a57d375d8d96ffd3331b73b123d43b; // Auth Key

    println!("Starting Authentication...");
}