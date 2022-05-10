use actix_web::{HttpResponse, web, Responder};
use chrono::Utc;
use sqlx::{PgPool};
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection)
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    connection: web::Data<PgPool>
) -> impl Responder {

    let query_span = tracing::info_span!(
        "Saving new subscriber details in the database"
    );

    // 'Result' has two variants: 'Ok' and 'Err'.
    // The first for successes, the second for failures.
    // We use a 'match' statement to choose what to to based on the outcome.
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .instrument(query_span)
    .await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to execute query, {:?}", e);
            HttpResponse::InternalServerError().finish()
        } 
    }    
}