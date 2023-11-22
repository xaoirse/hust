use clap::{Parser, Subcommand};
use std::{
    env::current_exe,
    fmt::Display,
    fs::read_to_string,
    io::{IsTerminal, Read},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::file;

#[derive(Debug, Parser)]
#[clap(name = "HUST", author, version, about, long_about = "Hunt Rust HUST")]
pub struct Cli {
    #[clap(short, long, global = true, help = "Quiet")]
    pub quiet: bool,

    #[clap(short, long, global = true, help = "Disable Notification")]
    pub notification: bool,

    pub name: Option<String>,

    pub args: Vec<String>,

    #[clap(long, help = "Set Default Path. saved in .hust.cfg")]
    pub path: Option<PathBuf>,

    #[clap(short, long, help = "WebHooks")]
    pub webhooks: Vec<WebHook>,

    #[clap(subcommand)]
    pub find: Option<Sub>,
}

#[derive(Subcommand, Debug)]
pub enum Sub {
    Find(Box<Find>),
}

#[derive(Parser, Debug)]
pub struct Find {
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,

    pub args: Vec<String>,

    #[clap(short, long, help = "Program")]
    pub program: Option<String>,

    #[clap(short, default_value = "1000000000", long, help = "number of results")]
    pub number: usize,
}

impl<T: AsRef<str> + Display> From<T> for Find {
    fn from(name: T) -> Self {
        Self {
            number: 1000000000,
            verbose: 0,
            program: None,
            args: vec![name.to_string()],
        }
    }
}

pub struct Config {
    pub quiet: bool,
    pub notification: bool,
    pub piped: bool,
    pub name: Option<String>,
    pub args: Vec<String>,
    pub path: PathBuf,
    pub webhooks: Vec<WebHook>,
    pub find: Option<Sub>,
}

#[derive(Debug)]
pub enum WebHook {
    Discord(String),
}

impl Display for WebHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebHook::Discord(url) => write!(f, "Discord => {}", url),
        }
    }
}

impl FromStr for WebHook {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().starts_with("https://discord.com/api/webhooks") {
            Ok(WebHook::Discord(s.trim().to_string()))
        } else {
            Err(format!("'{s}' is not a WebHook!"))
        }
    }
}

impl Config {
    pub fn parse() -> Self {
        let mut cli = Cli::parse();

        let mut config_file = String::new();

        let mut path = match cli.path {
            Some(path) => {
                if path.is_dir() {
                    if let Err(err) = write_path(&path) {
                        eprintln!("{err}")
                    }
                    path
                } else {
                    ".".into()
                }
            }
            None => ".".into(),
        };

        if let Some(file) = Self::get_config_file() {
            if let Ok(str) = read_to_string(file) {
                config_file = str
            }
        }

        let mut webhooks = Vec::new();

        // Parse file
        for line in config_file.split_whitespace() {
            if let Ok(w) = WebHook::from_str(line) {
                webhooks.push(w);
            } else {
                path = PathBuf::from(line);
            }
        }

        cli.webhooks.extend(webhooks);

        let mut piped = false;
        // Check if somthing is piped or not
        if !std::io::stdin().is_terminal() {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .expect("Can't read from Stdin");
            let buf: Vec<_> = buf.split_whitespace().map(str::to_string).collect();
            cli.args.extend(buf);
            piped = !piped;
        }

        Self {
            quiet: cli.quiet,
            notification: cli.notification,
            piped,
            name: cli.name,
            args: cli.args,
            path,
            webhooks: cli.webhooks,
            find: cli.find,
        }
    }

    pub fn get_config_file() -> Option<PathBuf> {
        if let Ok(p) = current_exe() {
            if let Some(p) = p.parent() {
                if p.join(".hust.cfg").is_file() {
                    return Some(p.join(".hust.cfg"));
                } else if Path::new("$HOME/.config/hust/hust.cfg").is_file() {
                    return Some(PathBuf::from("$HOME/.config/hust/hust.cfg"));
                } else if Path::new("$HOME/.hust.cfg").is_file() {
                    return Some(PathBuf::from("$HOME/.hust.cfg"));
                }
            }
            return Some(p.join(".hust.cfg"));
        }
        None
    }
}

pub fn write_path(path: &Path) -> crate::Result<()> {
    if let Some(config_file) = Config::get_config_file() {
        file::append(config_file, &[path.to_str().unwrap_or_default()])
    } else {
        Err("Can't get config file".into())
    }
}
