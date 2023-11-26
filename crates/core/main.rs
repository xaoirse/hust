#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod config;
mod database;
mod file;
mod notification;

use config::{Config, Find, Sub};
use database::DataBase as db;
use memchr::memmem;
use notification::send_notification;

use std::{
    error::Error,
    fs::{self, File},
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
        search2(&find, &cfg.path);
        return Ok(String::new());
    }
    if let Some(name) = &cfg.name {
        if cfg.args.len().eq(&0) && !cfg.piped {
            search2(&Find::from(name), &cfg.path);
            Ok(String::new())
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
    status.push_str("\nConfig file: ");
    status.push_str(
        Config::get_config_file()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("None"),
    );

    Ok(status)
}
fn search(find: &Find, path: &Path) -> Result<String> {
    let mut key = "*";
    let mut args = find.args.as_slice();

    if let Some((first, rest)) = find.args.split_first() {
        if first == "ip" || first == "domain" || first == "other" {
            key = first;
            args = rest;
        }
    }

    let mut result = Vec::new();
    for entry in (fs::read_dir(path)?).flatten() {
        if entry.path().is_file() {
            continue;
        }
        if let Some(program) = &find.program {
            if !entry.file_name().eq_ignore_ascii_case(program) {
                continue;
            }
        }

        for entry in (fs::read_dir(entry.path())?).flatten() {
            if entry.path().is_dir() {
                continue;
            }

            if let Ok(vec) = std::fs::read(entry.path()) {
                let mut start = 0;
                for arg in args {
                    for end in memmem::find_iter(&vec, "\n") {
                        if memmem::find(&vec[start..end], arg.as_bytes()).is_some() {
                            if find.verbose == 0 {
                                result.push(String::from_utf8_lossy(&vec[start..end]).to_string());
                            } else {
                                result.push(format!(
                                    "{} | {}",
                                    find.program.as_ref().unwrap_or(&"".to_string()),
                                    String::from_utf8_lossy(&vec[start..end])
                                ))
                            }
                        }
                        start = end;
                    }
                }
            }
        }
    }

    // let result: Vec<String> = fs::read_dir(path)?
    //     .par_bridge()
    //     .filter_map(|d| d.ok())
    //     .filter(|d| {
    //         if let Some(p) = &find.program {
    //             d.file_name().to_string_lossy().contains(p)
    //         } else {
    //             true
    //         }
    //     })
    //     .filter_map(|de| {
    //         if let Ok(d) = fs::read_dir(de.path()) {
    //             let program = de.file_name().to_str().unwrap_or("").to_string();
    //             Some(
    //                 d.par_bridge()
    //                     .filter_map(|d| d.ok())
    //                     .filter(|d| {
    //                         if key.is_empty() {
    //                             true
    //                         } else {
    //                             d.file_name().to_string_lossy().eq(key)
    //                         }
    //                     })
    //                     .filter_map(|d| std::fs::File::open(d.path()).ok())
    //                     .flat_map_iter(|f| {
    //                         BufReader::new(f)
    //                             .lines()
    //                             .map_while(std::result::Result::ok)
    //                             .filter(|l| {
    //                                 args.is_empty() || args.iter().any(|arg| l.contains(arg))
    //                             })
    //                             .map(|l| {
    //                                 if find.verbose == 0 {
    //                                     l.trim().to_string()
    //                                 } else {
    //                                     format!("{} | {}", program, l.trim())
    //                                 }
    //                             })
    //                     })
    //                     .collect::<Vec<String>>(),
    //             )
    //         } else {
    //             None
    //         }
    //     })
    //     .flatten()
    //     .collect();

    Ok(result.join("\n"))
}

fn insert(path: PathBuf, args: Vec<String>) -> Result<String> {
    Ok(db::from(args)
        .save(path)?
        .into_iter()
        .collect::<Vec<String>>()
        .join("\n"))
}

use memmap2::{Mmap, MmapOptions};

trait Memfind {
    fn find(&self, needle: &[String]) -> Vec<&[u8]>;
}

impl Memfind for Mmap {
    fn find(&self, needles: &[String]) -> Vec<&[u8]> {
        let mut res = Vec::new();
        let mut start = 0;
        for i in 0..self.len() {
            if self[i] == b'\0' {
                return Vec::new();
            }

            if self[i] == b'\n' {
                if needles.is_empty()
                    || needles.iter().any(|needle| {
                        memchr::memmem::find(&self[start..i], needle.as_bytes()).is_some()
                    })
                {
                    res.push(&self[start..i])
                }
                start = i + 1;
                continue;
            }
        }
        res
    }
}
fn search2(find: &Find, path: &Path) {
    let mut key = "*";
    let mut args = find.args.as_slice();

    if let Some((first, rest)) = find.args.split_first() {
        if first == "ip" || first == "domain" || first == "other" {
            key = first;
            args = rest;
        }
    }

    fs::read_dir(path)
        .unwrap()
        .flatten()
        .filter(|e| {
            e.path().is_dir()
                && match &find.program {
                    Some(p) => e.path().to_string_lossy().contains(p),
                    None => true,
                }
        })
        .flat_map(|e| fs::read_dir(e.path()))
        .flatten()
        .flatten()
        .filter(|e| e.file_name() == key || key == "*")
        .flat_map(|e| File::open(e.path()))
        .for_each(|f| {
            let mmap = unsafe { MmapOptions::new().map(&f).unwrap() };

            for f in mmap.find(args) {
                println!("{}", String::from_utf8_lossy(f))
            }
        });
}
