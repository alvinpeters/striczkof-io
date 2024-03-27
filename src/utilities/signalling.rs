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

pub(crate) struct AllSignallables<'a> {
    signallables: Vec<&'a dyn Signallable>,
}

impl<'a> AllSignallables<'a> {
    pub(crate) fn new() -> AllSignallables<'a> {
        AllSignallables {
            signallables: Vec::new(),
        }
    }

    pub(crate) fn add_signallable(&'a mut self, signallable: &'a dyn Signallable) {
        &self.signallables.push(signallable);
    }
}

impl Signallable for AllSignallables<'_> {
    fn shutdown(&self) {
        info!("Shutting down all servers");
        for signallable in self.signallables.iter() {
            signallable.shutdown();
        }
    }

    fn update_config(&self) {
        todo!()
    }
}