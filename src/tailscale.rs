use std::io;
use std::sync::{Arc, Mutex};

use tempfile::TempDir;
use tokio::net::TcpStream;
use tokio::task;
use tsnet::{Network, Server, ServerBuilder};

pub struct TailnetDialer {
    server: Arc<Mutex<Server>>,
    _state_dir: TempDir,
}

fn map_tsnet_error(err: tsnet::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err.to_string())
}

impl TailnetDialer {
    pub fn new(hostname: String, auth_key: Option<String>, control_url: Option<String>) -> io::Result<Self> {
        let state_dir = tempfile::Builder::new()
            .prefix("socktail-tsnet-")
            .tempdir_in(std::env::temp_dir())?;

        let mut builder = ServerBuilder::new()
            .dir(state_dir.path().to_path_buf())
            .ephemeral()
            .disable_log();

        if !hostname.is_empty() {
            builder = builder.hostname(&hostname);
        }

        if let Some(auth_key) = auth_key.filter(|v| !v.is_empty()) {
            builder = builder.authkey(auth_key);
        }

        if let Some(control_url) = control_url.filter(|v| !v.is_empty()) {
            builder = builder.control_url(control_url);
        }

        let server = builder.build().map_err(map_tsnet_error)?;

        Ok(Self {
            server: Arc::new(Mutex::new(server)),
            _state_dir: state_dir,
        })
    }

    pub async fn dial_tcp(&self, target: String) -> io::Result<TcpStream> {
        let server = Arc::clone(&self.server);
        let std_stream = task::spawn_blocking(move || {
            let guard = server
                .lock()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "tailscale lock poisoned"))?;
            let stream = guard.connect(Network::Tcp, &target).map_err(map_tsnet_error)?;
            stream.set_nonblocking(true)?;
            Ok::<std::net::TcpStream, io::Error>(stream)
        })
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "tailscale dial task failed"))??;

        TcpStream::from_std(std_stream)
    }
}
