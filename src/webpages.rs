mod index;
mod about;

use actix_web::web;

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(index::index);
    cfg.service(about::about);
}
