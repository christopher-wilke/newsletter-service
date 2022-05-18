use actix_web::{HttpServer, App, web, dev::Server};
use sqlx::{PgPool};
use tracing_actix_web::TracingLogger;
use std::net::TcpListener;

use crate::{routes::*, email_client::EmailClient};

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: EmailClient
) -> Result<Server, std::io::Error> {

    let con = web::Data::new(connection);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(con.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}