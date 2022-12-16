use crate::create_sender;
use clap::clap_derive::{Args, Parser, Subcommand};
#[derive(Debug, Parser)]
#[command(name = "lynxHR")]
#[command(about = "Measure your heart rate and transmit data", long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) subcommand: Subcommand,
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, help = "Sets the level of verbosity", global(true))]
    pub(crate) verbose: u8,
}
#[derive(Debug, Subcommand)]
pub(crate) enum Subcommand {
    #[command(name = "dry_run")]
    DryRun(DryRun),
    #[command(name = "run")]
    Run(Run),
    #[command(name = "list_adapters")]
    ListAdapters {},
    #[command(name = "list_devices")]
    ListDevices(ListDevices),
}
#[derive(Debug, Args)]
pub(crate) struct Run {
    #[arg(long = "api_key", help = "The api key to use for the watch")]
    pub(crate) api_key: String,
    #[arg(
        long = "adapter",
        help = "The adapter number to use. Use list_adapters to get the list of adapters",
        default_value = "0"
    )]
    pub(crate) adapter: u8,
    #[arg(
        conflicts_with = "device_mac",
        long = "device_name",
        help = "The device name to use. Use list_devices to get the list of devices"
    )]
    pub(crate) device_name: Option<String>,
    #[arg(
        long = "device_mac",
        help = "The device mac address to use. Use list_devices to get the list of devices"
    )]
    pub(crate) device_mac: Option<String>,
    #[command(flatten)]
    pub(crate) other_args: SenderCommands,
}

#[derive(Debug, Args)]
pub(crate) struct DryRun {
    #[command(flatten)]
    pub(crate) other_args: SenderCommands,
}

#[derive(Debug, Args)]
pub(crate) struct ListDevices {
    #[arg(
        long = "adapter",
        help = "The adapter number to use. Use list_adapters to get the list of adapters"
    )]
    pub(crate) adapter: u8,
}

// TODO: Implemment a better way to handle the sender commands, making a function that registers the sender commands and calls them when needed

#[derive(Debug, Args)]
pub(crate) struct SenderCommands {}
