use anyhow::Context;
use clap::Parser;
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

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'f', long, default_value = "./db.db")]
    db_file: String,
    #[arg(short, long)]
    seed_urls: Vec<String>,
    #[arg(short = 'd', long)]
    seed_from_db: bool,
    #[arg(short, long, default_value_t = 10)]
    threads: usize,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    spider(args)?;

    Ok(())
}

fn init_db(file: &str) -> Result<Connection> {
    let conn = Connection::open(file)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pages(url TEXT PRIMARY KEY, type TEXT, content_id INTEGER)",
        (),
    )?;
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS page_content USING fts4(content TEXT, tokenize=unicode61)",
        (),
    )?;
    Ok(conn)
}

fn store_site(conn: &mut Connection, site: &Site) -> Result<()> {
    let tx = conn.transaction()?;
    if let Some(content) = &site.text {
        tx.execute("INSERT INTO page_content(content) VALUES(?1)", [content])?;
        tx.execute(
            "INSERT INTO pages (url, type, content_id) VALUES (?1, ?2, ?3)
             ON CONFLICT(url) DO UPDATE SET content_id=excluded.content_id",
            params![
                site.url.to_string(),
                site.url.gopher_type.to_string(),
                tx.last_insert_rowid()
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

fn store_url(conn: &mut Connection, url: &GopherURL) -> Result<()> {
    conn.execute(
        "INSERT INTO pages (url, type) VALUES (?1, ?2)",
        params![url.to_string(), url.gopher_type.to_string()],
    )?;
    Ok(())
}

fn spider(mut args: Args) -> Result<()> {
    let mut visited: HashSet<GopherURL> = HashSet::new();
    let (urls_tx, urls_rx) = unbounded();
    let (sites_tx, sites_rx) = unbounded();
    let mut workers = Vec::with_capacity(args.threads);
    let mut conn = init_db(&args.db_file)?;

    for i in 0..workers.capacity() {
        let urls_rx = urls_rx.clone();
        let sites_tx = sites_tx.clone();
        workers.push(thread::spawn(move || worker(i, urls_rx, sites_tx)));
    }

    if args.seed_from_db {
        let mut stmt =
            conn.prepare("SELECT url FROM pages WHERE type = ?1 AND content_id IS NULL")?;
        let mut count = 0;
        stmt.query_map([GopherItem::Submenu.to_string()], |row| row.get(0))?
            .for_each(|x| {
                if let Ok(x) = x {
                    log::debug!("loaded seed {x}");
                    args.seed_urls.push(x);
                    count += 1;
                }
            });
        log::info!("loaded {count} seed urls from DB");
    }

    for url in args.seed_urls {
        if let Ok(url) = GopherURL::try_from(url.as_str()) {
            visited.insert(url.clone());
            urls_tx.send(url).context("sending seed url")?;
        }
    }

    loop {
        let site = sites_rx.recv().context("receiving site")?;
        if site.url.selector.chars().filter(|c| *c == '/').count() >= 50 {
            // limit selector depth
            continue;
        }
        store_site(&mut conn, &site)
            .unwrap_or_else(|e| log::error!("[spider] storing site data: {e:#}"));
        log::info!(
            "[spider] {} urls in queue, {} urls visited",
            urls_tx.len(),
            visited.len()
        );
        for url in site.links.into_iter().flatten() {
            if !visited.insert(url.clone()) {
                continue;
            }
            store_url(&mut conn, &url)
                .unwrap_or_else(|e| log::error!("[spider] storing url {url}: {e:#}"));
            urls_tx
                .send(url)
                .unwrap_or_else(|e| log::error!("[spider] sending url to worker: {e:#}"));
        }
    }
}

fn worker(id: usize, urls: Receiver<GopherURL>, sites: Sender<Site>) {
    log::info!("worker {id} started");
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
            log::info!("[worker {id}] fetched {url}");
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
