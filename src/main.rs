use std::net::TcpListener;

mod routes;
mod startup;

use newsletter_service::configuration::get_configuration;
use startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // panic if we cannot read the configuration
    let configuration = get_configuration().expect("Failed to read the configuration");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Could not bind");
    run(listener)?.await
}