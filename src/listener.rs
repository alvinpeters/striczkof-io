use std::net::SocketAddr;
use std::vec::IntoIter;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use hyper::header::HOST;
use hyper::rt::{Read, Write};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use log::debug;
use tokio::io::{Result, Error, ErrorKind};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::task::{JoinHandle, JoinSet};
use tokio_rustls::{TlsAcceptor, TlsStream};
use tokio_util::sync::CancellationToken;

static INDEX: &[u8] = b"The Index service!";
async fn index(request: Request<hyper::body::Incoming>) -> std::result::Result<Response<Full<Bytes>>, hyper::Error> {
    match request.headers().get(HOST).unwrap().to_str().unwrap() {
        "127.0.0.1:8080" => debug!("yay"),
        _ => debug!("nay!")
    }
    Ok(Response::new(Full::new(Bytes::from(INDEX))))
}

pub(crate) struct RunningListener {
    pub(crate) socket: SocketAddr,
    /// True when this is an HTTPS listener.
    pub(crate) over_tls: bool,
    /// Cancellation token for stopping the running listener.
    cancellation_token: CancellationToken,
    join_handle: JoinHandle<(TcpListener, Option<TlsAcceptor>)>
}

impl RunningListener {
    pub(crate) async fn stop(mut self) -> Listener {
        self.cancellation_token.cancel();
        let (tcp_listener, tls_acceptor) = self.join_handle.await.unwrap();
        // Return a stopped listener.
        Listener {
            socket: self.socket,
            tls_acceptor,
            over_tls: false,
            tcp_listener,
        }
    }
}

pub(crate) struct Listener {
    pub(crate) socket: SocketAddr,
    /// Will be None whenever listening or just isn't TLS capable.
    tls_acceptor: Option<TlsAcceptor>,
    /// True when this is an HTTPS listener
    pub(crate) over_tls: bool,
    /// Only None whenever the listener is active
    tcp_listener: TcpListener,
}

impl Listener {
    pub(crate) async fn bind(socket_addr: SocketAddr) -> Result<Listener> {
        Ok(Listener {
            socket: socket_addr,
            tls_acceptor: None,
            over_tls: false,
            tcp_listener: TcpListener::bind(&socket_addr).await?,
        })
    }

    async fn listen_or_cancel(
        tcp_listener: &TcpListener,
        cancellation_token: &CancellationToken
    ) -> Option<(TcpStream, SocketAddr)> {
        // Await on either cancellation or receipt of connection.
        let (tcp_stream, socket_addr) = select! {
            biased;
            _ = cancellation_token.cancelled() => {
                // The token was cancelled
                return None;
            }
            res = tcp_listener.accept() => {
                match res {
                    Ok((s, a)) => (s, a),
                    Err(e) => {
                        eprintln!("Bruh moment!: {}", e); // TODO: l10n: ERR_NOT_LISTENING
                        return None;
                    }
                }
            },
        };
        Some((tcp_stream, socket_addr))
    }

    fn serve_https(tcp_stream: TcpStream, socket_addr: SocketAddr, tls_acceptor: TlsAcceptor) {
        debug!("HTTPS request received from {}!", socket_addr);
        tokio::task::spawn(async move {
            let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => tls_stream,
                Err(err) => {
                    eprintln!("failed to perform tls handshake: {err:#}");
                    return;
                }
            };
            let io = TokioIo::new(tls_stream);
            Self::serve(io).await;
        });
    }

    async fn serve<I>(io: I) where I: Read + Write + Unpin + 'static {
        if let Err(err) = auto::Builder::new(TokioExecutor::new())
            .serve_connection(io, service_fn(index))
            .await
        {
            eprintln!("Error serving connection: {:?}", err);
        }
    }

    fn serve_http(tcp_stream: TcpStream, socket_addr: SocketAddr) {
        debug!("H TTP request received from {}!", socket_addr);
        tokio::task::spawn(async move {
            let io = TokioIo::new(tcp_stream);
            Self::serve(io).await;
        });
    }

    /// Must have a cancellation token provided.
    pub(crate) async fn run(mut self, cancellation_token: CancellationToken) -> Listener {
        let (tcp_listener, tls_acceptor)
            = Self::listen(self.tcp_listener, self.tls_acceptor, cancellation_token).await;
        self.tcp_listener = tcp_listener;
        self.tls_acceptor = tls_acceptor;
        self
    }

    /// Requires a cancellation token, will otherwise make one if not provided
    pub(crate) fn start(
        mut self,
        cancellation_token: Option<CancellationToken>,
    ) -> RunningListener {
        let over_tls = self.tls_acceptor.is_some();
        let cancellation_token = if let Some(t) = cancellation_token {
            t
        } else {
            CancellationToken::new()
        };
        let token = cancellation_token.child_token();
        let join_handle = tokio::task::spawn(async move {
            Self::listen(self.tcp_listener, self.tls_acceptor, token).await
        });

        // Return an active listener.
        RunningListener {
            socket: self.socket,
            over_tls,
            cancellation_token,
            join_handle,
        }
    }

    /// Listen
    async fn listen(
        tcp_listener: TcpListener,
        mut tls_acceptor: Option<TlsAcceptor>,
        cancellation_token: CancellationToken,
    ) -> (TcpListener, Option<TlsAcceptor>) {
        if let Some(a) = tls_acceptor {
            loop {
                let (tcp_stream, socket_addr) = match Self::listen_or_cancel(
                    &tcp_listener, &cancellation_token
                ).await {
                    // Listener accepted
                    Some((s, a)) => (s, a),
                    // Listener cancelled
                    None => break
                };
                Self::serve_https(tcp_stream, socket_addr, a.clone())
            }
            tls_acceptor = Some(a);
        } else {
            loop {
                let (tcp_stream, socket_addr) = match Self::listen_or_cancel(&tcp_listener, &cancellation_token).await {
                    // Listener accepted
                    Some((s, a)) => (s, a),
                    // Listener cancelled
                    None => break
                };
                Self::serve_http(tcp_stream, socket_addr);
            }
        }
        (tcp_listener, tls_acceptor)
    }

    pub(crate) async fn with_tls_acceptor(mut self, tls_acceptor: TlsAcceptor) -> Listener {
        self.tls_acceptor = Option::from(tls_acceptor);
        self.over_tls = true;
        self
    }

    fn bound_socket(&self) -> &SocketAddr {
        &self.socket
    }
}

