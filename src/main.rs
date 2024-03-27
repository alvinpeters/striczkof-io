mod listener;
mod tls;
mod utilities;
mod database;
mod config;
mod server;

use log::{error, warn, info, debug};
use tokio::signal;
use tokio::sync::mpsc;
use utilities::{logging, signalling};
#[cfg(feature = "web")]
use crate::server::web::WebServer;
use crate::utilities::signalling::{AllSignallables, Signal, Signallable, TargetServer};


const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");
const PROGRAM_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROGRAM_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Print out a sign-of-life line.
    println!("{PROGRAM_NAME} - {PROGRAM_VERSION} : {PROGRAM_AUTHOR}");
    // l10n gets initialised here when implemented.
    // Obviously, hard to localise if the localisation component just fails so just say:
    // localisation::init(lang).expect("Failed to initialise localisation for language: {lang}!");
    // l10n: ERR_LOGGER_INIT_FAIL
    logging::init(log::LevelFilter::Debug).expect("Failed to start the logger!");
    // l10n: INF_MAIN_START
    info!("Server initialising.");
    // The main thread should just act as a daemon coordinating all other threads.
    // A mspc channel would be used by the servers to communicate to this thread and have them do
    // stuff, i.e.: pausing, restart, or even shutdown.
    // The main thread will then sort them out using cancellation tokens, etc.
    // Realistically, the number of signals at a time should not exceed that amount
    let (signal_send, mut signal_recv) = mpsc::channel(10);

    let mut all_signallables = AllSignallables::new();
    // Initialise servers here
    #[cfg(feature = "web")]
    let web_server = WebServer::new(signal_send.clone());
    #[cfg(feature = "web")]
    all_signallables.add_signallable(&web_server);

    // Now, we wait.
    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                all_signallables.shutdown();
                break;
            },
            signal = signal_recv.recv() => match signal {
                Some(s) => match s {
                    Signal::Shutdown(t) => {
                        match t {
                            TargetServer::All => {
                                all_signallables.shutdown();
                                break;
                            },
                            _ => {}
                        }
                    },
                    _ => {},

                },
                None => {}
            },
        }
    }
    // The end.
    Ok(())
}


