#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod args;
mod database;
mod notification;
mod utils;
use utils::Memfind;

use database::DataBase as db;

use args::{Args, Webhook};
use itertools::Itertools;
use memchr::memmem;
use memmap2::MmapOptions;
use notification::send_notification;

use std::{
    error::Error,
    ffi::OsString,
    fs::{self, read_dir, File},
    io::Write,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    let cfg = Args::parse().unwrap();

    if let Err(err) = run(cfg) {
        eprintln!("{err}");
        std::process::exit(2);
    }
}

fn run(args: Args) -> Result<()> {
    match args.args.split_first() {
        Some((first, rest)) => match first.as_bytes() {
            b"ip" | b"domain" => search(&args.path, &args.program, first, rest, args.verbosity),
            b"log" => todo!(), // TODO
            _ => {
                match args.program {
                    Some(program) => insert(args.path, program, &args.args, args.webhooks)?,
                    None => return Err("Program (-p) must be specified!".into()),
                };
                Ok(())
            }
        },
        None => status(args),
    }
}

fn status(cfg: Args) -> Result<()> {
    let mut stdout = std::io::stdout();
    write!(
        stdout,
        "Taste That PINK VENOM!\n\n  Hunt Path: {}\n\n   Webhooks: {}\n\nConfig file: {}",
        &cfg.path.to_string_lossy(),
        cfg.webhooks
            .iter()
            .map(|w| w.to_string_lossy())
            .join("             \n"),
        args::get_config_file()?.0.to_string_lossy()
    )?;

    Ok(())
}

fn insert(
    path: PathBuf,
    program: OsString,
    args: &[OsString],
    webhooks: Vec<Webhook>,
) -> Result<()> {
    let mut db = db::init(path, &program)?;

    send_notification(
        webhooks,
        db.import(args)?
            .iter()
            .map(|str| str.to_string_lossy())
            .join("\n"),
    )?;

    db.write()?;

    Ok(())
}

fn search(
    path: &Path,
    program: &Option<OsString>,
    first: &OsString,
    args: &[OsString],
    v: bool,
) -> Result<()> {
    // for dir in fs::read_dir(path)? {
    //     if let Ok(e) = dir {
    //         if e.path().is_dir() {
    //             if let (Some(prog), Some(path)) = (program, e.path().file_name()) {
    //                 if memmem::find(path.as_bytes(), prog.as_bytes()).is_none() {
    //                     continue;
    //                 }

    //                 if let Ok(e) = read_dir(e.path()) {
    //                     for e in e {
    //                         if let Ok(e) = e {
    //                             if first.as_bytes() == b"*" || &e.file_name() == first {
    //                                 if let Ok(f) = File::open(e.path()) {
    //                                     let mmap = unsafe { MmapOptions::new().map(&f).unwrap() };
    //                                     for f in mmap.find(args) {
    //                                         if v {
    //                                             println!(
    //                                                 "{} {}",
    //                                                 e.file_name().to_string_lossy(),
    //                                                 String::from_utf8_lossy(f)
    //                                             );
    //                                         } else {
    //                                             println!("{}", String::from_utf8_lossy(f));
    //                                         }
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    fs::read_dir(path)?
        .flatten()
        .filter(|e| {
            e.path().is_dir()
                && match (program, e.path().file_name()) {
                    (Some(program), Some(path)) => {
                        memmem::find(path.as_bytes(), program.as_bytes()).is_some()
                    }
                    _ => true,
                }
        })
        .for_each(|program| {
            if let Ok(e) = read_dir(program.path()) {
                e.flatten()
                    .filter(|e| first.as_bytes() == b"*" || &e.file_name() == first)
                    .flat_map(|e| File::open(e.path()))
                    .for_each(|f| {
                        let mmap = unsafe { MmapOptions::new().map(&f).unwrap() };

                        for f in mmap.find(args) {
                            if v {
                                println!(
                                    "{} {}",
                                    program.file_name().to_string_lossy(),
                                    String::from_utf8_lossy(f)
                                );
                            } else {
                                println!("{}", String::from_utf8_lossy(f));
                            }
                        }
                    });
            }
        });

    Ok(())
}
