use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder};
use std::sync::Mutex;
use uuid::Uuid;
use types::Comment;



struct CommentData {
    comments: Mutex<Vec<Comment>>,
}

fn id_generator() -> Uuid {
    Uuid::new_v4()
}

#[get("/posts/{post_id}/comments")]
async fn get_comments(path: web::Path<(Uuid,)>, data: web::Data<CommentData>) -> impl Responder {
    let post_id = path.0;
    let comments = &data.comments.lock().unwrap();
    let comments_for_post = comments.iter().filter(|c| c.post_id == post_id).collect::<Vec<&Comment>>();
    HttpResponse::Ok().json(comments_for_post)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let comments = web::Data::new(CommentData {
        comments: Mutex::new(vec![]),
    });


    HttpServer::new( move || {
        App::new()
            .app_data(comments.clone())
            .service(get_comments)
    })
    .bind(("127.0.0.1", 4001))?
    .run()
    .await
}