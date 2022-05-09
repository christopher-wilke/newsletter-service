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

pub async fn subscribe(
    form: web::Form<FormData>,
    connection: web::Data<PgPool>
) -> impl Responder {

    let request_id = Uuid::new_v4();

    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscribe_email = %form.email,
        subscriber_name = %form.name
    );

    let _request_span_guard = request_span.enter();

    // We do not call '.enter' on query_span!
    // '.instrument' takes care of it at the right moments in the query future lifetime

    let query_span = tracing::info_span!(
        "Saving new sbuscriber details in the database"
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
        Ok(_) => {
            tracing::info!(
                "request_id {} - New sbuscriber details have been saved",
                request_id
            );
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            tracing::error!("request_id {} - Failed to execute query, {:?}", request_id, e);
            HttpResponse::InternalServerError().finish()
        } 
    }    
}