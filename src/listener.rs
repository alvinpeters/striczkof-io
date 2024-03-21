use std::net::SocketAddr;
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

pub(crate) struct Listener {
    pub(crate) socket: SocketAddr,
    /// Will be None whenever listening or just isn't TLS capable.
    tls_acceptor: Option<TlsAcceptor>,
    /// True when this is an HTTPS listener
    pub(crate) over_tls: bool,
    /// Only None whenever the listener is active
    tcp_listener: Option<TcpListener>,
    /// Only None whenever the listener is active or token is not provided
    /// A cancellation token will be made if not present once listener starts
    cancellation_token: Option<CancellationToken>,
    /// True when the listener can cancel on its own
    self_cancellable: bool

}

impl Listener {
    pub(crate) async fn bind(socket_addr: SocketAddr) -> Result<Listener> {
        let tcp_listener = Option::from(TcpListener::bind(&socket_addr).await?);
        Ok(Listener {
            socket: socket_addr,
            tls_acceptor: None,
            over_tls: false,
            tcp_listener,
            cancellation_token: None,
            self_cancellable: true,
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
        let io = TokioIo::new(tcp_stream);
        debug!("H TTP request received from {}!", socket_addr);
        tokio::task::spawn(async move {
            Self::serve(io).await;
        });
    }

    pub(crate) async fn listen(mut self) -> Listener {
        let cancellation_token = match self.cancellation_token.take() {
            Some(t) => t,
            None => {
                let t = CancellationToken::new();
                self.cancellation_token = Option::from(t.clone());
                t
            }
        };
        let tcp_listener = self.tcp_listener.take().unwrap();
        if self.over_tls {
            let tls_acceptor = self.tls_acceptor.unwrap();
            loop {
                let (tcp_stream, socket_addr) = match Self::listen_or_cancel(&tcp_listener, &cancellation_token).await {
                    // Listener accepted
                    Some((s, a)) => (s, a),
                    // Listener cancelled
                    None => break
                };
                Self::serve_https(tcp_stream, socket_addr, tls_acceptor.clone())
            }
            self.tls_acceptor = Option::from(tls_acceptor);
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
        self.self_cancellable = true;
        self.tcp_listener = Option::from(tcp_listener);
        self
    }

    pub(crate) async fn with_tls_acceptor(mut self, tls_acceptor: TlsAcceptor) -> Listener {
        self.tls_acceptor = Option::from(tls_acceptor);
        self.over_tls = true;
        self
    }

    /// Use your own cancellation token children here.
    pub(crate) fn with_cancellation_token(mut self, cancellation_token: CancellationToken) -> Result<Listener> {
        if self.running() {
            return Err(Error::new(ErrorKind::AddrInUse, "TODO: get_text!(ERR_LISTENER_ALREADY_LISTENING, socket)"))
        }
        self.cancellation_token = Option::from(cancellation_token);
        self.self_cancellable = false;
        Ok(self)
    }

    fn bound_socket(&self) -> &SocketAddr {
        &self.socket
    }

    fn running(&self) -> bool {
        self.tcp_listener.is_none()
    }

    async fn pause(&mut self) -> Result<()> {
        if self.self_cancellable {
            match self.cancellation_token.take() {
                Some(t) => {
                    t.cancel();
                    Ok(())
                },
                None => Err(Error::new(ErrorKind::NotConnected, "TODO: get_text!(ERR_NOT_LISTENING, socket)"))
            }
        } else {
            Err(Error::new(ErrorKind::Unsupported, "TODO: get_text!(ERR_NOT_SELF_CANCELLABLE, socket)"))
        }
    }

    async fn halt(&mut self) -> Result<()> {
        self.pause().await?;
        Ok(())
    }
}

pub(crate) struct ListenerGroup {
    listeners: Vec<Listener>
}

impl ListenerGroup {
    fn new() -> ListenerGroup {
        ListenerGroup {
            listeners: Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use crate::listener::Listener;

    #[tokio::test]
    async fn run_single_listener() {
        let socket
            = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 255)), 0);
        let listener = Listener::bind(socket).await;

    }

    #[test]
    fn run_multiple_listeners() {
        let sockets = [
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 255)), 0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 255)), 0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 255)), 0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 255)), 0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 255)), 0),
        ];


    }
}
