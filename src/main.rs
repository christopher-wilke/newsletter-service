use std::net::TcpListener;

mod routes;
mod startup;

use newsletter_service::configuration::{get_configuration, DatabaseSettings};
use sqlx::{PgPool, PgConnection, Connection, Pool, Postgres, Executor};
use startup::run;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, prelude::__tracing_subscriber_SubscriberExt};
use uuid::Uuid;

/// Compose multiple layers into a `tracing`'s subscriber.
/// 
/// # Implementation Notes
/// 
/// We are using `impl Subscriber` as return type to avoid having
/// to spell out the actualy type of the returned subscriber, which is need quite complex.
/// 
/// We need to explicitly call out that the returned subscriber is `Send` and `Sync` to make it possible
/// to pass it it to `init_subscriber` later on.
pub fn get_subscriber(
    name: String,
    env_filter: String
) -> impl Subscriber + Send + Sync {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = BunyanFormattingLayer::new(
        "newsletter_layer".into(),
        std::io::stdout
    );

     Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Registers a subscriber as global default to process span data
/// 
/// It should only be called once!
pub fn init_subscriber(
    subscriber: impl Subscriber + Send + Sync
) {
    LogTracer::init().expect("failed to set logger");
    set_global_default(subscriber).expect("Fauled to set subscriber");
}

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let subscriber = get_subscriber(
        "newsletter-service".into(), 
        "info".into()
    );
    init_subscriber(subscriber);

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