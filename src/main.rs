mod cli;
mod db;

use std::{
	fs::File,
	io::{self, Write},
};

use clap::Parser;
use cli::{Cli, Commands};
use db::create_schema;
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
	let cli = Cli::parse();

	let mut conn = Connection::open("database.db")?;
	create_schema(&mut conn)?;

	let mut out = match cli.output.as_deref() {
		Some(path) => {
			Box::new(File::create(&path).expect("could not access path!")) as Box<dyn Write>
		}
		None => Box::new(io::stdout()) as Box<dyn Write>,
	};

	match cli.command {
		Commands::Print => match print_command_string(&conn) {
			Ok(string) => out
				.write_all(string.as_bytes())
				.expect("could not access file!"),
			Err(e) => return Err(e),
		},
		Commands::Update { matches: _ } => todo!(),
	}

	Ok(())
}

fn print_command_string(conn: &Connection) -> Result<String> {
	let mut stmt =
		conn.prepare("SELECT name, MAX(rating), rd FROM players ORDER BY MAX(rating) DESC;")?;

	let mut result = stmt.query([])?;

	let mut string = String::from("# Ranking\n```");
	let mut row_count = 0;

	while let Some(row) = result.next()? {
		row_count += 1;

		let name: String = row.get(0)?;
		let rating: f64 = row.get(1)?;
		let rd: f64 = row.get(2)?;

		string.push_str(&format!("\n{row_count}: {name} — {rating}±{rd}"))
	}

	string.push_str("\n```\n");

	Ok(string)
}
