use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Cursor};
use getopts::Options;
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
    /// Creates an empty configuration
    pub(crate) fn new() -> Self {
        Config {
            ipv4_http_socket: None,
            ipv6_http_socket: None,
            tls_config: None,
            ipv4_https_socket: None,
            ipv6_https_socket: None,
        }
    }

    /// Parse comand line arguments
    /// Returns modified configuration if successful, returns None when help or version is requested
    pub(crate) fn parse_args(mut self, args: Vec<String>) -> Option<Self> {
        let cmd = args[0].clone();

        let mut opts = Options::new();
        /// IPv4 HTTP socket (IPv4:port)
        opts.optopt("", "ipv4-http-socket",
                    "IPv4 address and port for an HTTP connection", "ADDR:PORT");
        opts.optopt("", "ipv6-http-socket",
                    "IPv6 address and port for an HTTP connection", "ADDR:PORT");
        opts.optopt("", "ipv4-https-socket",
                    "IPv4 address and port for an HTTPS connection", "ADDR:PORT");
        opts.optopt("", "ipv6-https-socket",
                    "IPv6 address and port for an HTTPS connection", "ADDR:PORT");
        opts.optopt("", "tls-key",
                    "TLS key DER file (must be specified together with --tls-cert)", "FILE");
        opts.optopt("", "tls-cert",
                    "TLS certificate chain PEM file (must be specified together with --tls-key)", "FILE");
        /// Verbosity, can take up to 2 v's
        opts.optflagmulti("v", "verbose", "increase verbosity");
        /// Version
        opts.optflag("V", "version", "print version info and exit");
        /// Help menu
        opts.optflag("h", "help", "print this help menu");
        let matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(f) => panic!("{}", f.to_string())
        };
        if matches.opt_present("h") {
            println!("{}", opts.usage(&format!("Usage: {} [options]", cmd)));
            return None;
        }
        if matches.opt_present("V") {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            return None;
        }
        // Set config
        if matches.opt_present("ipv4-http-socket") {
            self.ipv4_http_socket = matches.opt_str("ipv4-http-socket");
        }
        if matches.opt_present("ipv6-http-socket") {
            self.ipv6_http_socket = matches.opt_str("ipv6-http-socket");
        }
        if matches.opt_present("ipv4-https-socket") {
            self.ipv4_https_socket = matches.opt_str("ipv4-https-socket");
        }
        if matches.opt_present("ipv6-https-socket") {
            self.ipv6_https_socket = matches.opt_str("ipv6-https-socket");
        }
        self.tls_config = if matches.opt_present("tls-key") && matches.opt_present("tls-cert") {
            match get_cert_chain_and_key(
                matches.opt_str("tls-cert").unwrap().as_str(),
                matches.opt_str("tls-key").unwrap().as_str()) {
                Ok((cert_chain, key_der)) => get_tls_config(cert_chain, key_der),
                Err(e) => {
                    eprintln!("Failed to load TLS configuration: {}", e);
                    None
                }
            }
        } else if matches.opt_present("tls-key") ^ matches.opt_present("tls-cert") {
            eprintln!("TLS key or certificate chain is set, but not both. Neither will be loaded.");
            None
        } else {
            None
        };
        Some(self)
    }

    /// Set TLS if needed, set default values.
    pub(crate) fn finalise(mut self) -> Result<Self, &'static str> {
        self.ipv6_https_socket = Some(String::from("[::1]:8443"));
        self.ipv6_http_socket = Some(String::from("[::1]:8080"));
        self.ipv4_http_socket = Some(String::from("127.0.0.1:8080"));

        self.tls_config = match self.tls_config {
            Some(c) => {Some(c)}
            None => if cfg!(feature = "compiled_tls") {
                let out: Option<ServerConfig> = None;
                #[cfg(feature = "compiled_tls")]
                let out = if self.ipv4_https_socket.is_some() || self.ipv6_https_socket.is_some() {
                    println!("HTTPS bindings set, configuring TLS with compiled TLS files.");
                    let (cert_chain, key_der) = match get_compiled_cert_chain_and_key() {
                        Ok((cert_chain, key_der)) => (cert_chain, key_der),
                        Err(e) => return Err(e)
                    };
                    get_tls_config(cert_chain, key_der)
                } else {
                    println!("No HTTPS bindings set, not configuring TLS.");
                    None
                };
                out
            } else {
                None
            },
        };
        return Ok(self);
    }

    pub(crate) fn both_https_sockets_set(&self) -> bool {
        self.ipv4_https_socket.is_some() && self.ipv6_https_socket.is_some()
    }
}

