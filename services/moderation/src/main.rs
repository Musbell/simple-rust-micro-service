use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use types::Event;



#[post("/events")]
async fn moderation(req_body: web::Json<Event>) -> impl Responder {

     let (comment_data, event_type) = (req_body.data.clone(), req_body.event_type.clone());

    

    if event_type == "CommentCreated" {
    
      if let types::Data::CommentData(comment) = comment_data {
    let status = if comment.content.contains("orange") {
        "rejected"
    } else {
        "approved"
    };
    println!("Status: {}", status);
}
       
    } else if event_type == "PostCreated" {
        println!("Post Data: {:?}", comment_data);
    } else {
        println!("Invalid event type");
    }

    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    std::env::set_var("RUST_LOG", "actix_web=debug");

    HttpServer::new( move || {
        App::new()
            .service(moderation)
    })
  .bind("127.0.0.1:4002")?
  .run()
  .await
}
