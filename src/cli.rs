use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
	#[arg(short, long, value_name = "FILE")]
	pub output: Option<PathBuf>,

	#[arg(short, long, value_name = "DB")]
	pub database: PathBuf,

	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
	Print,
	Update {
		rating_period: i32,
	},
	Load {
		#[arg(value_name = "FILE")]
		matches: PathBuf,
		rating_period: i32,
	},
}
