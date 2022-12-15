
fn chars() {

    // Char UUID List
    let chars_uuids: Vec<Uuid> = vec![
        create_uuid!("00000009-0000-3512-2118-0009af100700"), // Authentication

        // No Authentication Required
        create_uuid!("00000006-0000-3512-2118-0009af100700"), // Battery (Used for NeosVR Only)
        create_uuid!("00002a2b-0000-1000-8000-00805f9b34fb"), // Current Time (Used for NeosVR Only / Logs)

        // Authentication Required
        create_uuid!("00002a37-0000-1000-8000-00805f9b34fb"), // HR Mesure
        //create_uuid!("00002a39-0000-1000-8000-00805f9b34fb") // HR Control
    ];

    // Chars Objects

    let battery: Characteristic = Characteristic { 
        uuid: Uuid::parse_str("00000006-0000-3512-2118-0009af100700").unwrap(), 
        service_uuid: Uuid::parse_str("0000fee0-0000-1000-8000-00805f9b34fb").unwrap(), 
        properties: CharPropFlags::READ
    };

    let time: Characteristic = Characteristic { 
        uuid: Uuid::parse_str("00002a2b-0000-1000-8000-00805f9b34fb").unwrap(), 
        service_uuid: Uuid::parse_str("0000fee0-0000-1000-8000-00805f9b34fb").unwrap(), 
        properties: CharPropFlags::READ
    };

    let hr_control: Characteristic = Characteristic { 
        uuid: Uuid::parse_str("00002a39-0000-1000-8000-00805f9b34fb").unwrap(), 
        service_uuid: Uuid::parse_str("0000180d-0000-1000-8000-00805f9b34fb").unwrap(), 
        properties: CharPropFlags::READ | CharPropFlags::WRITE
    };
}