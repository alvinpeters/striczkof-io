use tokio::sync::mpsc::Sender;
use crate::utilities::signalling::Signal;

pub(crate) struct WebServer {

}

impl WebServer {
    pub(crate) fn new(signal_send: Sender<Signal>) -> WebServer {
        WebServer {

        }
    }
}

impl