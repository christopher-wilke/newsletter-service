use actix_web::{HttpServer, App, web, dev::Server};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing_actix_web::TracingLogger;
use std::{net::TcpListener};

use crate::{routes::*, email_client::EmailClient, configuration::Settings};

pub async fn build(configuration: Settings) -> Result<Server, std::io::Error> {

    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        std::time::Duration::from_millis(100)
    );

    let address = format!(
        "{}:{}",
        configuration.application.host,
        configuration.application.port
    );

    let listener = TcpListener::bind(address)?;
    run(
        listener,
        connection_pool,
        email_client
    )
}

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