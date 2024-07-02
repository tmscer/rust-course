mod config;
pub use config::{Config, DEFAULT_WEB_SERVER_ADDRESS};

mod docs;
mod endpoints;

mod error;
pub use error::Error;

mod repo;
pub use repo::{FullMessage, Repository};

use crate::args::ServerArgs;

pub async fn run(args: &ServerArgs, repo: impl Repository) -> anyhow::Result<()> {
    let arc_args = std::sync::Arc::new(args.clone());
    let arc_repo: std::sync::Arc<Box<dyn Repository>> = std::sync::Arc::new(Box::new(repo));

    tracing::info!("Starting web server at {}", arc_args.web.web_address);

    actix_web::HttpServer::new(move || {
        let repo = arc_repo.clone();

        let mut app = actix_web::App::new()
            // TODO: Logging of each request
            .wrap(actix_web::middleware::Logger::default())
            .app_data(actix_web::web::Data::from(repo))
            .app_data(actix_web::web::Data::from(arc_args.clone()))
            .service(endpoints::get_messages::handler)
            .service(endpoints::download::handler)
            .service(endpoints::delete_messages::handler);

        if !arc_args.web.disable_docs {
            const DOCS_PATH: &str = "/_docs";
            tracing::info!(
                "Documentation available at http://{}{DOCS_PATH}",
                arc_args.web.web_address
            );
            app = app.service(docs::endpoints("/_docs"));
        }

        app
    })
    .bind(args.web.web_address)?
    .workers(args.web.actix_num_workers)
    .run()
    .await?;

    tracing::info!("Web server stopped");

    Ok(())
}
