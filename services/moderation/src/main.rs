use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use log::{error, info};
use request::on_response;
use serde_json::json;
use types::{Comment, CommentStatus, Event, EventType, ModerateComment};

async fn publish_event_fn(
    url: &str,
    comment: &Comment,
    event_type: EventType,
    status: CommentStatus,
) -> Result<(), String> {
    let comment_json = json!({
        "event_type": event_type,
         "data": {
           "ModerationData": {
               "id": comment.id,
                "post_id": comment.post_id,
                "status": status,
                "content": comment.content
            }
        },
    });


    let client = reqwest::Client::new();

    let res = client.post(url).json(&comment_json).send().await;

    on_response(res).await
}

#[post("/events")]
async fn moderation(req_body: web::Json<Event>) -> impl Responder {
    let (comment_data, event_type) = (req_body.data.clone(), req_body.event_type.clone());

    match event_type {
        EventType::CommentCreated => {
            if let types::Data::CommentData(comment) = comment_data {
                let status = if comment.content.contains("orange") {
                    CommentStatus::Rejected
                } else {
                    CommentStatus::Approved
                };

                info!("Comment status created: {:?}", status);
                if let Err(err) = publish_event_fn("http://localhost:4005/events", &comment, EventType::CommentModerated, status).await {
                    error!("Failed to publish Moderated Comment event: {}", err);
                    return HttpResponse::InternalServerError().json("Error in processing Moderated Comment event");
                }
            }
        }
        _ => {
            error!("Unexpected event type: {:?}", event_type);
            return HttpResponse::BadRequest().json("Unexpected event type received");
        }
    }

    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    std::env::set_var("RUST_LOG", "actix_web=debug");

    HttpServer::new(move || App::new().service(moderation))
        .bind("127.0.0.1:4002")?
        .run()
        .await
}
