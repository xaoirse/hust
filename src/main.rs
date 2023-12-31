mod args;
mod database;
mod notification;
mod utils;
use cidr_utils::cidr::IpCidr;
use utils::Memfind;

use database::DataBase as db;

use args::{Args, Webhook};
use itertools::Itertools;
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
            b"domain" | b"ip" => search(&args.path, &args.program, first, rest, args.verbosity),
            b"log" => todo!(), //TODO
            _ => {
                match args.program {
                    Some(program) => insert(
                        args.path,
                        program,
                        args.args,
                        args.notification,
                        args.webhooks,
                    )?,
                    None => return Err("Program (-p) must be specified!".into()),
                };
                Ok(())
            }
        },
        None => status(args),
    }
}

const BANNER: &str = "
    ██░ ██  █    ██   ██████ ▄▄▄█████▓
    ▓██░ ██▒ ██  ▓██▒▒██    ▒ ▓  ██▒ ▓▒
    ▒██▀▀██░▓██  ▒██░░ ▓██▄   ▒ ▓██░ ▒░
    ░▓█ ░██ ▓▓█  ░██░  ▒   ██▒░ ▓██▓ ░ 
    ░▓█▒░██▓▒▒█████▓ ▒██████▒▒  ▒██▒ ░ 
    ▒ ░░▒░▒░▒▓▒ ▒ ▒ ▒ ▒▓▒ ▒ ░  ▒ ░░   
    ▒ ░▒░ ░░░▒░ ░ ░ ░ ░▒  ░ ░    ░    
    ░  ░░ ░ ░░░ ░ ░ ░  ░  ░    ░      
    ░  ░  ░   ░           ░            
";
fn status(cfg: Args) -> Result<()> {
    let mut stdout = std::io::stdout();
    write!(
        stdout,
        "{}\n        Taste That PINK VENOM!    \n\nHunt Path: {}\n\nWebhooks: {}\n\nConfig file: {}",
        BANNER,
        &cfg.path.to_string_lossy(),
        cfg.webhooks
            .iter()
            .join("             \n"),
        args::get_config_file()?.0.to_string_lossy()
    )?;

    Ok(())
}

fn insert(
    path: PathBuf,
    program: OsString,
    args: Vec<OsString>,
    notification: bool,
    webhooks: Vec<Webhook>,
) -> Result<()> {
    let mut db = db::init(&path, &program)?.import(args, true);

    db.write()?;

    let args = db.new.1;
    if !args.is_empty() {
        let append_res = utils::append(
            path.join("hust.log"),
            &args
                .iter()
                .map(|str| {
                    format!(
                        "{} | {} | {}",
                        program.to_string_lossy(),
                        str.to_string_lossy(),
                        chrono::Local::now().to_rfc2822()
                    )
                })
                .join("\n"),
        );

        if !notification {
            send_notification(
                webhooks,
                format!(
                    "## {}\n{}",
                    program.to_string_lossy(),
                    args.iter().map(|str| str.to_string_lossy()).join("\n"),
                ),
            )?;
        }

        append_res?;
    }
    Ok(())
}

fn search(
    path: &Path,
    program: &Option<OsString>,
    first: &OsString,
    args: &[OsString],
    v: bool,
) -> Result<()> {
    fs::read_dir(path)?
        .flatten()
        .filter(|e| {
            e.path().is_dir()
                && match (program, e.path().file_name()) {
                    (Some(program), Some(path)) => program == path,
                    _ => true,
                }
        })
        .for_each(|program| {
            if let Ok(e) = read_dir(program.path()) {
                let iter = e.flatten().filter(|e| &e.file_name() == first);
                if first == "domain" {
                    iter.flat_map(|e| File::open(e.path())).for_each(|f| {
                        let mmap = unsafe { MmapOptions::new().map(&f).unwrap() };

                        for arg in mmap.find(args) {
                            if v {
                                println!(
                                    "{} | {}",
                                    program.file_name().to_string_lossy(),
                                    String::from_utf8_lossy(arg)
                                );
                            } else {
                                println!("{}", String::from_utf8_lossy(arg));
                            }
                        }
                    });
                } else {
                    // Search in CIDRs
                    iter.flat_map(|e| File::open(e.path())).for_each(|f| {
                        let mmap = unsafe { MmapOptions::new().map(&f).unwrap() };

                        let args = args
                            .iter()
                            .filter_map(|arg| IpCidr::from_str(arg.to_string_lossy()).ok())
                            .collect_vec();

                        for line in mmap.find(&[]) {
                            if let Ok(cidr) =
                                IpCidr::from_str(unsafe { std::str::from_utf8_unchecked(line) })
                            {
                                for arg in args.iter() {
                                    if cidr.contains(arg.first_as_ip_addr())
                                        || cidr.contains(arg.last_as_ip_addr())
                                    {
                                        if v {
                                            println!(
                                                "{} | {} | {}",
                                                program.file_name().to_string_lossy(),
                                                cidr,
                                                arg
                                            );
                                        } else {
                                            println!("{}", cidr);
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            }
        });

    Ok(())
}
