use std::path;

/// Command-line arguments for the client.
#[derive(clap::Parser)]
pub struct ClientArgs {
    #[clap(short, long = "nick")]
    pub nickname: String,
    #[clap(flatten)]
    pub common: common::cli::Args,

    #[cfg(feature = "mtls")]
    #[clap(flatten)]
    pub mtls: MtlsArgs,
}

#[cfg(feature = "mtls")]
#[derive(clap::Parser)]
pub struct MtlsArgs {
    /// Domain to require from the server.
    #[clap(long, default_value = "localhost")]
    pub cert_domain: String,

    /// Path to the client's certificate.
    #[clap(long, default_value = "../ssl/client1.crt")]
    pub cert: path::PathBuf,

    /// Path to the client's private key.
    #[clap(long, default_value = "../ssl/client1.key")]
    pub key: path::PathBuf,

    /// Path to the CA certificate.
    #[clap(long, default_value = "../ssl/ca.crt")]
    pub ca_cert: path::PathBuf,
}
