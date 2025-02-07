use anyhow::Context;
use crossbeam_channel::bounded;
use crossbeam_channel::unbounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use log::error;
use log::info;
use snitch::gopher::fetch_url;
use snitch::gopher::GopherItem;
use snitch::gopher::GopherURL;
use snitch::gopher::Menu;
use std::collections::HashSet;
use std::env;
use std::sync::mpsc;
use std::thread;
use std::{io::Read, str};

use anyhow::Result;

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

    env_logger::init();
    spider(env::args().skip(1))?;

    Ok(())
}

fn spider(seed: impl Iterator<Item = String>) -> Result<()> {
    let mut visited: HashSet<GopherURL> = HashSet::new();
    let (urls_tx, urls_rx) = unbounded();
    let (sites_tx, sites_rx) = unbounded();
    let mut workers = Vec::with_capacity(10);
    for _ in 0..10 {
        let urls_rx = urls_rx.clone();
        let sites_tx = sites_tx.clone();
        workers.push(thread::spawn(move || worker(urls_rx, sites_tx)));
    }
    for url in seed {
        if let Ok(url) = GopherURL::try_from(url.as_str()) {
            urls_tx.send(url).context("sending seed url")?;
        }
    }

    loop {
        let site = sites_rx.recv().context("receiving site")?;
        visited.insert(site.url.clone());
        log::info!("indexed {}", site.url);
        for url in site.links.into_iter().flatten() {
            if visited.contains(&url) {
                continue;
            }
            urls_tx
                .send(url)
                .unwrap_or_else(|e| log::error!("sending url to worker: {e}"));
        }
    }
}

fn worker(urls: Receiver<GopherURL>, sites: Sender<Site>) {
    log::info!("worker started");
    loop {
        if let Ok(url) = urls.recv() {
            log::info!("indexing {url}");
            match get_url(&url) {
                Ok(site) => sites
                    .send(site)
                    .unwrap_or_else(|e| log::error!("failed to sending {url}: {e}")),
                Err(e) => {
                    log::error!("failed to fetch {url}: {e}");
                }
            }
        }
    }
}

fn get_url(url: &GopherURL) -> Result<Site> {
    match url.gopher_type {
        GopherItem::TextFile => {
            let mut text = String::new();
            fetch_url(&url, None)
                .context("fetching text file")?
                .read_to_string(&mut text)
                .context("reading text file")?;
            Ok(Site {
                text: Some(text),
                url: url.clone(),
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
                url: url.clone(),
            })
        }
        _ => Ok(Site {
            url: url.clone(),
            text: None,
            links: None,
        }),
    }
}
