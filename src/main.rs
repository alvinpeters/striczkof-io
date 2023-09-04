mod config;
mod webpages;

use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use actix_web::{App, HttpServer};

const VERSION: &str = env!("CARGO_PKG_NAME");

use rustls;
use rustls_pemfile::{certs, pkcs8_private_keys};
use actix_web::{get, HttpResponse, Responder};
use rustls::{Certificate, PrivateKey, ServerConfig};
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "index.stpl")]
struct Index {}

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().body(Index {}.render_once().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {


    // TODO: Make TLS optional and only bind HTTP address on failure
    let ssl_config = load_ssl_config();
    println!("Hello, world!{}", VERSION);
    HttpServer::new(|| {
        App::new()
            .service(index)
    })
        // IPv6 bindingss
        .bind_rustls("[::]:443", ssl_config.clone())?
        .bind("[::]:80")?
        // IPv4 bindings
        .bind_rustls("0.0.0.0:443", ssl_config)?
        .bind("0.0.0.0:80")?
        .run()
        .await
}

fn load_ssl_config() -> rustls::server::ServerConfig {
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();
    let cert_file = &mut BufReader::new(
        Cursor::new(include_bytes!("../striczkof.io.crt")));
    let key_file = &mut BufReader::new(
        Cursor::new(include_bytes!("../striczkof.io.key")));
    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}
