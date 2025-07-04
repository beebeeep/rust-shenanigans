use snafu::{prelude::*, Whatever};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("IO error: {e}: {source}"))]
    IO { e: String, source: std::io::Error },
}

fn main() {
    match foo().context(IOSnafu { e: "sad" }) {
        Ok(_) => println!("ok"),
        Err(e) => println!("{e}"),
    }
}

fn foo() -> Result<String, std::io::Error> {
    return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "asda"));
}
