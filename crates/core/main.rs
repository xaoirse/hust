#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod config;
mod database;
mod file;
mod notification;

use config::Config;
use database::DataBase as db;
use notification::send_notification;

use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() {
    let cfg = Config::parse();

    let quiet = cfg.quiet;

    set_path(&cfg.path).unwrap();

    match run(cfg).await {
        Ok(result) => {
            if !quiet {
                print!("{}", result.trim());
            }
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    }
}

async fn run(cfg: Config) -> Result<String> {
    if let Some(name) = &cfg.name {
        if cfg.args.len().eq(&0) {
            search(name)
        } else {
            match insert(cfg.path.join(name), cfg.args) {
                Ok(result) => {
                    if result.trim().is_empty() {
                        return Ok(result);
                    }

                    let logs: Vec<String> = result
                        .lines()
                        .map(|line| {
                            let str = format!("{} | {} | {}", name, line, chrono::Local::now());
                            str
                        })
                        .collect();

                    file::append(&logs, cfg.path.join("hust.log"))?;

                    if let Err(err) = send_notification(
                        cfg.webhooks,
                        format!("## {}:\n{result}", cfg.name.unwrap_or_default()),
                    )
                    .await
                    {
                        eprintln!("{err}");
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
    Ok("Hello World!".to_string())
}
fn search(_str: &str) -> Result<String> {
    todo!("Search")
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
