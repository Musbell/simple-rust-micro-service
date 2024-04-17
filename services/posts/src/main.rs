use actix_web::{web, App, HttpResponse, HttpServer, Responder};
// use serde::{Deserialize, Serialize};
use log::{error, info};
use request::on_response;
use serde_json::json;
use std::sync::Mutex;
use types::{Event, EventType, Post};
use uuid::Uuid;

struct PostData {
    posts: Mutex<Vec<Post>>,
}

fn id_generator() -> Uuid {
    Uuid::new_v4()
}

async fn publish_event_fn(url: &str, post: &Post) -> Result<(), String>{
    let post_json = json!({
        "event_type": EventType::PostCreated,
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
async fn all_posts(data: web::Data<PostData>) -> impl Responder {
    let posts = data.posts.lock().expect("Mutex poisoned");
    HttpResponse::Ok().json(&*posts)
}

async fn create_post(data: web::Data<PostData>, post: web::Json<serde_json::Value>) -> impl Responder {
    let id = id_generator();
    let mut posts = data.posts.lock().expect("Mutex poisoned");
    let mut new_post = match serde_json::from_value::<Post>(post.into_inner()) {
        Ok(new_post) => new_post,
        Err(e) => {
            error!("Error parsing post JSON: {:?}", e);
            return HttpResponse::BadRequest().json("Malformed JSON data");
        }
    };
    new_post.id = Some(id);
    posts.push(new_post.clone());
    if let Err(err) = publish_event_fn("http://localhost:4005/events", &new_post).await {
        error!("Failed to publish POST event: {}", err);
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().body(format!("Post created: {:?}", new_post))
}

async fn events(req_body: web::Json<serde_json::Value>) -> impl Responder {
    let event : Event = match serde_json::from_value(req_body.into_inner()){
        Ok(body) => body,
        Err(e) => {
            error!("Failed to parse Event JSON: {:?}", e);
            return HttpResponse::BadRequest().json("Malformed JSON data");
        }
    };
    info!("Received Event: {:?}", event.event_type);
    HttpResponse::Ok().body("Received POST Event")
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    info!("starting up");

    let posts = web::Data::new(PostData {
        posts: Mutex::new(vec![]),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(posts.clone())
            .service(web::resource("/posts").route(web::get().to(all_posts)))
            .service(web::resource("/post").route(web::post().to(create_post)))
            .service(web::resource("/events").route(web::post().to(events)))
    })
    .bind("127.0.0.1:4000")?
    .run()
    .await
}
