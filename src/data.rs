use chrono::NaiveDateTime;
use rusqlite::{params, Connection, Result};

#[derive(Debug)]
pub struct Match {
	pub date: NaiveDateTime,
	pub player1: String,
	pub score1: i32,
	pub score2: i32,
	pub player2: String,
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
