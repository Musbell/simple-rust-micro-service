use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use request::on_response;
use serde_json::json;
use std::sync::Mutex;
use types::{Comment, Data, Event, Post, ModerateComment, EventType};
use log::{info, error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug,)]
struct EventData {
    events: Mutex<Vec<Event>>,
}

impl EventData {
    fn new() -> Self {
        EventData {
            events: Mutex::new(vec![]),
        }
    }
}

#[get("/events")]
async fn events(data: web::Data<EventData>) -> impl Responder {
    let events = data.events.lock().unwrap();
    HttpResponse::Ok().json((*events).clone())
}


#[post("/events")]
async fn handle_event(
    data: web::Data<EventData>,
    event: web::Json<serde_json::Value>,
) -> impl Responder {
    let mut event_list = data.events.lock().expect("Mutex poisoned");
    let event: Event = serde_json::from_value(event.into_inner()).expect("Error parsing event");
    event_list.push(event.clone());

    let client = reqwest::Client::new();
    let urls = [
        "http://localhost:4000/events",
        "http://localhost:4001/events",
        "http://localhost:4002/events",
        "http://localhost:4003/events"];
    let event_vector = event_list.clone();




    for &url in &urls {
        let res = client.post(url).json(&event_vector).send().await;
        match res {
            Ok(response) => {
                on_response(Ok(response)).await;
            },
            Err(err) => {
                error!("Error: {}", err);
            }
        }
    }

    HttpResponse::Ok().finish()
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let event_data = web::Data::new(EventData::new());

    HttpServer::new(move || {
        App::new()
            .app_data(event_data.clone())
            .service(events)
            .service(handle_event)
    })
        .bind(("127.0.0.1", 4005))?
        .run()
        .await
}