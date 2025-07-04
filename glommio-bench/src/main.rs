use std::time::Instant;

use futures_lite::{AsyncReadExt, AsyncWriteExt, StreamExt};
use glommio::{
    LocalExecutorBuilder,
    net::{TcpListener, TcpStream},
};
use rand::{Rng, RngCore};
use sha2::{Digest, Sha256};
use zerocopy::{FromBytes, IntoBytes};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ex = LocalExecutorBuilder::new(glommio::Placement::Fixed(0));
    let srv = ex.spawn(server)?;

    let ex = LocalExecutorBuilder::new(glommio::Placement::Fixed(1));
    let cli = ex.spawn(client)?;

    let _ = srv.join()?;
    let _ = cli.join()?;
    Ok(())
}

async fn server() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("localhost:1137").expect("starting server");
    let mut incoming_conns = listener.incoming();
    let mut buf = vec![0; 8 + 1024 * 1024];
    while let Some(stream) = incoming_conns.next().await {
        let mut stream = stream.unwrap();
        loop {
            if let Err(e) = stream.read_exact(&mut buf[0..8]).await {
                eprintln!("reading size: {e}");
                continue;
            }
            let sz = usize::read_from_bytes(&buf[0..8]).unwrap();
            if let Err(e) = stream.read_exact(&mut buf[0..sz]).await {
                eprintln!("reading payload: {e}");
                continue;
            }
            let d = Sha256::digest(&buf[0..sz]);
            if let Err(e) = stream.write_all(&d).await {
                eprintln!("sending result: {e}");
                continue;
            }
        }
    }
    Ok(())
}

async fn client() -> Result<(), std::io::Error> {
    let mut conn = TcpStream::connect("localhost:1137")
        .await
        .expect("connecting to server");
    let mut rng = rand::rng();
    let mut payload = vec![0; 8 + 1024 * 1024];
    loop {
        let sz = (rand::random::<u64>()) % (1024 * 1024);
        rng.fill_bytes(&mut payload[0..sz as usize + 8]);
        let orig_d = Sha256::digest(&payload[8..sz as usize + 8]);
        let start = Instant::now();
        sz.write_to(&mut payload[0..8]).unwrap();
        if let Err(e) = conn.write_all(&payload[0..sz as usize + 8]).await {
            eprintln!("writing payload: {e}");
            continue;
        }
        let sent = Instant::now();
        if let Err(e) = conn.read_exact(&mut payload[0..32]).await {
            eprintln!("reading result: {e}");
            continue;
        }
        if !orig_d.as_slice().eq(&payload[0..32]) {
            eprintln!("digest error")
        }

        let rtt = start.elapsed();
        let send_time = rtt - sent.elapsed();
        println!(
            "sent {sz} bytes, send time {:?}, rtt {:?}, send speed {:.2} MiB/sec, amortized speed {:.2} MiB/sec",
            send_time,
            rtt,
            sz as f64 / send_time.as_secs_f64() / 1024.0 / 1024.0,
            sz as f64 / start.elapsed().as_secs_f64() / 1024.0 / 1024.0
        );
    }
}
