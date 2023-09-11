use actix_web::{get, HttpResponse, Responder};
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "about.stpl")]
struct About {}

#[get("/about")]
pub async fn about() -> impl Responder {
    HttpResponse::Ok().body(About {}.render_once().unwrap())
}
