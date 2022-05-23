use newsletter_service::{configuration::{get_configuration, DatabaseSettings}, telemetry::{get_subscriber, init_subscriber}, startup::{run, Application}, email_client::EmailClient};
use secrecy::ExposeSecret;
use sqlx::{PgPool, PgConnection, Connection, Pool, Postgres, Executor};
use uuid::Uuid;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let subscriber = get_subscriber(
        "newsletter-service".into(), 
        "info".into(),
        std::io::stdout
    );
    init_subscriber(subscriber);

    let mut configuration = get_configuration().expect("Failed to read the configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;

    Ok(())
}

pub async fn configure_database(config: &DatabaseSettings) -> Pool<Postgres> {
    let mut con = PgConnection::connect(&config.connection_string_without_db().expose_secret())
                                                            .await
                                                            .expect("Failed to connect");
    con.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
                .await
                .expect("Could not create Table");
    
    // Migrate DB
    let connection_pool = PgPool::connect_lazy(&config.connection_string().expose_secret())
                                                            .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
                .run(&connection_pool)
                .await
                .expect("Failed to migrate DB");

    connection_pool
}