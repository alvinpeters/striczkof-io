use std::env;

fn main() {
    let tls_key = option_env!("TLS_KEY");
    let tls_cert = option_env!("TLS_CERT");
    println!(concat!("cargo:warning=", env!("TLS_KEY")));
    if tls_key.is_some() && tls_cert.is_some() {
        // If both environment variables are set, enable a feature.
        println!("cargo:rustc-cfg=feature=\"compiled_tls\"");
    } else if tls_key.is_some() ^ tls_cert.is_some() {
        println!("cargo:warning=TLS_KEY or TLS_CERT is set, but not both. neither will be compiled in.");
    }

}