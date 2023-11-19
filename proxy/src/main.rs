use actix_web::{get, web, App, HttpServer, Responder};
use chrono::{DateTime, Utc};
use std::env;
use url::form_urlencoded;
use dotenv::dotenv;
use serde::Deserialize;
use anyhow::{anyhow, Result};
use convert_case::{Case, Casing};

#[derive(Debug, Deserialize, Clone)]
struct Event {
    summary: String,
    description: Option<String>,
    location: Option<String>,
    start: EventDateTime,
    end: EventDateTime,
}

#[derive(Debug, Deserialize, Clone)]
struct EventDateTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
    date: Option<String>,
    #[serde(rename = "timeZone")]
    time_zone: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CalendarEvents {
    kind: String,
    items: Vec<Event>,
}

#[get("/locations/{location}/event")]
async fn index(location: web::Path<String>) -> impl Responder {
    println!("Get calendar events for {}", location);
    let gcal_response = get_calendar_events().await;
    match gcal_response {
        Ok(r) => {
            parse_next_events(r, location.to_string().to_case(Case::Title), 1).await.unwrap()
        },
        _ => {
            println!("Failed to get calendar events");
            "Failed to get calendar events".to_string()
        }
    }
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Eat shit, {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Launching fuckoff4-proxy");
    println!("Check dotenv");
    dotenv().ok();
    println!("Run webserver");
    HttpServer::new(|| App::new().service(index).service(hello))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}

fn parse_json(json_str: &str) -> Result<CalendarEvents> {
    serde_json::from_str(json_str).map_err(|e| anyhow!("{}", e))
}

async fn parse_next_events(gcal_payload: String, location: String, num: i32) -> Result<String> {
    match parse_json(gcal_payload.as_str()) {
        Ok(calendar_events) => {
            for e in calendar_events.items {
                match e.location {
                    Some(ref l) => {
                        if l.contains(&location) {
                            return format_gcal_1602(e)
                        }
                    },
                    _ => {
                        
                    },
                }
            }
            return Ok("No upcoming events.".to_string())
        },
        Err(err) => Err(anyhow!("Error parsing JSON: {}", err)),
    } 
}

fn format_gcal_1602(event: Event) -> Result<String> {
    match event.start.date {
        Some(d) => Ok(format!("{}\n{}", event.summary, d)),
        None => Ok(event.summary)
    }
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
