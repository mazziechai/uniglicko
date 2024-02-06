use std::collections::HashMap;

use chrono::NaiveDateTime;
use rusqlite::{params, Connection, Result};
use skillratings::glicko2::{glicko2_rating_period, Glicko2Config, Glicko2Rating};
use skillratings::Outcomes;

#[derive(Debug)]
pub struct Match {
	pub date: NaiveDateTime,
	pub player1: String,
	pub score1: i32,
	pub score2: i32,
	pub player2: String,
}

pub struct RatingPeriodMatch {
	pub players: (usize, usize),
	pub score1: usize,
	pub score2: usize,
}

pub fn create_schema(conn: &mut Connection) -> Result<()> {
	let tx = conn.transaction()?;

	tx.execute(
		"CREATE TABLE IF NOT EXISTS players (
			id     INTEGER PRIMARY KEY AUTOINCREMENT,
			name   TEXT    NOT NULL
						   UNIQUE,
			rating         NOT NULL,
			rd             NOT NULL,
			vol            NOT NULL
		);",
		[],
	)?;

	tx.execute(
		"CREATE TABLE IF NOT EXISTS matches (
			id            INTEGER PRIMARY KEY AUTOINCREMENT,
			player_1      INTEGER REFERENCES players (id) 
								  NOT NULL,
			player_2      INTEGER REFERENCES players (id) 
								  NOT NULL,
			score1        INTEGER NOT NULL,
			score2        INTEGER NOT NULL,
			date                  NOT NULL,
			rating_period         NOT NULL
		);",
		[],
	)?;

	tx.commit()
}

pub fn get_player_id(conn: &Connection, name: &str) -> Result<Option<usize>> {
	let mut stmt = conn.prepare("SELECT id FROM players WHERE name = ?1;")?;
	let mut result = stmt.query_map([&name], |row| row.get::<usize, usize>(0))?;

	let id: Option<usize> = match result.next() {
		Some(x) => Some(x?),
		None => None,
	};

	Ok(id)
}

pub fn new_player(conn: &Connection, name: &str) -> Result<()> {
	let mut stmt = conn.prepare(
		"INSERT INTO players (name, rating, rd, vol) 
		 VALUES (?1, ?2, ?3, ?4);",
	)?;
	stmt.execute(params![name, 1500.0, 350.0, 0.06])?;

	Ok(())
}

pub fn calculate_rating_period(
	conn: &Connection,
	rating_period: i32,
) -> Result<Vec<(usize, Glicko2Rating)>> {
	let mut match_select_stmt = conn.prepare(
		"SELECT player_1, player_2, score1, score2 FROM matches
		WHERE rating_period = ?1;",
	)?;
	let matches: Vec<RatingPeriodMatch> = match_select_stmt
		.query_map([rating_period], |row| {
			Ok(RatingPeriodMatch {
				players: (row.get(0)?, row.get(1)?),
				score1: row.get(2)?,
				score2: row.get(3)?,
			})
		})?
		.map(|m| m.expect("could not unwrap match!"))
		.collect();

	let mut player_select_stmt = conn.prepare("SELECT id, rating, rd, vol FROM players;")?;
	let players: HashMap<usize, Glicko2Rating> = player_select_stmt
		.query_map([], |row| {
			Ok((
				row.get(0)?,
				Glicko2Rating {
					rating: row.get(1)?,
					deviation: row.get(2)?,
					volatility: row.get(3)?,
				},
			))
		})?
		.map(|p| p.expect("could not unwrap ratings!"))
		.collect();

	let mut new_ratings: Vec<(usize, Glicko2Rating)> = Vec::new();

	for player in players.iter() {
		let mut match_results: Vec<(Glicko2Rating, Outcomes)> = Vec::new();

		let player_matches = matches
			.iter()
			.filter(|&m| m.players.0 == *player.0 || m.players.1 == *player.0);

		for m in player_matches {
			let mut outcomes: Vec<Outcomes> = Vec::new();

			if m.players.0 == *player.0 {
				for _ in 0..m.score1 {
					outcomes.push(Outcomes::WIN);
				}

				for _ in 0..m.score2 {
					outcomes.push(Outcomes::LOSS);
				}

				for outcome in outcomes {
					match_results.push((*players.get(&m.players.1).unwrap(), outcome));
				}
			} else if m.players.1 == *player.0 {
				for _ in 0..m.score2 {
					outcomes.push(Outcomes::WIN);
				}

				for _ in 0..m.score1 {
					outcomes.push(Outcomes::LOSS);
				}

				for outcome in outcomes {
					match_results.push((*players.get(&m.players.0).unwrap(), outcome));
				}
			}
		}

		new_ratings.push((
			*player.0,
			glicko2_rating_period(&player.1, &match_results, &Glicko2Config::new()),
		));
	}

	Ok(new_ratings)
}
