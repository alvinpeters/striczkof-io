mod listener;
mod tls;
mod utilities;
mod database;
mod config;
mod server;
mod sync;

use std::sync::RwLock;

use log::{error, warn, info, debug};
use tokio::signal;
use tokio::sync::mpsc;
use utilities::{logging, signalling};
#[cfg(feature = "web")]
use crate::server::web::WebServer;
use crate::utilities::signalling::{ Signal, Signallable, TargetServer};
use crate::utilities::error_handling::ExitResult;


const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");
const PROGRAM_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROGRAM_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

struct ContextInner {
    
}

pub(crate) struct Context {
    context_inner: RwLock<ContextInner>,
}

#[tokio::main]
async fn main() -> ExitResult {
    // Print out a sign-of-life line.
    println!("{PROGRAM_NAME} - {PROGRAM_VERSION} : {PROGRAM_AUTHOR}\n");
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

    // Initialise servers here
    #[cfg(feature = "web")]
    let web_server = WebServer::new(signal_send.clone());

    // Now, we wait.
    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                web_server.shutdown();
                break;
            },
            signal = signal_recv.recv() => match signal {
                Some(s) => match s {
                    Signal::Shutdown(t) => {
                        match t {
                            TargetServer::WebServer => {
                                web_server.shutdown();
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
    ExitResult::Ok
}


