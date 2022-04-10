use std::net::TcpListener;

use newsletter_service::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bind");
    let port = listener.local_addr().unwrap().port();
    let server = newsletter_service::run(listener).expect("could not bind");
    let _ = tokio::spawn(server);
    println!("{}", format!("http://127.0.0.1:{}", port));

    Ok(())
    
}