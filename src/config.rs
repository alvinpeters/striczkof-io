use std::fs::File;
use std::io::{BufReader, Cursor};
use getopts::Options;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys, private_key};

pub(crate) struct Config {
    pub(crate) http_sockets: Vec<String>,
    pub(crate) https_sockets: Vec<String>,
    /// TLS Configuration (key and certificate)
    pub(crate) tls_config: Option<ServerConfig>,
}

impl Config {
    /// Creates an empty configuration
    pub(crate) fn new() -> Self {
        Config {
            http_sockets: Vec::new(),
            https_sockets: Vec::new(),
            tls_config: None,
        }
    }

    /// Parse comand line arguments
    /// Returns modified configuration if successful, returns None when help or version is requested
    pub(crate) fn parse_args(mut self, args: Vec<String>) -> Option<Self> {
        let cmd = args[0].clone();

        let mut opts = Options::new();
        // IPv4 HTTP socket (IPv4:port)
        opts.optmulti("", "http-socket",
                    "Address and port for a simple HTTP connection", "ADDR:PORT");
        opts.optmulti("", "https-socket",
                      "Address and port for an HTTP through SSL connection", "ADDR:PORT");
        opts.optopt("", "tls-key",
                    "TLS key DER file (must be specified together with --tls-cert)", "FILE");
        opts.optopt("", "tls-cert",
                    "TLS certificate chain PEM file (must be specified together with --tls-key)", "FILE");
        // Version
        opts.optflag("L", "live", "use live Let's Encrypt directory");
        // Verbosity, can take up to 2 v's
        opts.optflagmulti("v", "verbose", "increase verbosity");
        // Version
        opts.optflag("V", "version", "print version info and exit");
        // Help menu
        opts.optflag("h", "help", "print this help menu");
        let matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(f) => panic!("{}", f.to_string())
        };
        if matches.opt_present("h") {
            println!("{}", opts.usage(&format!("Usage: {} [options]", cmd)));
            #[cfg(feature = "compiled_tls")] {
                println!("Compiled with TLS key '{}' and TLS certificate chain '{}'", env!("TLS_CERT"), env!("TLS_KEY"));
                println!("It will be loaded if you use an HTTPS socket.");
            }
            return None;
        }
        if matches.opt_present("V") {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            return None;
        }
        // Set config
        if matches.opt_present("http-socket") {
            self.http_sockets = matches.opt_strs("http-socket");
        }
        if matches.opt_present("https-socket") {
            self.https_sockets = matches.opt_strs("https-socket");
        }
        self.tls_config = if matches.opt_present("tls-key") && matches.opt_present("tls-cert") {
            if !self.https_sockets.is_empty() {
                match get_cert_chain_and_key(
                    matches.opt_str("tls-cert").unwrap().as_str(),
                    matches.opt_str("tls-key").unwrap().as_str()) {
                    Ok((cert_chain, key_der)) => get_tls_config(cert_chain, key_der),
                    Err(e) => {
                        eprintln!("Failed to load TLS configuration: {}", e);
                        None
                    }
                }
            } else {
                eprintln!("There is no point in setting TLS key and certificate chain if there are no HTTPS sockets set.");
                None
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
        // Set TLS config if key and certs are compiled in
        self.tls_config = match self.tls_config {
            Some(c) => {Some(c)}
            None => {
                #[cfg(feature = "compiled_tls")]
                if !self.https_sockets.is_empty() {
                    println!("HTTPS bindings set, configuring TLS with compiled TLS files.");
                    let (cert_chain, key_der) = match get_compiled_cert_chain_and_key() {
                        Ok((cert_chain, key_der)) => (cert_chain, key_der),
                        Err(e) => return Err(e)
                    };
                    get_tls_config(cert_chain, key_der)
                } else {
                    println!("No HTTPS bindings set, not configuring TLS.");
                    None
                }
                #[cfg(not(feature = "compiled_tls"))]
                None
            }
        };
        return Ok(self);
    }
}

fn get_cert_chain_and_key<'a>(cert_path: &str, key_path: &str) -> Result<(Vec<CertificateDer<'a>>, PrivateKeyDer<'a>), &'static str> {
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
    let tls_certs: Vec<CertificateDer<'a>> = certs(cert_buffer).map(
        |c| {
            c.unwrap()
        }
    ).collect();
    let mut tls_key= match private_key(key_buffer) {
        Ok(key) => {
            match key {
                None => return Err("Failed to load provided TLS keys"),
                Some(key) => key
            }
        },
        Err(_) => {
            return Err("Failed to load provided TLS keys");
        }
    };
    if tls_certs.is_empty() {
        return Err("Either provided key or certificate chain is empty");
    };
    Ok((tls_certs, tls_key))
}

/// Gets the TLS key and certificate compiled into the binary.
/// Gives you compile time error if the files are not found.
#[cfg(feature = "compiled_tls")]
fn get_compiled_cert_chain_and_key<'a>() -> Result<(Vec<CertificateDer<'a>>, PrivateKeyDer<'a>), &'static str> {
    let cert_buffer = &mut BufReader::new(
        Cursor::new(include_bytes!(concat!("../", env!("TLS_CERT")))));
    let key_buffer = &mut BufReader::new(
        Cursor::new(include_bytes!(concat!("../", env!("TLS_KEY")))));

    let tls_certs: Vec<CertificateDer<'a>> = certs(cert_buffer).map(
        |c| {
            c.unwrap()
        }
    ).collect();
    let mut tls_key= match private_key(key_buffer) {
        Ok(key) => {
            match key {
                None => return Err("Failed to load provided TLS keys"),
                Some(key) => key
            }
        },
        Err(_) => {
            return Err("Failed to load provided TLS keys");
        }
    };
    if tls_certs.is_empty() {
        return Err("Either the compiled keys or the certificate chain is empty.\n\
                    Perhaps you compiled in the wrong TLS files?");
    };
    Ok((tls_certs, tls_key))
}

fn get_tls_config(cert_chain: Vec<CertificateDer<'static>>, key_der: PrivateKeyDer<'static>) -> Option<ServerConfig> {
    match ServerConfig::builder()
        .with_no_client_auth().with_single_cert(cert_chain, key_der) {
        Ok(config) => Some(config),
        Err(_) => {
            eprintln!("Failed to load TLS configuration.");
            None
        }
    }
}
