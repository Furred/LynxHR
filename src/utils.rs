use std::thread;
use std::time::Duration;
use anyhow::bail;
use chrono::{NaiveTime, Timelike};
use crossbeam_channel::Sender;
use rand::Rng;
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
// #[command(flatten)] sender_name: Option<sender_name>,
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


pub(crate) fn convert_verbose_level_to_log_level(verbose_level: u8) -> log::LevelFilter {
    // 0 is error, 1 is warn, 2 is info, 3 is debug, 4 is trace
    match verbose_level {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        4 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Trace,
    }
}


pub(crate) fn dry_run_thread(sender: Sender<SendData>) -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    let mut charging = false;
    loop {
        let hr = rng.gen_range(25..=150);
        let battery_percentage = rng.gen_range(0..=100);
        let current_hour = chrono::Local::now().hour();
        let current_minute = chrono::Local::now().minute();
        let time = NaiveTime::from_hms_opt(current_hour, current_minute, 0).unwrap();
        let data = SendData {
            hr,
            battery_percentage,
            charging,
            time,
        };
        sender.send(data)?;
        thread::sleep(Duration::from_secs(1));
        if time.second() % 5 == 0 {
            charging = !charging;
        }
    }
}