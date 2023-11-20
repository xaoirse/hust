use std::{
    env::current_exe,
    fs::read_to_string,
    io::{IsTerminal, Read},
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "HUST", author, version, about, long_about = "Hunt Rust HUST")]
pub struct Cli {
    #[clap(short, long, global = true, help = "Quiet")]
    pub quiet: bool,

    pub name: Option<String>,

    pub args: Vec<String>,

    #[clap(short, long, help = "Set Default Path. saved in .hust.cfg")]
    pub path: Option<PathBuf>,

    #[clap(short, long, help = "WebHooks")]
    pub webhooks: Vec<WebHook>,
}

pub struct Config {
    pub quiet: bool,

    pub name: Option<String>,

    pub args: Vec<String>,

    pub path: PathBuf,

    pub webhooks: Vec<WebHook>,
}

#[derive(Debug)]
pub enum WebHook {
    Discord(String),
}

impl FromStr for WebHook {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("https://discord.com/api/webhooks") {
            Ok(WebHook::Discord(s.to_string()))
        } else {
            Err(format!("'{s}' is not a webhook!"))
        }
    }
}

impl Config {
    pub fn parse() -> Self {
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

        for line in file.split_whitespace() {
            if line.trim().starts_with("https://discord.com/api/webhooks") {
                webhooks.push(WebHook::Discord(line.trim().to_string()));
            } else {
                path = PathBuf::from(line);
            }
        }

        let mut cli = Cli::parse();

        // Check if somthing is piped or not
        if !std::io::stdin().is_terminal() {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .expect("Can't read from Stdin");
            let buf: Vec<_> = buf.split_whitespace().map(str::to_string).collect();
            cli.args.extend(buf);
        }

        cli.webhooks.extend(webhooks);

        Self {
            quiet: cli.quiet,
            name: cli.name,
            args: cli.args,
            path: cli.path.unwrap_or(path),
            webhooks: cli.webhooks,
        }
    }
}
