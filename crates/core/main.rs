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
use std::io::{BufRead, BufReader};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    let cfg = Config::parse();

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

                    file::append(cfg.path.join("hust.log"), &logs)?;

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
        status(cfg)
    }
}

fn status(cfg: Config) -> Result<String> {
    let mut status = String::new();
    status.push_str("Taste That PINK VENOM!");

    status.push_str("\n\nPath: ");
    status.push_str(&cfg.path.to_string_lossy());

    status.push_str("\nWebhooks:");
    for w in cfg.webhooks {
        status.push('\n');
        status.push_str(&w.to_string());
    }

    Ok(status)
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
        .filter_map(|de| {
            if let Ok(d) = fs::read_dir(de.path()) {
                let program = de.file_name().to_str().unwrap_or("").to_string();
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
                        .filter_map(|d| std::fs::File::open(d.path()).ok())
                        .flat_map_iter(|f| {
                            BufReader::new(f)
                                .lines()
                                .map_while(std::result::Result::ok)
                                .filter(|l| {
                                    args.is_empty() || args.iter().any(|arg| l.contains(arg))
                                })
                                .map(|l| {
                                    if find.verbose == 0 {
                                        l.trim().to_string()
                                    } else {
                                        format!("{} | {}", program, l.trim())
                                    }
                                })
                        })
                        .collect::<Vec<String>>(),
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
