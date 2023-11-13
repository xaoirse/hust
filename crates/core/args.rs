use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "Hust",author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, global = true, help = "Quiet")]
    pub quiet: bool,

    pub name: Option<String>,

    pub args: Vec<String>,
    // #[clap(subcommand)]
    // pub subcmd: Option<Subcmd>,
}

// #[derive(Debug, Subcommand)]
// pub enum Subcmd {
//     To(To),
// }

// #[derive(Debug, Parser)]
// pub struct To {
//     #[clap(ignore_case = true, help = "Case Insensitive")]
//     program: String,
//     assets: Vec<Asset>,
// }
