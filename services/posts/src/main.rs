use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
// use serde::{Deserialize, Serialize};
use request::on_response;
use serde_json::json;
use std::sync::Mutex;
use types::{Event, Post};
use uuid::Uuid;

struct PostData {
    posts: Mutex<Vec<Post>>,
}

fn id_generator() -> Uuid {
    Uuid::new_v4()
}

async fn publish_event_fn(url: &str, post: &Post) -> Result<(), String> {
    let post_json = json!({
        "event_type": "PostCreated",
        "data": {
            "id": post.id,
            "title": post.title,
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

async fn create_post(data: web::Data<PostData>, post: web::Json<Post>) -> impl Responder {
    let id = id_generator();
    let mut posts = data.posts.lock().expect("Mutex poisoned");
    let mut new_post = post.into_inner();
    new_post.id = Some(id);
    posts.push(new_post.clone());
    if let Err(err) = publish_event_fn("http://localhost:4005/events", &new_post).await {
        log::error!("Failed to publish event: {}", err);
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().body(format!("Post created: {:?}", new_post))
}

async fn events(req_body: web::Json<Event>) -> impl Responder {
    println!("Received Event: {:?}", req_body.event_type);
    HttpResponse::Ok().body("Received Event")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

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
