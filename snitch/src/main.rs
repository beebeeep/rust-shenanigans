use anyhow::Context;
use encoding::all::WINDOWS_1251;
use encoding::DecoderTrap;
use encoding::Encoding;
use snitch::gopher::fetch_url;
use snitch::gopher::GopherItem;
use snitch::gopher::GopherURL;
use snitch::gopher::Menu;
use std::env;
use std::{fs::File, io, io::Read, str};

use anyhow::Result;
use rusqlite::{params, Connection};

struct Site {
    url: GopherURL,
    text: Option<String>,
    links: Option<Vec<GopherURL>>,
}

fn main() -> Result<()> {
    /*
    let mut conn = Connection::open("./db.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS books(filename TEXT, content_id INTEGER)",
        (),
    )?;
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS book_content USING fts4(content TEXT, tokenize=unicode61)",
        (),
    )?;

    for filename in io::stdin().lines() {
        let filename = filename?;
        let mut content = Vec::with_capacity(4096);
        File::open(&filename)?.read_to_end(&mut content)?;
        let book = match str::from_utf8(&content) {
            Ok(s) => String::from(s),
            Err(_) => WINDOWS_1251.decode(&content, DecoderTrap::Ignore).unwrap(),
        };

        let tx = conn.transaction()?;
        tx.execute("INSERT INTO book_content (content) VALUES (?1)", [book])?;
        tx.execute(
            "INSERT INTO books(filename, content_id) VALUES (?1, ?2)",
            params![filename, tx.last_insert_rowid()],
        )?;
        tx.commit()?;
        println!("imported {filename}");
    }
    */

    for url in env::args().skip(1) {
        let site = get_url(&url)?;
        println!("{}", site.text.unwrap_or(String::from("<nothing>")));
    }

    Ok(())
}

fn get_url(url: &str) -> Result<Site> {
    let url: GopherURL = url.try_into().context("parsing URL")?;
    match url.gopher_type {
        GopherItem::TextFile => {
            let mut text = String::new();
            fetch_url(&url, None)
                .context("fetching text file")?
                .read_to_string(&mut text)
                .context("reading text file")?;
            Ok(Site {
                text: Some(text),
                url,
                links: None,
            })
        }
        GopherItem::Submenu => {
            let site = Menu::from_url(&url, None).context("fetching site")?;
            Ok(Site {
                text: Some(
                    site.items
                        .iter()
                        .map(|x| x.label.clone())
                        .collect::<Vec<String>>()
                        .join("\n"),
                ),
                links: Some(site.items.iter().map(|x| x.url.clone()).flatten().collect()),
                url,
            })
        }
        _ => Ok(Site {
            url,
            text: None,
            links: None,
        }),
    }
}
