use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, long_about = None)]
pub struct Cli {
	#[arg(short, long, value_name = "FILE")]
	pub output: Option<PathBuf>,

	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
	Print,
	Update {
		start_id: i32,
		end_id: i32,
	},
	Load {
		#[arg(value_name = "FILE")]
		matches: PathBuf,
		rating_period: i32,
	},
}
