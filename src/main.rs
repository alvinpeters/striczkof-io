mod config;
mod webpages;

use std::env;
use config::Config;

use actix_web::{App, HttpServer};

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
            .configure(webpages::config)
    }).server_hostname(env!("_HOSTNAME"));
    if !config.http_sockets.is_empty() {
        for http_socket in config.http_sockets.iter() {
            web_server = web_server.bind(http_socket)?;
            println!("Bound to HTTP socket {}", http_socket);
        }
    }
    if !config.https_sockets.is_empty() {
        match config.tls_config {
            Some(c) => {
                println!("TLS is enabled. Binding to HTTPS sockets");
                let sockets_left = config.https_sockets.len();
                for https_socket in config.https_sockets.iter() {
                    web_server = web_server.bind_rustls_0_22(https_socket, c.clone())?;
                    println!("Bound to HTTPS socket {}", https_socket);
                }
            },
            None => {
                eprintln!("HTTPS sockets set but TLS is disabled. Not binding to HTTPS sockets.");
            }
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

