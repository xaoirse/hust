mod args;
mod assets;

use args::Args;
use assets::Assets;
use clap::Parser;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    let args = Args::parse();

    if let Err(err) = try_main(args) {
        eprintln!("{err}");
        std::process::exit(2);
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
    let program = args.name.unwrap();

    Assets::try_from(args.args)?.save_as(&program)?;

    Ok("Done".into())
}
