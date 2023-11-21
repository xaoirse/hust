use std::{
    env::current_exe,
    fmt::Display,
    fs::read_to_string,
    io::{IsTerminal, Read},
    path::PathBuf,
    str::FromStr,
};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "HUST", author, version, about, long_about = "Hunt Rust HUST")]
pub struct Cli {
    #[clap(short, long, global = true, help = "Quiet")]
    pub quiet: bool,

    #[clap(short, long, global = true, help = "Disable Notification")]
    pub notification: bool,

    pub name: Option<String>,

    pub args: Vec<String>,

    #[clap(short, long, help = "Set Default Path. saved in .hust.cfg")]
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
}

impl<T: AsRef<str> + Display> From<T> for Find {
    fn from(name: T) -> Self {
        Self {
            verbose: 0,
            program: None,
            args: vec![name.to_string()],
        }
    }
}

pub struct Config {
    pub quiet: bool,
    pub notification: bool,
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

        let mut file = String::new();

        // Read file
        if let Ok(p) = current_exe() {
            if let Some(p) = p.parent() {
                if let Ok(str) = read_to_string(p.join(".hust.cfg"))
                    .or(read_to_string(p.join("$HOME/.config/hust/hust.cfg")))
                    .or(read_to_string(p.join("$HOME/.hust.cfg")))
                {
                    file = str;
                }
            }
        }

        let mut path = PathBuf::from(".");
        let mut webhooks = Vec::new();

        // Parse file
        for line in file.split_whitespace() {
            if let Ok(w) = WebHook::from_str(line) {
                webhooks.push(w);
            } else {
                path = PathBuf::from(line);
            }
        }

        cli.webhooks.extend(webhooks);

        // Check if somthing is piped or not
        if !std::io::stdin().is_terminal() {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .expect("Can't read from Stdin");
            let buf: Vec<_> = buf.split_whitespace().map(str::to_string).collect();
            cli.args.extend(buf);
        }

        Self {
            quiet: cli.quiet,
            notification: cli.notification,
            name: cli.name,
            args: cli.args,
            path: cli.path.unwrap_or(path),
            webhooks: cli.webhooks,
            find: cli.find,
        }
    }
}
