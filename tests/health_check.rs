use actix_web::{Responder, HttpResponse};
use newsletter_service::*;

#[tokio::test]
async fn health_check_works()  {
    // Arange
    spawn_app().await.expect("Failed to spawn our App");

    // Need to bring 'reqwest' to perform HTTP requests against the app
    let client = reqwest::Client::new();

    // Act
    let response = client
                    .get("http://127.0.0.1:8000/health_check")
                    .send()
                    .await
                    .expect("Failed to execute request");
    
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// launch the app in the background somehow 
async fn spawn_app() -> std::io::Result<()> {
    todo!()
}