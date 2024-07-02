use std::path;

/// Command-line arguments for the server.
#[derive(clap::Parser, Clone)]
pub struct ServerArgs {
    #[clap(short, long, default_value = ".")]
    pub root: path::PathBuf,
    #[clap(flatten)]
    pub common: common::cli::Args,

    #[cfg(feature = "mtls")]
    #[clap(flatten)]
    pub mtls: MtlsArgs,

    #[clap(flatten)]
    pub web: crate::web::Config,
}

#[cfg(feature = "mtls")]
#[derive(clap::Parser, Clone)]
pub struct MtlsArgs {
    /// Path to the server's certificate.
    #[clap(long, default_value = "../ssl/server-localhost.bundle.crt")]
    pub cert: path::PathBuf,

    /// Path to the server's private key.
    #[clap(long, default_value = "../ssl/server-localhost.key")]
    pub key: path::PathBuf,

    /// Path to the CA certificate used for authenticating clients.
    #[clap(long, default_value = "../ssl/ca.crt")]
    pub ca_cert: path::PathBuf,
}
