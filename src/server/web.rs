use tokio::sync::mpsc::Sender;
use super::Server;
use crate::{listener::ListenerGroup, utilities::signalling::{Signal, Signallable}};
use crate::listener::RunningListenerGroup;

pub(crate) struct WebServer {
    http_listeners: Option<ListenerGroup>,
    running_http_listeners: Option<ListenerGroup>,
    #[cfg(feature = "https")]
    https_listeners: Option<ListenerGroup>,
    #[cfg(feature = "https")]
    running_https_listeners: Option<ListenerGroup>,

}

impl WebServer {
    pub(crate) fn new(signal_send: Sender<Signal>) -> WebServer {
        WebServer {
            http_listeners: None,
            running_http_listeners: None,
            #[cfg(feature = "https")]
            https_listeners: None,
            #[cfg(feature = "https")]
            running_https_listeners: None,
        }
    }
}

impl Server for WebServer {
    fn start(&self) {
        
    }

    fn pause(&self) {
        todo!()
    }

    fn stop(self) {
        todo!()
    }
}

impl Signallable for WebServer {
    fn shutdown(&self) {
        
    }

    fn update_config(&self) {
        todo!()
    }
}