pub(crate) struct RunningListenerGroup {
    pub(crate) over_tls: bool,
    /// Cancellation token for stopping the running listener.
    cancellation_token: CancellationToken,
    join_set: JoinSet<Listener>,
}

impl RunningListenerGroup {
    async fn stop(mut self) -> ListenerGroup {
        let mut listeners = Vec::new();
        self.cancellation_token.cancel();
        while let Some(l) = self.join_set.join_next().await {
            let listener = l.unwrap(); // TODO: Get rid of this unwrap
            listeners.push(listener);
        }
        ListenerGroup {
            listeners,
            over_tls: self.over_tls,
        }
    }
}

pub(crate) struct ListenerGroup {
    listeners: Vec<Listener>,
    /// True when this is an HTTPS listener
    pub(crate) over_tls: bool,
}

impl ListenerGroup {
    async fn bind_all(socket_addrs: Vec<SocketAddr>) -> ListenerGroup {
        let mut listeners=  Vec::new();
        for socket_addr in socket_addrs {
            let listener = match Listener::bind(socket_addr).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Bruh moment!: {}", e); // TODO: l10n: ERR_NOT_LISTENING
                    continue;
                }
            };
            listeners.push(listener);
        }
        ListenerGroup {
            listeners,
            over_tls: false,
        }
    }

    pub(crate) async fn with_tls_acceptor(mut self, tls_acceptor: TlsAcceptor) -> ListenerGroup {
        for mut listener in &mut self.listeners {
            listener.tls_acceptor = Option::from(tls_acceptor.clone());
        }
        self.over_tls = true;
        self
    }

    pub(crate) fn start(mut self) -> RunningListenerGroup {
        let cancellation_token = CancellationToken::new();
        let mut join_set = JoinSet::new();
        while let Some(l) = self.listeners.pop() {
            join_set.spawn(l.run(cancellation_token.child_token()));
        }
        RunningListenerGroup {
            over_tls: self.over_tls,
            cancellation_token,
            join_set,
        }
    }

    pub(crate) async fn run(mut self, cancellation_token: CancellationToken) -> ListenerGroup {

        self
    }
}

#[cfg(test)]
mod tests {
    const TEST_DURATION: Duration = Duration::from_secs(5);

    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;
    use tokio::join;
    use tokio::time::sleep;
    use tokio_util::sync::CancellationToken;
    use crate::listener::{Listener, ListenerGroup};

    async fn cancel_after_duration(cancellation_token: CancellationToken, duration: Duration) {
        sleep(duration).await;
        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn run_single_listener() {
        let socket
            = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let listener = Listener::bind(socket).await.unwrap();
        let cancellation_token = CancellationToken::new();
        let listener_future = listener.run(
            cancellation_token.clone());
        let cancel_future = cancel_after_duration(
            cancellation_token, TEST_DURATION);
        let res = join!(cancel_future, listener_future);
    }

    #[tokio::test]
    async fn run_multiple_listeners() {
        let sockets = vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
        ];
        let listeners = ListenerGroup::bind_all(sockets).await;
        let cancellation_token = CancellationToken::new();
        let listeners_future = listeners.run(
            cancellation_token.clone());
        let cancel_future = cancel_after_duration(
            cancellation_token, TEST_DURATION);
        let res = join!(cancel_future, listeners_future);
    }
}
