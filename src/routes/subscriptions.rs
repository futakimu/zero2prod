use chrono::Utc;
use uuid::Uuid;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use crate::domain::{NewSubscriber, SubscriberName, SubscriberEmail};
use sqlx::postgres::PgQueryResult;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
name = "Adding a new subscriber",
skip(form, pool),
fields(
subscriber_email = % form.email,
subscriber_name = % form.name
)
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let name = match SubscriberName::parse(form.0.name) {
        Ok(name) => name,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let email = match SubscriberEmail::parse(form.0.email) {
        Ok(email) => email,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let new_subscriber = NewSubscriber { email, name };
    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
name = "Saving new subscriber details in the database",
skip(pool, new_subscriber)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<PgQueryResult, sqlx::Error> {
    let result = sqlx::query(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
        "#
    )
        .bind(Uuid::new_v4())
        .bind(new_subscriber.email.as_ref())
        .bind(new_subscriber.name.as_ref())
        .bind(Utc::now())
        .execute(pool)
        .await?;

    Ok(result)
}
