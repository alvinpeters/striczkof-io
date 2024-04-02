use tokio::sync::RwLock;

struct ContextInner {
    
}

pub(crate) struct Context {
    context_inner: RwLock<ContextInner>,
}

pub(super) struct Daemon {

}