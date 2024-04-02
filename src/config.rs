use std::net::SocketAddr;
use tokio_rustls::TlsAcceptor;

mod ini_file;
mod cli_opts;

#[cfg(feature = "https_web")]
struct HttpsWebConfig {
    https_sockets: Vec<SocketAddr>,
    tls_acceptor: TlsAcceptor,
}

#[cfg(feature = "web")]
struct WebConfig {
    http_sockets: Vec<SocketAddr>,
    #[cfg(feature = "https_web")]
    https_config: HttpsWebConfig,
}

pub(crate) struct Config {
    #[cfg(feature = "web")]
    web_config: WebConfig,
}

impl Config {
    pub(crate) fn new() {
        
    }
}
