use actix_web::{get, web, App, HttpServer, Responder};

use chrono::{DateTime, Utc};

use std::{env, arch::x86_64::__cpuid, task::Wake};

use url::form_urlencoded;

use dotenv::dotenv;

use serde::Deserialize;
use serde_json::Result;

#[derive(Deserialize)]
struct CalendarEvent {
    summary: String,
    description: String,
    location: String,
    start: chrono::NaiveDateTime,
    end: chrono::NaiveDateTime,
}

#[get("/")]
async fn index() -> impl Responder {
    let gcal_response = get_calendar_events().await;
    match gcal_response {
        Ok(r) => {
            let next_event = parse_next_events(r, 1).await.unwrap();
            "Hello World!"
        },
        _ => {
            "Failed to get calendar events"
        }
    }
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

async fn parse_next_events(gcal_payload: String, num: i32) -> anyhow::Result<String> {
    println!("Hello!?");
    println!("{}", gcal_payload);
    let j = json::parse(&gcal_payload)?;
    //println!("{}", );

    match j["items"][0].as_str() {
        Some(s) => {
            let e: CalendarEvent = serde_json::from_str(s)?;
            println!("Event: {}", e.summary);
        },
        None => {
        }
    }
    //println!("Event: {}", j["items"][0]);
    Ok("chom".to_string())
}

async fn get_calendar_events() -> anyhow::Result<String> {
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
    
    Ok(body)
}
