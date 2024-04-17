use actix_web::{web, HttpResponse, Responder, post, get, HttpServer, App};
use serde_json::Value;
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub comments: Vec<Comment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub content: String,
    pub status: String,
}

#[derive(Debug)]
pub struct PostData {
    pub posts: Arc<Mutex<Vec<Post>>>,
}

impl PostData {
    fn new() -> Self {
        PostData {
            posts: Arc::new(Mutex::new(vec![])),
        }
    }
}

async fn handle_event(event: &web::Json<Value>, posts_data: Arc<Mutex<Vec<Post>>>) -> impl Responder {
    let events = event.as_array().expect("Expected events to be an array");

    for ev in events {
        if let Some(event_type) = ev["event_type"].as_str() {
            match event_type {
                "PostCreated" => {
                    if let Some(post_data) = ev["data"]["PostData"].as_object() {
                        let post = Post {
                            id: Uuid::parse_str(post_data["id"].as_str().unwrap()).unwrap(),
                            title: post_data["title"].as_str().unwrap().to_owned(),
                            comments: Vec::new(),
                        };
                        handle_post_created(&posts_data, &post).await;
                    }
                }
                "CommentCreated" => {
                    if let Some(comment_data) = ev["data"]["CommentData"].as_object() {
                        let comment = Comment {
                            id: Uuid::parse_str(comment_data["id"].as_str().unwrap()).unwrap(),
                            post_id: Uuid::parse_str(comment_data["post_id"].as_str().unwrap()).unwrap(),
                            content: comment_data["content"].as_str().unwrap().to_owned(),
                            status: comment_data["status"].as_str().unwrap().to_owned(),
                        };
                        handle_comment_created(&posts_data, &comment).await;
                    }
                }
                "CommentUpdated" => {
                    if let Some(comment_data) = ev["data"]["CommentData"].as_object() {
                        let comment = Comment {
                            id: Uuid::parse_str(comment_data["id"].as_str().unwrap()).unwrap(),
                            post_id: Uuid::parse_str(comment_data["post_id"].as_str().unwrap()).unwrap(),
                            content: comment_data["content"].as_str().unwrap().to_owned(),
                            status: comment_data["status"].as_str().unwrap().to_owned(),
                        };
                        handle_comment_updated(&posts_data, &comment).await;
                    }
                }
                _ => println!("Unsupported event type: {}", event_type),
            }
        }
    }

    HttpResponse::Ok().body("Events processed")
}

async fn handle_post_created(posts: &Arc<Mutex<Vec<Post>>>, post: &Post) {
    let mut posts_guard = posts.lock().unwrap();
    posts_guard.push(post.clone());
}

async fn handle_comment_created(posts: &Arc<Mutex<Vec<Post>>>, comment: &Comment) {
    let mut posts_guard = posts.lock().unwrap();
    if let Some(post) = posts_guard.iter_mut().find(|p| p.id == comment.post_id) {
        post.comments.push(comment.clone());
    }
}

async fn handle_comment_updated(posts: &Arc<Mutex<Vec<Post>>>, comment: &Comment) {
    let mut posts_guard = posts.lock().unwrap();
    if let Some(post) = posts_guard.iter_mut().find(|p| p.id == comment.post_id) {
        if let Some(com) = post.comments.iter_mut().find(|c| c.id == comment.id) {
            com.content = comment.content.clone();
            com.status = comment.status.clone();
        }
    }
}


#[post("/events")]
async fn events_posts(event: web::Json<Value>, data: web::Data<PostData>) -> impl Responder {
    handle_event(&event, data.get_ref().posts.clone()).await
}

#[get("/posts")]
async fn all_posts(data: web::Data<PostData>) -> impl Responder {
    let posts_guard = data.posts.lock().unwrap().clone();
    HttpResponse::Ok().json(&*posts_guard)
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let posts_data = web::Data::new(PostData::new());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(posts_data.clone())  // Sharing PostData across app
            .service(all_posts)
            .service(events_posts)
    })
        .bind(("127.0.0.1", 4003))?
        .run();

    server.await?;
    Ok(())
}
