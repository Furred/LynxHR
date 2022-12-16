use chrono::NaiveTime;
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

// Create sender macro
// Usage:
// put the macro at the end of args.rs like this:
// create_sender!(sender_name, config_struct)
// Then, in the sender_commands struct, add this line:
// #[command(flatten)] sender_name: sender_name,
#[macro_export]
macro_rules! create_sender {
    ($name:ident, $other:ident) => {
        #[derive(Debug, Args)]
        #[group(skip)]
        pub(crate) struct $name {
            // --$name to enable the sender
            #[arg(long = stringify!($name), help = format!("Enable the {} sender", stringify!($name)))]
            pub(crate) $name: bool,
            // delegate other args to another struct
            #[command(flatten)]
            pub(crate) other_args: $other,
        }
    };
}

#[derive(Default, Debug)]
pub struct SendData {
    pub hr: u8,
    pub battery_percentage: u8,
    pub charging: bool,
    pub time: NaiveTime,
}
