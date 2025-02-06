use anyhow::Result;
use rusqlite::{params, Connection};

fn main() -> Result<()> {
    let mut conn = Connection::open("./db.sql")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS books(filename TEXT, content_id INTEGER)",
        (),
    )?;
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS book_content USING fts4(content TEXT)",
        (),
    )?;

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO book_content (content) VALUES (?1)",
        ["bla bla bla".to_string()],
    )?;
    tx.execute(
        "INSERT INTO books(filename, content_id) VALUES (?1, ?2)",
        params!["test".to_string(), tx.last_insert_rowid()],
    )?;
    tx.commit()?;
    Ok(())
}
