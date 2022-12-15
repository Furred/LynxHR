use crate::create_uuid;
use btleplug::api::{CharPropFlags, Characteristic};
use lazy_static::lazy_static;
use uuid::Uuid;
lazy_static! {
// Char UUID List
pub(crate) static ref CHARS_UUIDS: Vec<Uuid> = vec![
    create_uuid!("00000009-0000-3512-2118-0009af100700"), // Authentication
    // No Authentication Required
    create_uuid!("00000006-0000-3512-2118-0009af100700"), // Battery (Used for NeosVR Only)
    create_uuid!("00002a2b-0000-1000-8000-00805f9b34fb"), // Current Time (Used for NeosVR Only / Logs)
    // Authentication Required
    create_uuid!("00002a37-0000-1000-8000-00805f9b34fb"), // HR Mesure
                                                          //create_uuid!("00002a39-0000-1000-8000-00805f9b34fb") // HR Control
];

// Chars Objects

pub(crate) static ref BATTERY: Characteristic = Characteristic {
    uuid: create_uuid!("00000006-0000-3512-2118-0009af100700"),
    service_uuid: create_uuid!("0000fee0-0000-1000-8000-00805f9b34fb"),
    properties: CharPropFlags::READ,
};

pub(crate) static ref TIME: Characteristic = Characteristic {
    uuid: create_uuid!("00002a2b-0000-1000-8000-00805f9b34fb"),
    service_uuid: create_uuid!("0000fee0-0000-1000-8000-00805f9b34fb"),
    properties: CharPropFlags::READ,
};

pub(crate) static ref HR_CONTROL: Characteristic = Characteristic {
    uuid: create_uuid!("00002a39-0000-1000-8000-00805f9b34fb"),
    service_uuid: create_uuid!("0000180d-0000-1000-8000-00805f9b34fb"),
    properties: CharPropFlags::READ | CharPropFlags::WRITE,
};
}
