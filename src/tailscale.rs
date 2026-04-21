use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct TailnetDialer {
    device: Arc<tailscale::Device>,
}

fn map_error(err: tailscale::Error) -> io::Error {
    io::Error::other(err.to_string())
}

impl TailnetDialer {
    pub async fn new(
        hostname: String,
        auth_key: Option<String>,
        control_url: Option<String>,
    ) -> io::Result<Self> {
        std::env::set_var("TS_RS_EXPERIMENT", "this_is_unstable_software");

        let mut config = tailscale::Config::default();
        if !hostname.is_empty() {
            config.requested_hostname = Some(hostname);
        }

        if let Some(control_url) = control_url.filter(|v| !v.is_empty()) {
            config.control_server_url = control_url
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid control-url"))?;
        }

        let device = tailscale::Device::new(&config, auth_key.filter(|v| !v.is_empty()))
            .await
            .map_err(map_error)?;

        Ok(Self {
            device: Arc::new(device),
        })
    }

    pub async fn dial_tcp(&self, remote: SocketAddr) -> io::Result<tailscale::TcpStream> {
        self.device.tcp_connect(remote).await.map_err(map_error)
    }
}
