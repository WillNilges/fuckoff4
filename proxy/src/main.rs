use actix_web::{get, web, App, HttpServer, Responder};
use chrono::{DateTime, Utc};
use convert_case::{Case, Casing};
use dotenv::dotenv;

pub mod calendar;
use calendar::CalendarEvents;

#[get("/locations/{location}/event")]
async fn screen(location: web::Path<String>) -> impl Responder {
    println!("Get calendar events for {}", location);
    let upcoming_events = CalendarEvents::new().await;
    match upcoming_events {
        Ok(u) => match u.get_next_at_location(location.to_string().to_case(Case::Title)) {
            Some(e) => e.format_1602(),
            None => "No upcoming events.".to_string(),
        },
        Err(e) => format!("Failed to get calendar events: {}", e).to_string(),
    }
}

#[get("/reserve/<location>/")]
async fn reserve(
    location: web::Path<String>,
    start: web::Path<String>,
    end: web::Path<String>,
) -> impl Responder {
    let proposed_start = DateTime::parse_from_rfc3339(&start.to_string())
        .expect("Failed to parse timestamp")
        .with_timezone(&Utc);

    let proposed_end = DateTime::parse_from_rfc3339(&end.to_string())
        .expect("Failed to parse timestamp")
        .with_timezone(&Utc);

    let upcoming_events = CalendarEvents::new().await;
    match upcoming_events {
        Ok(u) => {
            if u.is_free_at_location(location.to_string(), proposed_start, proposed_end) {
                return "Is free!";
            }
            "Reserved at that time."
        }
        _ => "Failed to get calendar events",
    }
}

#[get("/billboard")]
async fn billboard() -> impl Responder {
    "Hello\nworld how are you doing today I am doing just fine\non this\nlovely day this is wonderful isnt it?"
}

//#[post("/billboard")]
//async fn set_billboard() -> impl Responder {
//
//}

#[get("/hello/{name}")]
async fn name(name: web::Path<String>) -> impl Responder {
    format!("FuckOff, {}!", &name)
}

#[get("/")]
async fn test() -> impl Responder {
    "FuckOff!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Launching fuckoff4-proxy");
    println!("Check dotenv");
    dotenv().ok();
    println!("Run webserver");
    HttpServer::new(|| App::new().service(screen).service(name).service(test).service(billboard))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
