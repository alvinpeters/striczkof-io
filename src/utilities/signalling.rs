use log::info;

pub(crate) enum TargetServer {
    /// Do something to all servers.
    All,
    /// Do something to the web server.
    #[cfg(feature = "web")]
    WebServer,
}

pub(crate) enum Signal {
    Shutdown(TargetServer),
    UpdateConfig(TargetServer),
}

pub(crate) trait Signallable {
    fn shutdown(&self);
    fn update_config(&self);
}
