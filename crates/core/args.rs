use memmap2::MmapOptions;
use std::ops::Deref;
use std::{
    env::current_exe,
    ffi::OsString,
    fs::{File, OpenOptions},
    io::IsTerminal,
    path::{Path, PathBuf},
};

use crate::Result;

/// Trait to allow trimming ascii whitespace from a &[u8].
pub trait TrimAsciiWhitespace {
    /// Trim ascii whitespace (based on `is_ascii_whitespace()`) from the
    /// start and end of a slice.
    fn trim_ascii_whitespace(&self) -> &[u8];
}

impl<T: Deref<Target = [u8]>> TrimAsciiWhitespace for T {
    fn trim_ascii_whitespace(&self) -> &[u8] {
        let from = match self.iter().position(|x| !x.is_ascii_whitespace()) {
            Some(i) => i,
            None => return &self[0..0],
        };
        let to = self.iter().rposition(|x| !x.is_ascii_whitespace()).unwrap();
        &self[from..=to]
    }
}

pub struct Args {
    pub quiet: bool,
    pub notification: bool,
    pub piped: bool,
    pub args: Vec<OsString>,
    pub path: PathBuf,
    pub webhooks: Vec<OsString>,
}
impl Args {
    pub fn parse() -> Result<Self> {
        use lexopt::prelude::*;

        let mut quiet = false;
        let mut notification = false;
        let mut piped = false;
        let mut args = Vec::new();
        let mut path = PathBuf::new();
        let mut webhooks = Vec::new();

        // Parsing Config File
        let mmap = unsafe { MmapOptions::new().map(&get_config_file()?).unwrap() };
        for line in mmap.split(|c| *c == b'\n') {
            let line = line.trim_ascii_whitespace();
            if line.starts_with(b"https://discord.com/api/webhooks") {
                webhooks.push(unsafe {
                    OsString::from_encoded_bytes_unchecked(line.trim_ascii_whitespace().to_vec())
                });
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
                Short('p') | Long("path") => {
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
                    webhooks = parser.values()?.collect();
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
            args,
            path,
            webhooks,
        })
    }
}

fn get_config_file() -> Result<File> {
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

    Ok(OpenOptions::new()
        .read(true)
        .truncate(true)
        .write(true)
        .create(true)
        .open(path)?)
}
