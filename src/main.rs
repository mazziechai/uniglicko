mod cli;
mod data;

use std::{
	cell::RefCell,
	collections::HashSet,
	error::Error,
	fs::File,
	io::{self, Write},
	path::PathBuf,
	rc::Rc,
};

use chrono::prelude::*;
use clap::Parser;
use cli::{Cli, Commands};
use csv::Reader;
use data::{calculate_rating_period, create_schema, get_player_id, new_player, Match};
use rusqlite::{params, Connection, Result};

fn main() -> Result<(), Box<dyn Error>> {
	let cli = Cli::parse();

	let conn = Rc::new(RefCell::new(Connection::open(cli.database)?));
	create_schema(&mut conn.borrow_mut())?;

	let mut out = match cli.output.as_deref() {
		Some(path) => {
			Box::new(File::create(&path).expect("could not access path!")) as Box<dyn Write>
		}
		None => Box::new(io::stdout()) as Box<dyn Write>,
	};

	match cli.command {
		Commands::Print => match print_command(&conn.borrow()) {
			Ok(string) => out
				.write_all(string.as_bytes())
				.expect("could not access file!"),
			Err(e) => panic!("could not use the print command! {e}"),
		},
		Commands::Load {
			matches,
			rating_period,
		} => match load_command(&conn.borrow(), matches, rating_period) {
			Ok(c) => println!("success loading {c} matches!"),
			Err(e) => return Err(e),
		},
		Commands::Update { rating_period } => match update_command(&conn.borrow(), rating_period) {
			Ok(s) => out.write_all(s.as_bytes()).expect("could not access file!"),
			Err(e) => return Err(e),
		},
	}

	Ok(())
}

fn print_command(conn: &Connection) -> Result<String> {
	let mut stmt = conn.prepare(
		"SELECT name, MAX(rating), rd FROM players GROUP BY name ORDER BY MAX(rating) DESC;",
	)?;

	let mut result = stmt.query([])?;

	let mut string = String::from("# Ranking\n```");
	let mut row_count = 0;

	while let Some(row) = result.next()? {
		row_count += 1;

		let name: String = row.get(0)?;
		let rating: f64 = row.get::<usize, f64>(1)?.round();
		let rd: f64 = row.get::<usize, f64>(2)?.round();

		string.push_str(&format!("\n{row_count}: {name} — {rating}±{rd}"))
	}

	string.push_str("\n```\n");

	Ok(string)
}

fn load_command(
	conn: &Connection,
	path: PathBuf,
	rating_period: i32,
) -> Result<usize, Box<dyn Error>> {
	let mut reader = Reader::from_path(&path)?;
	let records = reader.records();
	let mut matches: Vec<Match> = Vec::new();
	let mut players: HashSet<String> = HashSet::new();

	for result in records {
		let record = result?;

		let mut time_string = String::from(&record[0]);
		time_string.push_str(" 00:00");
		let date = NaiveDateTime::parse_from_str(&time_string, "%Y-%m-%d %H:%M")?;

		let player1 = &record[1];
		players.insert(player1.to_owned());
		let player2 = &record[4];
		players.insert(player2.to_owned());

		let score1 = &record[2].parse::<i32>()?;
		let score2 = &record[3].parse::<i32>()?;

		matches.push(Match {
			date,
			player1: player1.to_owned(),
			score1: *score1,
			score2: *score2,
			player2: player2.to_owned(),
		});
	}

	for x in &matches {
		let mut player1_id = get_player_id(&conn, &x.player1)?;
		if player1_id == None {
			new_player(&conn, &x.player1)?;
			player1_id = get_player_id(&conn, &x.player1)?;
		}

		let mut player2_id = get_player_id(&conn, &x.player2)?;
		if player2_id == None {
			new_player(&conn, &x.player2)?;
			player2_id = get_player_id(&conn, &x.player2)?;
		}

		conn.execute(
			"INSERT INTO matches
			(player_1, player_2, score1, score2, date, rating_period)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
			(
				player1_id,
				player2_id,
				x.score1,
				x.score2,
				x.date.to_string(),
				rating_period,
			),
		)?;
	}

	Ok(matches.len())
}

fn update_command(conn: &Connection, rating_period: i32) -> Result<String, Box<dyn Error>> {
	let new_ratings = calculate_rating_period(&conn, rating_period)?;

	let mut update_ratings_stmt =
		conn.prepare("UPDATE players SET rating = ?1, rd = ?2, vol = ?3 WHERE id = ?4;")?;

	let mut string = String::from("# Rating Update\n```");

	for rating in new_ratings.iter() {
		let mut player_name_stmt =
			conn.prepare("SELECT name, rating, rd FROM players WHERE id = ?1 GROUP BY name;")?;
		let player: (String, f64, f64) = player_name_stmt
			.query_map([rating.0], |row| {
				Ok((
					row.get::<usize, String>(0)?,
					row.get::<usize, f64>(1)?,
					row.get::<usize, f64>(2)?,
				))
			})?
			.next()
			.unwrap()
			.unwrap();

		update_ratings_stmt.execute(params![
			rating.1.rating,
			rating.1.deviation,
			rating.1.volatility,
			rating.0,
		])?;

		string.push_str(&format!(
			"\n{} — {}±{} → {}±{}",
			player.0,
			player.1.round(),
			player.2.round(),
			rating.1.rating.round(),
			rating.1.deviation.round()
		))
	}

	string.push_str("\n```");

	Ok(string)
}
