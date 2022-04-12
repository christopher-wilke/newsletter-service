use actix_web::{HttpServer, App, web, dev::Server};
use sqlx::{PgPool};
use std::net::TcpListener;

use crate::routes::*;

pub fn run(
    listener: TcpListener,
    connection: PgPool
) -> Result<Server, std::io::Error> {

    let con = web::Data::new(connection);

    let server = HttpServer::new(move|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(con.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}