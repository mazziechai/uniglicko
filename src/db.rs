use rusqlite::{Connection, Result};

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
		);
		",
		[],
	)?;

	tx.execute(
		"CREATE TABLE IF NOT EXISTS matches (
			id       INTEGER PRIMARY KEY AUTOINCREMENT,
			player_1 INTEGER REFERENCES players (id) 
							NOT NULL,
			player_2 INTEGER REFERENCES players (id) 
							NOT NULL,
			winner   INTEGER REFERENCES players (id) 
							NOT NULL,
			date             NOT NULL
		);",
		[],
	)?;

	tx.commit()
}
