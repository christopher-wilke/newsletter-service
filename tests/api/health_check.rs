use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works()  {
    // Arange
    let app_instance = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
                    .get(format!("{}/health_check", &app_instance.address))
                    .send()
                    .await
                    .expect("Failed to execute request");
    
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}