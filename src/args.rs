use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
	#[command(subcommand)]
	pub commands: Commands
}

#[derive(Subcommand)]
#[command(version, about, long_about = None)]
pub enum Commands {
	/// Starts the node
	StartNode(StartNodeCommand)
}
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct StartNodeCommand {
	/// The port on which the server will start
	#[arg(short, long, default_value_t = 9812, value_parser=clap::value_parser!(u16).range(1..))]
	pub port: u16,

	/// A file containing a list of trusted peers
	#[arg(short, long)]
	pub trusted_peers_file: Option<PathBuf>, // FIXME: Make this a file in the app data
}