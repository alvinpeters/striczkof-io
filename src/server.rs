#[cfg(feature = "web")]
pub(crate) mod web;

pub(super) trait Server {
    fn start(&self);

    /// Pauses without returning anything.
    fn pause(&self);

    /// Stops then consumes itself.
    fn stop(self);
}