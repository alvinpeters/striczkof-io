use actix_web::{Error, get, HttpResponse, Responder};
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "page_not_found.stpl")]
struct PageNotFound {}

pub(crate) async fn page_not_found() -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(PageNotFound {}.render_once().unwrap())
}

