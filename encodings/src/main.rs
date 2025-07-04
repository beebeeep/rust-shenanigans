use std::{
    io::{Read, Write},
    str::{self, FromStr, from_utf8},
    time::Duration,
};

use bytes::{Buf, BufMut, BytesMut};
use futures_lite::{AsyncWrite, AsyncWriteExt, FutureExt};
use glommio::{
    io::{DmaStreamWriterBuilder, OpenOptions},
    prelude::*,
    timer::sleep,
};
use prost::Message;

pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/encodings.rs"));
}
fn main() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let msg1 = pb::Chlos {
        action: pb::Action::Hear as i32,
        timestamp: Some(prost_types::Timestamp {
            seconds: ts.as_secs() as i64,
            nanos: ts.subsec_nanos() as i32,
        }),
        comment: "CHLOS".to_string(),
        value: vec![42; 256],
        count: 137,
    };
    let mut buf = bytes::BytesMut::with_capacity(1024);
    msg1.encode_length_delimited(&mut buf).unwrap();
    println!("wrote 1st message, buffer {}", buf.len());

    let msg2 = pb::Chlos {
        action: pb::Action::See as i32,
        timestamp: None,
        comment: "jump da fuck up".to_string(),
        value: vec![0; 128],
        count: 42,
    };
    msg2.encode_length_delimited(&mut buf).unwrap();
    println!("wrone 2nd message, buffer {}", buf.len());

    let buf2 = buf.clone();
    let len = prost::decode_length_delimiter(buf2).unwrap();
    let l = prost::decode_length_delimiter(&mut buf).unwrap();
    println!("next msg len: {}, buffer {}", l, buf.len());
    let decoded = pb::Chlos::decode(buf.split_to(l)).unwrap();
    println!("buffer {}\nmsg {decoded:?}", buf.len());
    let decoded = pb::Chlos::decode_length_delimited(&mut buf).unwrap();
    println!(
        "buffer {}, cloned buffer {}\nmsg {decoded:?}",
        buf.len(),
        buf2.len()
    );

    let mut b = BytesMut::from(&"123456"[..]);
    let mut a = [0u8; 3];
    b.copy_to_slice(&mut a);
    println!("{}", from_utf8(&a).unwrap());
    let c = b.chain(BytesMut::from(&"7890"[..]));
    let mut v = Vec::new();
    c.reader().read_to_end(&mut v).unwrap();
    println!("{}", String::from_utf8(v).unwrap());

    /*
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open("./test.txt")
        .unwrap();
    f.write_all(b"ALALALAL").unwrap();
    f.sync_all().unwrap();
    */
    let ex = LocalExecutor::default();
    ex.run(async {
        let f = OpenOptions::new()
            .read(true)
            .buffered_open("test.txt")
            .await
            .unwrap();

        let mut pos = 0;
        loop {
            let r = f.read_at(pos, 64).await.unwrap();
            println!("read: {}", String::from_utf8(r.to_vec()).unwrap());
            println!("file size: {}", f.file_size().await.unwrap());
            pos += r.len() as u64;
            sleep(Duration::from_secs(1)).await;
        }
        /*
        let f = OpenOptions::new()
            .write(true)
            .append(true)
            .dma_open("test.txt")
            .await
            .unwrap();
        println!("size {}", f.file_size().await.unwrap());
        let mut s = DmaStreamWriterBuilder::new(f)
            .with_write_behind(1)
            .with_buffer_size(512)
            .build();
        let data = [65u8; 10];
        s.write_all(&data[..]).await.unwrap();
        s.close().await.unwrap();
        */

        /*
        let f = OpenOptions::new()
            .write(true)
            .append(true)
            .dma_open("./test.txt")
            .await
            .unwrap();
        let mut s = DmaStreamWriterBuilder::new(f)
            .with_write_behind(1)
            .with_buffer_size(4096)
            .build();
        let data = b"CHLOS CHLOS CHLOS";
        s.write_all(data).await.unwrap();
        s.close().await.unwrap();
        */
    });
}
