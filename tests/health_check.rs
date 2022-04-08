use actix_web::{Responder, HttpResponse};
use newsletter_service::*;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}