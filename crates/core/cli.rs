use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "HUST", author, version, about, long_about = "Hunt Rust HUST")]
pub struct Cli {
    #[clap(short, long, global = true, help = "Quiet")]
    pub quiet: bool,

    pub name: Option<String>,

    pub args: Vec<String>,

    #[clap(short, long, help = "Set Default Path. saved in .hust.cfg")]
    pub path: Option<String>,
}