fn get_cert_chain_and_key(cert_path: &str, key_path: &str) -> Result<(Vec<Certificate>, PrivateKey), &'static str> {
    if cert_path.is_empty() || key_path.is_empty() {
        return Err("");
    }
    let cert_buffer = &mut BufReader::new(match File::open(cert_path) {
            Ok(f) => f,
            Err(_) => return Err("Failed to open TLS certificate chain file"),
        });
    let key_buffer = &mut BufReader::new(match File::open(key_path) {
            Ok(f) => f,
            Err(_) => return Err("Failed to open TLS key file"),
        });
    let tls_certs: Vec<Certificate> = match certs(cert_buffer) {
        Ok(certs) => {
            certs.into_iter()
                .map(Certificate)
                .collect()
        },
        Err(_) => {
            return Err("Failed to load provided TLS certificate chain");
        }
    };
    let mut tls_keys: Vec<PrivateKey> = match pkcs8_private_keys(key_buffer) {
        Ok(keys) => {
            keys.into_iter()
                .map(PrivateKey)
                .collect()
        },
        Err(_) => {
            return Err("Failed to load provided TLS keys");
        }
    };
    if tls_keys.is_empty() || tls_certs.is_empty() {
        return Err("Either provided key or certificate chain is empty");
    };
    Ok((tls_certs, tls_keys.remove(0)))
}

/// Gets the TLS key and certificate compiled into the binary.
/// Gives you compile time error if the files are not found.
#[cfg(feature = "compiled_tls")]
fn get_compiled_cert_chain_and_key() -> Result<(Vec<Certificate>, PrivateKey), &'static str> {
    let cert_buffer = &mut BufReader::new(
        Cursor::new(include_bytes!(concat!("../", env!("TLS_CERT")))));
    let key_buffer = &mut BufReader::new(
        Cursor::new(include_bytes!(concat!("../", env!("TLS_KEY")))));

    let tls_certs: Vec<Certificate> = match certs(cert_buffer) {
        Ok(certs) => {
            certs.into_iter()
                .map(Certificate)
                .collect()
        },
        Err(_) => {
            return Err("Failed to load compiled TLS certificates.\n\
                        Perhaps you compiled in the wrong TLS certificate file?");
        }
    };
    let mut tls_keys: Vec<PrivateKey> = match pkcs8_private_keys(key_buffer) {
        Ok(keys) => {
            keys.into_iter()
                .map(PrivateKey)
                .collect()
        },
        Err(_) => {
            return Err("Failed to load compiled TLS keys,\n\
                        Perhaps you compiled in the wrong TLS key file?");
        }
    };
    if tls_keys.is_empty() || tls_certs.is_empty() {
        return Err("Either the compiled keys or the certificate chain is empty.\n\
                    Perhaps you compiled in the wrong TLS files?");
    };
    Ok((tls_certs, tls_keys.remove(0)))
}

fn get_tls_config(cert_chain: Vec<Certificate>, key_der: PrivateKey) -> Option<ServerConfig> {
    let server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();
    match server_config.with_single_cert(cert_chain, key_der) {
        Ok(config) => Some(config),
        Err(_) => {
            eprintln!("Failed to load TLS configuration.");
            None
        }
    }
}
