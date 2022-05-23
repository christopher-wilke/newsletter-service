use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::{PgPool};
use uuid::Uuid;
use unicode_segmentation::UnicodeSegmentation;
use crate::{domain::{NewSubscriber, SubscriberName, SubscriberEmail}, email_client::EmailClient};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

#[tracing::instrument(
    name = " Saving new subscriber details in the database",
    skip(pool, new_subscriber),
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: NewSubscriber
) -> Result<(), sqlx::Error> {

    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
        )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        }
    )?;

    Ok(())
}

pub async fn subscribe(
    form: web::Form<FormData>,
    connection: web::Data<PgPool>,
    email_client: web::Data<EmailClient>
) -> HttpResponse {

    let new_subscriber: NewSubscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish()
    };

    // Send a useless email to the new subscriber. Ignoring delivery errors for now.
    if email_client
        .send_email(
            new_subscriber.email,
            "Welcome",
            "Welcome tou our newsletter!",
            "Welcome tou our newsletter!"
        )
        .await
        .is_err() 
        {
            return HttpResponse::InternalServerError().finish();
        }

    HttpResponse::Ok().finish()

    // match insert_subscriber(&connection, new_subscriber).await {
    //     Ok(_) => HttpResponse::Ok().finish(),
    //     Err(_) => HttpResponse::InternalServerError().finish()
    // }

}

/// Returns `true` if the input satifies all our validation constraints
/// on subscriber names, `false` otherwise
pub fn is_valid_name(s: &str) -> bool {
    let is_empty_or_whitespace = s.trim().is_empty();
    let is_too_long = s.graphemes(true).count() > 256;
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    !(is_empty_or_whitespace || is_too_long || contains_forbidden_characters)
}