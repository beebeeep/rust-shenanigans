mod http;

use async_fs::File;
use async_net::{TcpListener, TcpStream};
use futures_lite::io;
use smol::{
    LocalExecutor,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufWriter},
};
use snafu::{ResultExt, Whatever};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::http::{Code, Method};

#[derive(Clone)]
struct Server {
    root: Rc<str>,
}

struct Request {
    method: Method,
    client: SocketAddr,
    headers: HashMap<Box<str>, Box<str>>,
    body: Option<Box<[u8]>>,
}

struct Reply {
    code: Code,
    headers: Option<Vec<(Box<str>, Box<str>)>>,
}

impl Server {
    async fn handle_connection(
        &self,
        stream: TcpStream,
        client: SocketAddr,
    ) -> Result<(), Whatever> {
        let mut r = io::BufReader::new(stream.clone());
        let mut w = io::BufWriter::new(stream);
        let mut start = String::new();
        let mut headers = HashMap::new();
        r.read_line(&mut start)
            .await
            .whatever_context("reading request")?;

        // ** Reading HTTP message from client ** //
        // see https://datatracker.ietf.org/doc/html/rfc9112#message.format
        //
        let start = start.trim_end_matches("\r\n"); // read start-line
        loop {
            // read field lines (aka headers)
            let mut line = String::new();
            r.read_line(&mut line)
                .await
                .whatever_context("reading headers")?;
            if line == "\r\n" || line == "\n" {
                break;
            }
            let mut m = line.trim_end_matches("\r\n").split(':');
            if let (Some(k), Some(v)) = (m.next(), m.next()) {
                headers.insert(k.to_lowercase().trim().into(), v.trim().into());
            } else {
                w.write(b"HTTP/1.1 400 Bad Request\r\n")
                    .await
                    .whatever_context("writing response")?;
                return Ok(());
            }
        }

        // read body
        let mut body = None;
        if let Some(size) = headers
            .get("content-length")
            .and_then(|x: &Box<str>| x.parse::<usize>().ok())
        {
            let mut b = vec![0; size];
            r.read_exact(&mut b)
                .await
                .whatever_context("reading request body")?;
            body = Some(b.into_boxed_slice());
        }

        let mut m = start.split(' ');
        let req = match (m.next(), m.next(), m.next()) {
            (Some("GET"), Some(target), Some(_)) => Request {
                method: Method::Get(target.into()),
                client,
                headers,
                body,
            },
            _ => {
                w.write(b"HTTP/1.1 400 Bad Request\r\n")
                    .await
                    .whatever_context("writing response")?;
                return Ok(());
            }
        };

        let method = req.method.clone();
        match self.handle_request(req, w).await {
            Ok(reply) => {
                eprintln!("{:?} {} {}", method, reply.code, client);
            }
            Err(e) => {
                eprintln!("Error handling request from {}: {e:?}", client);
            }
        }

        Ok(())
    }

    async fn reply(
        &self,
        w: &mut BufWriter<TcpStream>,
        code: Code,
        headers: Option<Vec<(Box<str>, Box<str>)>>,
        body: Option<&[u8]>,
    ) -> Result<Reply, Whatever> {
        w.write(format!("HTTP/1.1 {}\r\n", code,).as_bytes())
            .await
            .whatever_context("writing header")?;

        if let Some(ref headers) = headers {
            for (k, v) in headers {
                w.write(format!("{}: {}\r\n", k, v).as_bytes())
                    .await
                    .whatever_context("writing header")?;
            }
        }
        if let Some(ref b) = body {
            w.write(format!("Content-Length: {}\r\n", b.len()).as_bytes())
                .await
                .whatever_context("writing content length")?;
        }
        w.write("\r\n".as_bytes())
            .await
            .whatever_context("writing header")?;
        if let Some(body) = body {
            w.write(body).await.whatever_context("writing body")?;
        }

        w.flush().await.whatever_context("flushing buffer")?;
        Ok(Reply { code, headers })
    }

    fn get_file_path(&self, path: &str) -> Option<impl AsRef<Path>> {
        let mut p = PathBuf::from(self.root.as_ref());
        for part in path.split('/') {
            match part {
                ".." => return None,
                "." => continue,
                part => p.push(part),
            }
        }

        Some(p)
    }

    async fn handle_request(
        &self,
        req: Request,
        mut w: BufWriter<TcpStream>,
    ) -> Result<Reply, Whatever> {
        if req.headers.get("host").is_none() {
            self.reply(&mut w, Code::BadRequest, None, None).await?;
        }

        match req.method {
            Method::Get(target) => {
                let path = match self.get_file_path(&target) {
                    Some(p) => p,
                    None => {
                        return self
                            .reply(
                                &mut w,
                                Code::NotFound,
                                None,
                                Some("file not found".as_bytes()),
                            )
                            .await;
                    }
                };

                match File::open(path).await {
                    Ok(f) => {
                        let meta = match f.metadata().await {
                            Ok(meta) => meta,
                            Err(e) => {
                                return self.reply(&mut w, Code::from(e), None, None).await;
                            }
                        };

                        let reply = self
                            .reply(
                                &mut w,
                                Code::Ok,
                                Some(vec![(
                                    Box::from("Content-Length"),
                                    Box::from(format!("{}", meta.len())),
                                )]),
                                None,
                            )
                            .await?;
                        io::copy(f, w).await.whatever_context("writing file")?;
                        Ok(reply)
                    }
                    Err(e) => {
                        return self.reply(&mut w, Code::from(e), None, None).await;
                    }
                }
            }
        }
    }
}

fn main() {
    let ex = LocalExecutor::new();
    smol::block_on(ex.run(async {
        let addr = ("::0", 8080);
        let listener = TcpListener::bind(addr).await.expect("binding to the port");
        let server = Server {
            root: Rc::from("./"),
        };
        loop {
            if let Ok((stream, peer_addr)) = listener.accept().await {
                let s = server.clone();
                if let Err(e) = ex
                    .spawn(async move { s.handle_connection(stream, peer_addr).await })
                    .await
                {
                    eprintln!("handling connection: {e}");
                }
            }
        }
    }));
}
