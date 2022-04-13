use std::net::TcpListener;

mod routes;
mod startup;

use newsletter_service::configuration::get_configuration;
use sqlx::{PgPool};
use startup::run;
use uuid::Uuid;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let mut configuration = get_configuration().expect("Failed to read the configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let con = PgPool::connect(&configuration.database.connection_string())
                                                    .await
                                                    .expect("could not create Pool");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Could not bind");
    run(listener, con)?.await
}