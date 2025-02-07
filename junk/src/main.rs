use std::{thread, time::Duration};

fn main() {
    let h = thread::spawn(boo);
    h.join().unwrap();
}

fn boo() {
    loop {
        println!("boo");
        thread::sleep(Duration::from_secs(1));
    }
}
