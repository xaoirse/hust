#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod cli;

use clap::Parser;
use cli::Cli;
use database::DataBase as db;
use std::io::{IsTerminal, Read};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    // Parse CLI Parameters
    let mut cli = Cli::parse();

    // Check if somthing is piped or not
    if !std::io::stdin().is_terminal() {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .expect("Can't read from Stdin");
        let buf: Vec<_> = buf.split_whitespace().map(str::to_string).collect();
        cli.args.extend(buf);
    }

    match try_main(cli) {
        Ok(result) => print!("{result}"),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    }
}

fn try_main(cli: Cli) -> Result<String> {
    if let Some(path) = &cli.path {
        database::set_path(path)?;
    };

    if cli.name.is_none() {
        status(cli)
    } else if cli.args.len().eq(&0) {
        search(cli)
    } else {
        insert(cli)
    }
}

fn status(_cli: Cli) -> Result<String> {
    todo!("Status")
}
fn search(_cli: Cli) -> Result<String> {
    todo!("Search")
}

fn insert(cli: Cli) -> Result<String> {
    let path = cli.name.unwrap();

    Ok(db::from(cli.args)
        .save_as(&path)?
        .into_iter()
        .collect::<Vec<String>>()
        .join("\n"))
}
