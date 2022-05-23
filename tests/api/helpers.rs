use std::net::TcpListener;
use newsletter_service::{startup::{run, build, get_connection_pool, Application}, configuration::{get_configuration, DatabaseSettings}, telemetry::{get_subscriber, init_subscriber}, email_client::EmailClient};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{PgPool, PgConnection, Connection, Executor, Pool, Postgres};
use uuid::Uuid;

// Ensures that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {

    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    // We cannot assign the output of `get_subscriber` to a variable based on the value
    // of `TEST_LOG` because the sink is part of the typed returned by `get_subscriber`,
    // therefore they are not the same type. We could work around it, but this is the
    // most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            subscriber_name, 
            default_filter_level,
            std::io::stdout
        );
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(
            subscriber_name, 
            default_filter_level,
            std::io::sink
        );
        init_subscriber(subscriber);
    }
    
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool
}

pub async fn spawn_app() -> TestApp {

    Lazy::force(&TRACING);

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    // Create and migrate the db
    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to the the app");

    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address: address,
        db_pool: get_connection_pool(&configuration)
    }
}

async fn configure_database(config: &DatabaseSettings) -> Pool<Postgres> {
    let mut con = PgConnection::connect(&config.connection_string_without_db().expose_secret())
                                                            .await
                                                            .expect("Failled to connect");
    con.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
                .await
                .expect("Could not create Table");
    
    // Migrate DB
    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
                                                            .await
                                                            .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
                .run(&connection_pool)
                .await
                .expect("Failed to migrate DB");

    connection_pool
}