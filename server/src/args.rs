use std::path;

#[derive(clap::Parser)]
pub struct ServerArgs {
    #[clap(short, long, default_value = ".")]
    pub root: path::PathBuf,
    #[clap(flatten)]
    pub common: common::cli::Args,
}
