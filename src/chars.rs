use crate::{create_uuid, create_char};
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
    create_uuid!("00000001-0000-3512-2118-0009af100700"), // Sensor Sp02?  Maybe
];

// Chars Objects

pub(crate) static ref BATTERY: Characteristic = create_char!(
        "00000006-0000-3512-2118-0009af100700",
        "0000fee0-0000-1000-8000-00805f9b34fb",
        (READ)
    );

pub(crate) static ref TIME: Characteristic = create_char!(
        "00002a2b-0000-1000-8000-00805f9b34fb",
        "0000fee0-0000-1000-8000-00805f9b34fb",
        (READ)
    );

pub(crate) static ref HR_CONTROL: Characteristic = create_char!(
        "00002a39-0000-1000-8000-00805f9b34fb",
        "0000180d-0000-1000-8000-00805f9b34fb",
        (READ, WRITE)
    );

}
