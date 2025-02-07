use anyhow::Context;
use crossbeam_channel::{unbounded, Receiver, Sender};
use rusqlite::{params, Connection};
use snitch::gopher::{fetch_url, GopherItem, GopherURL, Menu};
use std::collections::HashSet;
use std::env;
use std::io::Read;
use std::thread;

use anyhow::Result;

struct Site {
    url: GopherURL,
    text: Option<String>,
    links: Option<Vec<GopherURL>>,
}

fn main() -> Result<()> {
    env_logger::init();

    spider(env::args().skip(1))?;

    Ok(())
}

fn init_db() -> Result<Connection> {
    let conn = Connection::open("./db.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sites(url TEXT, content_id INTEGER)",
        (),
    )?;
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS site_content USING fts4(content TEXT, tokenize=unicode61)",
        (),
    )?;
    Ok(conn)
}

fn store_site(conn: &mut Connection, site: &Site) -> Result<()> {
    let tx = conn.transaction()?;
    if let Some(content) = &site.text {
        tx.execute("INSERT INTO site_content(content) VALUES(?1)", [content])?;
        tx.execute(
            "INSERT INTO sites(url, content_id) VALUES (?1, ?2)",
            params![site.url.to_string(), tx.last_insert_rowid()],
        )?;
    }
    tx.commit()?;
    Ok(())
}

fn spider(seed: impl Iterator<Item = String>) -> Result<()> {
    let mut visited: HashSet<GopherURL> = HashSet::new();
    let (urls_tx, urls_rx) = unbounded();
    let (sites_tx, sites_rx) = unbounded();
    let mut workers = Vec::with_capacity(10);
    let mut conn = init_db()?;

    for _ in 0..workers.capacity() {
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
        store_site(&mut conn, &site).unwrap_or_else(|e| log::error!("storing site data: {e:#}"));
        log::info!("indexed {} ({} urls in queue)", site.url, urls_tx.len());
        for url in site.links.into_iter().flatten() {
            if visited.contains(&url) {
                continue;
            }
            urls_tx
                .send(url)
                .unwrap_or_else(|e| log::error!("sending url to worker: {e:#}"));
        }
    }
}

fn worker(urls: Receiver<GopherURL>, sites: Sender<Site>) {
    log::info!("worker started");
    loop {
        if let Ok(url) = urls.recv() {
            match get_url(&url) {
                Ok(site) => sites
                    .send(site)
                    .unwrap_or_else(|e| log::error!("failed to sending {url}: {e:#}")),
                Err(e) => {
                    log::error!("failed to fetch {url}: {e:#}");
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
            let site = Menu::from_url(&url, None).context("fetching menu")?;
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
