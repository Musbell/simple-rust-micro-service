use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use request::on_response;
use serde_json::json;
use std::sync::Mutex;
use types::{Event, Post, Comment, Data};
use uuid::Uuid;

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

async fn publish_post_event_fn(url: &str, post: &Post) -> Result<(), String> {
    let post_json = json!({
        "event_type": "PostCreated",
        "data": {
           "PostData": {
                 "id": post.id,
                "title": post.title,
            }
        },
    });

    let client = reqwest::Client::new();

    let res = client.post(url).json(&post_json).send().await;

    on_response(res).await
}
async fn publish_comment_event_fn(url: &str, comment: &Comment) -> Result<(), String> {
    let comment_json = json!({
        "event_type": "CommentCreated",
        "data": {
            "CommentData": {
               "id": comment.id,
                "post_id": comment.post_id,
                "status": comment.status,
                "content": comment.content
            }
        }
    });

    let client = reqwest::Client::new();

    let res = client.post(url).json(&comment_json).send().await;

    on_response(res).await
}

#[post("/events")]
async fn handle_event(
    data: web::Data<EventData>,
    event: web::Json<serde_json::Value>,
) -> impl Responder {
    let mut event_list = data.events.lock().expect("Mutex poisoned");
    let event: Event = serde_json::from_value(event.into_inner()).expect("Error parsing event");
    event_list.push(event.clone());

    match event.data {
        Data::PostData(post) => {
            publish_post_event_fn("http://localhost:4000/events", &post)
                .await
                .expect("Error publishing POST event");
        }
        Data::CommentData(comment) => {
            publish_comment_event_fn("http://localhost:4001/events", &comment)
                .await
                .expect("Error publishing COMMENT event");
        }
    }

    HttpResponse::Ok().body(format!("Event received: {:?}", event_list))
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
