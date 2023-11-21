#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod config;
mod database;
mod file;
mod notification;

use config::{Config, Find, Sub};
use database::DataBase as db;
use notification::send_notification;

use rayon::prelude::*;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    let cfg = Config::parse();

    set_path(&cfg.path).unwrap();

    match run(cfg) {
        Ok(result) => {
            print!("{}", result.trim());
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    }
}

fn run(cfg: Config) -> Result<String> {
    if let Some(Sub::Find(find)) = cfg.find {
        return search(&find, &cfg.path);
    }
    if let Some(name) = &cfg.name {
        if cfg.args.len().eq(&0) {
            search(&Find::from(name), &cfg.path)
        } else {
            match insert(cfg.path.join(name), cfg.args) {
                Ok(result) => {
                    if result.trim().is_empty() {
                        return Ok(result);
                    }

                    // Prepare results for append to hust.log
                    let time = chrono::Local::now();
                    let logs: Vec<String> = result
                        .lines()
                        .map(|line| format!("{} | {} | {}", name, line, time))
                        .collect();

                    file::append(&logs, cfg.path.join("hust.log"))?;

                    if !cfg.notification {
                        if let Err(err) = send_notification(
                            cfg.webhooks,
                            format!("## {}:\n{result}", cfg.name.unwrap_or_default()),
                        ) {
                            eprintln!("{err}");
                        }
                    }

                    Ok(result)
                }
                Err(err) => Err(err),
            }
        }
    } else {
        status()
    }
}

fn status() -> Result<String> {
    Ok("HOW YOU LIKE THAT?".to_string())
}
fn search(find: &Find, path: &Path) -> Result<String> {
    let mut key = "";
    let mut args = find.args.as_slice();

    if let Some((first, rest)) = find.args.split_first() {
        if first == "ip" || first == "domain" || first == "other" {
            key = first;
            args = rest;
        }
    }

    let result: Vec<String> = fs::read_dir(path)?
        .par_bridge()
        .filter_map(|d| d.ok())
        .filter(|d| {
            if let Some(p) = &find.program {
                d.file_name().to_string_lossy().contains(p)
            } else {
                true
            }
        })
        .filter_map(|d| {
            let program = d.file_name();

            if let Ok(d) = fs::read_dir(d.path()) {
                Some(
                    d.par_bridge()
                        .filter_map(|d| d.ok())
                        .filter(|d| {
                            if key.is_empty() {
                                true
                            } else {
                                d.file_name().to_string_lossy().eq(key)
                            }
                        })
                        .filter_map(|d| fs::read_to_string(d.path()).ok())
                        .flat_map(move |s| {
                            s.par_lines()
                                .filter(|l| {
                                    args.is_empty() || args.par_iter().any(|arg| l.contains(arg))
                                })
                                .map(|l| {
                                    if find.verbose == 0 {
                                        l.trim().to_string()
                                    } else {
                                        format!("{} | {}", program.to_string_lossy(), l.trim())
                                    }
                                })
                                .collect::<Vec<String>>()
                        }),
                )
            } else {
                None
            }
        })
        .flatten()
        .collect();

    Ok(result.join("\n"))
}

fn insert(path: PathBuf, args: Vec<String>) -> Result<String> {
    Ok(db::from(args)
        .save(path)?
        .into_iter()
        .collect::<Vec<String>>()
        .join("\n"))
}

pub fn set_path(path: &Path) -> Result<()> {
    file::save("./.hust.cfg", &[path.to_str().unwrap_or_default()])
}
