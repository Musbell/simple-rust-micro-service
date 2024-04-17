use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::{error, info, log};
use request::on_response;
use serde_json::json;
use std::sync::Mutex;
use types::{Comment, CommentStatus, Event, EventType};
use uuid::Uuid;
use rayon::prelude::*;

struct CommentData {
    comments: Mutex<Vec<Comment>>,
}

fn id_generator() -> Uuid {
    Uuid::new_v4()
}

async fn publish_event_fn(
    url: &str,
    comment: &Comment,
    event_type: EventType,
) -> Result<(), String> {
    let comment_json = json!({
        "event_type": event_type,
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

#[get("/posts/{post_id}/comments")]
async fn get_comments(path: web::Path<(Uuid,)>, data: web::Data<CommentData>) -> impl Responder {
    let post_id = path.0;
    let comments = &data.comments.lock().unwrap();
    let comments_for_post = comments
        .par_iter()
        .filter(|c| c.post_id == Some(post_id))
        .collect::<Vec<&Comment>>();
    HttpResponse::Ok().json(comments_for_post)
}

#[post("/posts/{post_id}/comments")]
async fn create_comment(
    path: web::Path<(Uuid,)>,
    data: web::Data<CommentData>,
    comment: web::Json<Comment>,
) -> impl Responder {
    let post_id = path.0;
    let id = id_generator();
    let mut comments = data.comments.lock().unwrap();
    let mut new_comment = comment.into_inner();
    new_comment.id = Some(id);
    new_comment.post_id = Some(post_id);
    new_comment.status = Some(CommentStatus::Pending);
    comments.push(new_comment.clone());
    if let Err(err) = publish_event_fn(
        "http://localhost:4005/events",
        &new_comment,
        EventType::CommentCreated,
    )
    .await
    {
        error!("Failed to publish COMMENT event: {}", err);
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().json(new_comment)
}

#[post("/events")]
async fn events(req_body: web::Json<Event>, data: web::Data<CommentData>) -> impl Responder {
    let (comment_data, event_type) = (req_body.data.clone(), req_body.event_type.clone());

    println!("{:?}", comment_data);
    println!("{:?}", event_type);

    HttpResponse::Ok().body("Received COMMENT Event")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    std::env::set_var("RUST_LOG", "actix_web=debug");

    let comments = web::Data::new(CommentData {
        comments: Mutex::new(vec![]),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(comments.clone())
            .service(get_comments)
            .service(create_comment)
            .service(events)
    })
    .bind(("127.0.0.1", 4001))?
    .run()
    .await
}
