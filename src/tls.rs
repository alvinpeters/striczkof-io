use std::{fs, io};
use std::io::{Result, ErrorKind};
use std::sync::Arc;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

pub(crate) fn create_acceptor(
    private_key: PrivateKeyDer<'static>,
    public_certificates: Vec<CertificateDer<'static>>
) -> Result<TlsAcceptor> {
    // TODO: Temporary, make proper TLS handler
    let mut server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(public_certificates, private_key)
        .unwrap();
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];
    // Return this
    Ok(TlsAcceptor::from(Arc::new(server_config)))
}

fn load_pem_reader(file_path: &str) -> io::BufReader<fs::File> {
    // TODO: Proper logger/error handler
    let pem_file = match fs::File::open(file_path) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => todo!("get_text!(ERR_TLS_FILE_NOT_FOUND, file_path)"),
            ErrorKind::PermissionDenied => todo!("get_text!(ERR_TLS_FILE_NOT, file_path)"),
            ErrorKind::InvalidInput => todo!("get_text!(ERR_TLS_FILE_NOT, file_path)"),
            _ => todo!("get_text!(ERR, file_path)")
        }
    };
    // Return BufReader
    io::BufReader::new(pem_file)
}

pub(crate) fn private_key_from_file(file_path: &str) -> PrivateKeyDer<'static> {
    let mut reader = load_pem_reader(file_path);

    // Return private key
    // TODO: Proper logger/handler
    rustls_pemfile::private_key(&mut reader).map(|key| key.unwrap()).unwrap()
}

pub(crate) fn public_certificates_from_file(file_path: &str) -> Vec<CertificateDer<'static>> {
    let mut reader = load_pem_reader(file_path);
    // Return private key
    // TODO: Proper logger/handler
    rustls_pemfile::certs(&mut reader).map(|cert| cert.unwrap()).collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use rcgen::generate_simple_self_signed;

    ///
    const PRIV_KEY_FILE: &str = "testing.priv.key";
    const PUB_CERT_FILE: &str = "testing.pub.cert";

    fn generate_cert() {
        // Do nothing if both private key and public certificate exist.
        if Path::new(PRIV_KEY_FILE).exists() && Path::new(PUB_CERT_FILE).exists() {
            return;
        }
        let domain_hosts = vec![
            "localhost".to_string()
        ];
        let cert = generate_simple_self_signed(domain_hosts).unwrap();
        // Write the private key and public certificate to their respective files.
        fs::write(PRIV_KEY_FILE, cert.serialize_private_key_pem()).expect(
            format!("Cannot write to {}!", PRIV_KEY_FILE).as_str()
        );
        fs::write(PUB_CERT_FILE, cert.serialize_pem().unwrap()).expect(
            format!("Cannot write to {}!", PUB_CERT_FILE).as_str()
        );
    }

    #[test]
    fn create_acceptor_from_files() {
        generate_cert()
    }
}