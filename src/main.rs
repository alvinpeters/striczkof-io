mod config;

use std::env;
use config::Config;

use actix_web::{App, HttpServer};

use rustls;
use actix_web::{get, HttpResponse, Responder};
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
    let config = match Config::new().parse_args(env::args().collect()) {
        Some(c) => match c.finalise() {
            Ok(c) => c,
            Err(e) => {
                println!("Failed to finalise config: {}", e);
                return Ok(());
            }
        },
        None => return Ok(()) // Just called help or version
    };
    let mut web_server = HttpServer::new(|| {
        App::new()
            .service(index)
    }).server_hostname(env!("_HOSTNAME"));
    match config.ipv6_http_socket {
        Some(ipv6) => {
            match config.ipv4_http_socket {
                Some(ipv4) => {
                    println!("Binding to IPv4 HTTP socket {}", ipv4);
                    web_server = web_server.bind(ipv4)?;
                },
                None => {}
            };
            println!("Binding to IPv6 HTTP socket {}", ipv6);
            web_server = web_server.bind(ipv6)?;
        },
        None => {
            match config.ipv4_http_socket {
                Some(ipv4) => {
                    println!("Binding to IPv4 HTTP socket {}", ipv4);
                    web_server = web_server.bind(ipv4)?;
                },
                None => {
                    println!("No HTTP sockets set");
                }
            };
        }
    };
    match config.tls_config {
        Some(sc) => {
            println!("TLS is enabled. Binding to HTTPS sockets");
            match config.ipv6_https_socket {
                Some(ipv6) => {
                    match config.ipv4_https_socket {
                        Some(ipv4) => {
                            println!("Binding to IPv4 HTTPS socket {}", ipv4);
                            web_server = web_server.bind_rustls(ipv4, sc.clone())?;
                        },
                        None => {
                            println!("No IPv4 HTTPS socket set");
                        }
                    };
                    println!("Binding to IPv6 HTTPS socket {}", ipv6);
                    web_server = web_server.bind_rustls(ipv6, sc)?;
                },
                None => {
                    match config.ipv4_https_socket {
                        Some(ipv4) => {
                            println!("Binding to IPv4 HTTPS socket {}", ipv4);
                            web_server = web_server.bind_rustls(ipv4, sc)?;
                        },
                        None => {
                            println!("No HTTPS sockets set");
                        }
                    };
                }
            };
        },
        None => {
            println!("TLS is disabled");
        }
    }
    if web_server.addrs().is_empty() {
        println!("I don't think you can call this a web server if it doesn't bind to any sockets.");
        Ok(())
    } else {
        println!("Server going live!");
        let result = web_server.run().await;
        println!("Server stopped");
        result
    }
}
