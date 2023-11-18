use actix_web::{get, web, App, HttpServer, Responder};

use chrono::{DateTime, Utc};

use std::env;

use url::form_urlencoded;

use dotenv::dotenv;

#[get("/")]
async fn index() -> impl Responder {
    get_calendar().await.unwrap();
    "Hello, World!"
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    HttpServer::new(|| App::new().service(index).service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

async fn get_calendar() -> anyhow::Result<()> {
    // Get the current UTC time
    let utc: DateTime<Utc> = Utc::now();

    // Format the time as ISO 8601
    let iso_time = utc.to_rfc3339();

    let api_key = env::var("API_KEY")?;

    let calendar_id = env::var("CALENDAR_ID")?;

    let params = [
        ("maxResults", "10"),
        ("orderBy", "startTime"),
        ("showDeleted", "false"),
        ("singleEvents", "true"),
        ("timeMin", &iso_time),
        ("fields", "kind,items(location, start, end, summary, description)"),
        ("key", &api_key),
    ];

    // Encode parameters into a query string
    let encoded_params: String = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(params.iter())
        .finish();

    // Build the complete URL
    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/{}/events?{}",
        calendar_id, encoded_params
    );

    let body = reqwest::get(url).await?.text().await?;
    
    println!("body = {:?}", body);

    Ok(())
}
