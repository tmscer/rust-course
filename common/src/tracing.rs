use tracing::Level;
use tracing_subscriber::FmtSubscriber;

/// Initializes tracing with a `tracing_subscriber::FmtSubscriber` that logs to stdout.
pub fn init() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
