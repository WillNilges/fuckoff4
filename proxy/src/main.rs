use actix_web::{get, web, App, HttpServer, Responder};
use chrono::{DateTime, Timelike, Utc};
use convert_case::{Case, Casing};
use dotenv::dotenv;

use std::env;

use async_mutex::Mutex;

pub mod calendar;
use calendar::CalendarEvents;

struct EventCache {
    events: Mutex<CalendarEvents>,
    last_update: Mutex<DateTime<Utc>>,
}

async fn screen(cache: web::Data<EventCache>, location: web::Path<String>) -> String {
    println!("Get calendar events for {}", location);
    let mut last_update = cache.last_update.lock().await;
    let mut events = cache.events.lock().await;

    // Check if we need to update.
    let ttl: i64 = match env::var("CACHE_TTL") {
        Ok(t) => t.parse::<i64>().unwrap(),
        Err(_) => 30,
    };

    if Utc::now() > *last_update + chrono::Duration::seconds(ttl) {
        print!("Refreshing cache...");
        match (*events).update().await {
            Ok(_) => {
                *last_update = Utc::now();
                println!(" done");
            }
            Err(e) => {
                let msg = format!("Failed to get calendar events: {}", e).to_string();
                println!("{}", msg);
                return msg;
            }
        };
    }

    let event_text = match (*events).get_next_at_location(&location.to_case(Case::Title)) {
        Some(e) => e.format_1602(),
        None => "No upcoming events.".to_string(),
    };

    let now = chrono::offset::Local::now();
    let mut time_text = format!("Time: {}:{}", now.hour(), now.minute());
    time_text = format!("{: >width$}", time_text, width = 20);

    let screen = format!("{}\n\n{}", event_text, time_text);
    return screen
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
            if u.is_free_at_location(&location, proposed_start, proposed_end) {
                return "Is free!";
            }
            "Reserved at that time."
        }
        _ => "Failed to get calendar events",
    }
}

async fn oh_hi() -> impl Responder {
    "Oh, hi."
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Launching sidegrade4-proxy");
    println!("Check dotenv");
    dotenv().ok();
    println!("Run webserver");

    let cache = web::Data::new(EventCache {
        events: Mutex::new(CalendarEvents::new().await.unwrap()),
        last_update: Mutex::new(Utc::now()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(cache.clone())
            .route("/locations/{location}/event", web::get().to(screen))
            .route("/", web::get().to(oh_hi))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
