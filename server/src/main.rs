use clap::Parser;

mod args;
use args::ServerArgs;

mod msg_exec;
use diesel::SelectableHelper;
use futures::try_join;
pub(crate) use msg_exec::MessageExecutor;
use msg_exec::{ExecNotification, Message};

mod receive_file;
pub(crate) use receive_file::receive_streamed_file;

mod db;
mod schema;

mod server;
#[cfg(feature = "mtls")]
pub(crate) use server::TlsListener;
pub(crate) use server::{Client, Server};

/// Defines names of metrics according to conventions specified at <https://prometheus.io/docs/practices/naming/#metric-names>.
mod metrics;
mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    common::tracing::init()?;

    metrics::register(prometheus::default_registry())?;

    let args = ServerArgs::parse();
    let mut listener = metrics::MeteredListener::new(get_listener(&args).await?);
    listener.set_read_metric(crate::metrics::MESSAGES_RECEIVED_BYTES.clone());
    listener.set_write_metric(crate::metrics::MESSAGES_SENT_BYTES.clone());

    tracing::info!("Listening on {}", args.common.server_address);

    let mut server = Server::new(listener);

    let (sender, receiver) = tokio::sync::mpsc::channel(8);
    let executor = MessageExecutor::new(args.root.clone()).with_notifications(sender);

    let db_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::Error::msg("DATABASE_URL not set"))?;

    let repo = db::Repository::new(&db_url)?;

    try_join!(
        persist_to_db(&db_url, receiver),
        server.run(executor),
        web::run(&args, repo),
    )?;

    Ok(())
}

async fn get_listener(args: &ServerArgs) -> anyhow::Result<impl server::Listener> {
    let listener = tokio::net::TcpListener::bind(args.common.server_address).await?;

    #[cfg(feature = "mtls")]
    let listener = {
        let certs = common::tls::load_certs(&args.mtls.cert)?;
        let priv_key = common::tls::load_keys(&args.mtls.key)?;
        let roots_store = common::tls::load_root_certs(&args.mtls.ca_cert)?;

        let verifier = rustls::server::WebPkiClientVerifier::builder(roots_store.into()).build()?;

        let tls_config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(verifier)
            .with_single_cert(certs, priv_key)?;

        TlsListener::new(listener, tls_config)
    };

    Ok(listener)
}

async fn persist_to_db(
    db_url: &str,
    mut receiver: tokio::sync::mpsc::Receiver<ExecNotification>,
) -> anyhow::Result<()> {
    use diesel_async::scoped_futures::ScopedFutureExt;
    use diesel_async::AsyncConnection;
    use diesel_async::AsyncPgConnection;
    use diesel_async::RunQueryDsl;

    let mut conn = AsyncPgConnection::establish(db_url)
        .await
        .map_err(|e| anyhow::Error::new(e).context("Failed to establish DB connection"))?;

    tracing::info!("Connected to database");

    while let Some(notification) = receiver.recv().await {
        tracing::debug!("Received notification");

        conn.transaction::<(), diesel::result::Error, _>(|conn| {
            async move {
                let row_message = db::NewMessage {
                    public_id: uuid::Uuid::new_v4(),
                    timestamp: notification.timestamp.naive_utc(),
                    user_nickname: notification.client_nickname.unwrap_or("ANON".to_string()),
                    user_ip: notification.client_ip.to_string(),
                };

                let inserted = diesel::insert_into(schema::message::table)
                    .values(&row_message)
                    .returning(db::Message::as_returning())
                    .get_results(conn)
                    .await?;

                let row_message = inserted
                    .into_iter()
                    .next()
                    .ok_or_else(|| diesel::result::Error::RollbackTransaction)?;

                match notification.message {
                    Message::Text(text) => {
                        let row_text = db::NewMessageText {
                            message_id: row_message.message_id,
                            text,
                        };

                        diesel::insert_into(schema::message_text::table)
                            .values(&row_text)
                            .execute(conn)
                            .await?;
                    }
                    Message::File {
                        filename,
                        filepath,
                        mime,
                        hash,
                        length,
                    } => {
                        let row_file = db::NewMessageFile {
                            message_id: row_message.message_id,
                            filename,
                            filepath,
                            mime,
                            length: length as i64,
                            hash: hex::encode(&hash),
                        };

                        diesel::insert_into(schema::message_file::table)
                            .values(&row_file)
                            .execute(conn)
                            .await?;
                    }
                }

                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        tracing::info!("Saved notification to DB");
    }

    Ok(())
}
