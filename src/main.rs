use std::net::TcpListener;
use env_logger::Env;

mod routes;
mod startup;

use newsletter_service::configuration::{get_configuration, DatabaseSettings};
use sqlx::{PgPool, PgConnection, Connection, Pool, Postgres, Executor};
use startup::run;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, prelude::__tracing_subscriber_SubscriberExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Redirect all `log`s events to our subscriber
    LogTracer::init().expect("failed to set logger");

    // We are falling back to printing all spans at info-level or above
    // if the RUST_LOG env var has not been set for us
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = BunyanFormattingLayer::new(
        "newsletter_layer".into(),
        std::io::stdout
    );

    // The 'with' method is provided by `SubscriberExt`, an extensions
    // trait for `Subscriber` exposed by `tracing_subscriber`
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    // `set_global_default` can be used by applications to specify what subscriber should be
    // used to process spans.
    set_global_default(subscriber).expect("Fauled to set subscriber");

    let mut configuration = get_configuration().expect("Failed to read the configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let con = configure_database(&configuration.database).await;

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Could not bind");
    run(listener, con)?.await
}

pub async fn configure_database(config: &DatabaseSettings) -> Pool<Postgres> {
    let mut con = PgConnection::connect(&config.connection_string_without_db())
                                                            .await
                                                            .expect("Failled to connect");
    con.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
                .await
                .expect("Could not create Table");
    
    // Migrate DB
    let connection_pool = PgPool::connect(&config.connection_string())
                                                            .await
                                                            .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
                .run(&connection_pool)
                .await
                .expect("Failed to migrate DB");

    connection_pool
}