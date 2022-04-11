use std::net::TcpListener;
use newsletter_service::{startup::run, configuration::get_configuration};
use sqlx::{PgConnection, Connection};

#[tokio::test]
async fn health_check_works()  {
    // Arange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
                    .get(format!("{}/health_check", &address))
                    .send()
                    .await
                    .expect("Failed to execute request");
    
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_a_valid_form_data() {
    // Arrange
    let app_address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    // The Connection trait must be in scope for us to invoke the connection, it is not an inherit method of the struct.
    let mut connection = PgConnection::connect(&connection_string)
                                                .await
                                                .expect("Failed to connect to Postgres");
    let client = reqwest::Client::new();

    // Act
    let body = "name=christopher%20wilke&email=christopher.wilke86%40googlemail.com";
    let response = client
                                    .post(&format!("{}/subscriptions", &app_address))
                                    .header("Content-Type", "application/x-www-form-urlencoded")
                                    .body(body)
                                    .send()
                                    .await
                                    .expect("could not send form data");
    
    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
                .fetch_one(&mut connection)
                .await
                .expect("Failed to fetch data");
    
    assert_eq!(saved.email, "christopher.wilke86@googlemail.com");
    assert_eq!(saved.name, "christopher wilke");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
    // Act
        let response = client
                                    .post(&format!("{}/subscriptions", &app_address))
                                    .header("Content-Type", "application/x-www-form-urlencoded")
                                    .body(invalid_body)
                                    .send()
                                    .await
                                    .expect("Failed to execute request.");
    // Assert
    assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
            );
    }
}

// launch the app in the background somehow 
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bind");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("could not bind");
    let _ = tokio::spawn(server);
    
    format!("http://127.0.0.1:{}", port)
}