mod index;
mod page_not_found;

use actix_web::web::ServiceConfig;
use actix_web::web::to;
use actix_files::Files;

pub(crate) fn config(cfg: &mut ServiceConfig) {
    cfg.default_service(to(page_not_found::page_not_found));
    cfg.service(index::index);
    cfg.service(Files::new("/assets", std::path::Path::new("/var/www/assets")));
    cfg.service(Files::new("/.well-known", std::path::Path::new("/var/www/wkd")));
}
