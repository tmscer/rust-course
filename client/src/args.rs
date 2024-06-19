#[derive(clap::Parser)]
pub struct ClientArgs {
    #[clap(short, long = "nick")]
    pub nickname: String,
    #[clap(flatten)]
    pub common: common::cli::Args,
}
