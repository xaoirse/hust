use memmap2::MmapOptions;
use std::{
    borrow::Cow,
    env::current_exe,
    ffi::OsString,
    fs::{File, OpenOptions},
    io::IsTerminal,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use crate::{utils::TrimAsciiWhitespace, Result};

pub enum Webhook {
    Discord(OsString),
}
impl TryFrom<&[u8]> for Webhook {
    type Error = ();

    fn try_from(value: &[u8]) -> std::prelude::v1::Result<Self, Self::Error> {
        if value.starts_with(b"https://discord.com/api/webhooks") {
            Ok(Self::Discord(unsafe {
                OsString::from_encoded_bytes_unchecked(value.trim_ascii_whitespace().to_vec())
            }))
        } else {
            Err(())
        }
    }
}

impl Webhook {
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        match self {
            Webhook::Discord(url) => url.to_string_lossy(),
        }
    }
}

pub struct Args {
    pub quiet: bool,
    pub notification: bool,
    pub piped: bool,
    pub verbosity: bool,
    pub program: Option<OsString>,
    pub args: Vec<OsString>,
    pub path: PathBuf,
    pub webhooks: Vec<Webhook>,
}
impl Args {
    pub fn parse() -> Result<Self> {
        use lexopt::prelude::*;

        let mut quiet = false;
        let mut notification = false;
        let mut piped = false;
        let mut verbosity = false;
        let mut program = None;
        let mut args = Vec::new();
        let mut path = PathBuf::from(".");
        let mut webhooks = Vec::new();

        // Parsing Config File
        let mmap = unsafe { MmapOptions::new().map(&get_config_file()?.1).unwrap() };
        for line in mmap.split(|c| *c == b'\n') {
            let line = line.trim_ascii_whitespace();
            if let Ok(webhook) = Webhook::try_from(line) {
                webhooks.push(webhook);
            } else {
                let line = unsafe {
                    OsString::from_encoded_bytes_unchecked(line.trim_ascii_whitespace().to_vec())
                };

                if Path::new(&line).is_dir() {
                    path = PathBuf::from(&line);
                }
            }
        }

        // Parsing Cli
        let mut parser = lexopt::Parser::from_env();
        while let Some(arg) = parser.next()? {
            match arg {
                Short('q') | Long("quiet") => {
                    quiet = true;
                }
                Short('n') | Long("notification") => {
                    notification = true;
                }
                Short('v') | Long("verbosity") => {
                    verbosity = true;
                }
                Short('p') | Long("program") => {
                    program = Some(parser.value()?);
                }
                Long("path") => {
                    let str: OsString = parser.value()?;
                    if Path::new(&str).is_dir() {
                        path = PathBuf::from(&str);
                    } else {
                        return Err(format!(
                            "{:?} is not a Directory.\n{:?} will be used!",
                            str, path
                        )
                        .into());
                    }
                }

                Short('w') | Long("webhooks") => {
                    webhooks = parser
                        .values()?
                        .flat_map(|str| match Webhook::try_from(str.as_bytes()) {
                            Ok(wh) => Some(wh),
                            Err(_) => {
                                eprintln!("{str:?} is not a supported webhook");
                                None
                            }
                        })
                        .collect();
                }

                Value(val) => args.push(val),
                Short('h') | Long("help") => {
                    println!("It's BLACK and it's PINK once the sun down...");
                    std::process::exit(0);
                }
                _ => return Err(arg.unexpected().into()),
            }
        }

        // Check if somthing is piped or not
        if !std::io::stdin().is_terminal() {
            let mmap = unsafe { MmapOptions::new().map(&std::io::stdin()).unwrap() };

            mmap.split(|c| c.is_ascii_whitespace())
                .map(|bytes| unsafe { OsString::from_encoded_bytes_unchecked(bytes.to_vec()) })
                .for_each(|str| args.push(str));

            piped = true;
        }

        Ok(Args {
            quiet,
            notification,
            piped,
            verbosity,
            program,
            args,
            path,
            webhooks,
        })
    }
}

pub fn get_config_file() -> Result<(PathBuf, File)> {
    let path = if let Ok(p) = current_exe() {
        if let Some(p) = p.parent() {
            if p.join(".hust.cfg").is_file() {
                p.join(".hust.cfg")
            } else if Path::new("$HOME/.config/hust/hust.cfg").is_file() {
                PathBuf::from("$HOME/.config/hust/hust.cfg")
            } else if Path::new("$HOME/.hust.cfg").is_file() {
                PathBuf::from("$HOME/.hust.cfg")
            } else {
                p.join(".hust.cfg")
            }
        } else {
            PathBuf::from(".hust.cfg")
        }
    } else {
        PathBuf::from(".hust.cfg")
    };

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(&path)?;

    Ok((path, file))
}
