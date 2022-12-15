
// Create UUID Marco
macro_rules! create_uuid {
    ($a:expr) => {Uuid::parse_str($a).unwrap()};
}