use actix_web::{get, web, App, HttpServer, Responder};
use dotenv::dotenv;
use convert_case::{Case, Casing};

pub mod calendar;
use calendar::CalendarEvents;

#[get("/locations/{location}/event")]
async fn screen(location: web::Path<String>) -> impl Responder {
    println!("Get calendar events for {}", location);
    let upcoming_events = CalendarEvents::new().await;
    match upcoming_events {
        Ok(u) => {
            match u.get_next_at_location(location.to_string().to_case(Case::Title)) {
                Some(e) => {
                    return e.format_1602()
                },
                None => {
                    return "No upcoming events.".to_string()
                }
            }
        },
        _ => {
            return "Failed to get calendar events".to_string()
        },
    }
}

#[get("/{name}")]
async fn name(name: web::Path<String>) -> impl Responder {
    format!("FuckOff, {}!", &name)
}

#[get("/")]
async fn test() -> impl Responder {
    format!("FuckOff!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Launching fuckoff4-proxy");
    println!("Check dotenv");
    dotenv().ok();
    println!("Run webserver");
    HttpServer::new(
        || App::new()
            .service(screen)
            .service(name)
            .service(test)
    )
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
