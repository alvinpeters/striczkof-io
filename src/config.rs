use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Cursor};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

pub(crate) struct Config {
    /// If none of the four sockets are set, don't bind to anything I guess

    /// IPv4 address and port to bind HTTPS to
    pub(crate) ipv4_http_socket: Option<String>,
    /// IPv6 address and port to bind HTTP to
    pub(crate) ipv6_http_socket: Option<String>,
    /// If either sockets is set, TLS will be enabled

    /// TLS Configuration (key and certificate)
    pub(crate) tls_config: Option<ServerConfig>,
    /// IPv4 address and port to bind HTTPS to
    pub(crate) ipv4_https_socket: Option<String>,
    /// IPv6 address and port to bind HTTPS to
    pub(crate) ipv6_https_socket: Option<String>,
}

impl Config {
    /// Load the configuration, using default values for now
    pub(crate) fn new() -> Result<Config, Box<dyn Error>> {
        let tls_key = "";
        let tls_cert = "";
        let ipv4_https_socket = Some(String::from("127.0.0.1:8443"));
        let ipv6_https_socket = Some(String::from("[::1]:8443"));
        /// Return the configuration
        Ok(Config {
            ipv4_http_socket: Some(String::from("127.0.0.1:8080")),
            ipv6_http_socket: Some(String::from("[::1]:8080")),
            tls_config: if ipv4_https_socket.is_some() || ipv6_https_socket.is_some() {
                println!("HTTPS bindings set, configuring TLS");
                get_tls_config(tls_key, tls_cert)
            } else {
                println!("HTTPS bindings not set, not configuring TLS");
                None
            },
            ipv4_https_socket,
            ipv6_https_socket,
        })
    }

    pub(crate) fn both_https_sockets_set(&self) -> bool {
        self.ipv4_https_socket.is_some() && self.ipv6_https_socket.is_some()
    }
}

fn get_tls_config(key_file: &str, cert_file: &str) -> Option<ServerConfig> {
    #[cfg(not(feature = "compiled_tls"))]
    let key_buffer = &mut BufReader::new(if key_file.is_empty() {
        println!("TLS key is not specified, not using TLS");
        return None;
    } else {
        match File::open(key_file) {
            Ok(f) => f,
            Err(_) => {
                eprintln!("Failed to open TLS key file");
                return None;
            }
        }
    });
    #[cfg(not(feature = "compiled_tls"))]
    let cert_buffer = &mut BufReader::new(if cert_file.is_empty() {
        println!("TLS certificate is not specified, not using TLS");
        return None;
    } else {
        match File::open(cert_file) {
            Ok(f) => f,
            Err(_) => {
                eprintln!("Failed to open TLS certificate file");
                return None;
            }
        }
    });
    #[cfg(feature = "compiled_tls")]
    let key_buffer = &mut BufReader::new(
        Cursor::new(include_bytes!(concat!("../", env!("TLS_KEY")))));
    #[cfg(feature = "compiled_tls")]
    let cert_buffer = &mut BufReader::new(
        Cursor::new(include_bytes!(concat!("../", env!("TLS_CERT")))));
    let mut tls_keys: Vec<PrivateKey> = pkcs8_private_keys(key_buffer)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();
    let tls_certs: Vec<Certificate> = certs(cert_buffer)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    if tls_keys.is_empty() ^ tls_certs.is_empty() {
        eprintln!("Either TLS keys or certificates are missing, not using TLS.");
        return None;
    }
    let server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();
    match server_config.with_single_cert(tls_certs, tls_keys.remove(0)) {
        Ok(config) => Some(config),
        Err(_) => {
            eprintln!("Failed to load TLS configuration.");
            None
        }
    }
}
