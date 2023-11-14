mod args;

use args::Args;
use clap::Parser;
use database::DataBase as db;
use std::io::{IsTerminal, Read};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    // Parse CLI Parameters
    let mut args = Args::parse();

    // Check if somthing is piped or not
    if !std::io::stdin().is_terminal() {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).unwrap();
        let buf: Vec<_> = buf.split_whitespace().map(str::to_string).collect();
        args.args.extend(buf);
    }

    match try_main(args) {
        Ok(result) => println!("{result}"),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    }
}

fn try_main(args: Args) -> Result<String> {
    if args.name.is_none() {
        status(args)
    } else if args.args.len().eq(&0) {
        search(args)
    } else {
        insert(args)
    }
}

fn status(_args: Args) -> Result<String> {
    todo!()
}
fn search(_args: Args) -> Result<String> {
    todo!()
}

fn insert(args: Args) -> Result<String> {
    let path = args.name.unwrap();

    db::try_from(args.args)?.save_as(&path)?;

    Ok("Done".into())
}
