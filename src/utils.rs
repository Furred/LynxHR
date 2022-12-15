// Create UUID macro
#[macro_export]
macro_rules! create_uuid {
    ($a:expr) => {
        Uuid::parse_str($a).unwrap()
    };
}

// Create Characteristic macro
#[macro_export]
macro_rules! create_char {
    ($a:expr, $b:expr, ($($c:ident),*)) => {
        Characteristic {
            uuid: create_uuid!($a),
            service_uuid: create_uuid!($b),
            properties: CharPropFlags::empty() $(| CharPropFlags::$c)*,
        }
    };
}
