use actix_web::{get, post, web, App, HttpResponse, Responder};
use actix_web::dev::ServiceFactory;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "index.stpl")]
struct Index {}

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().body(Index {}.render_once().unwrap())
}

// pub(crate) fn register_pages() -> IntoServiceFactory<S, Request> {
//     App::new()
//         .service(index);
//
