use actix_web::{get, HttpResponse, Responder};
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "index.stpl")]
struct Index {}

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().body(Index {}.render_once().unwrap())
}
