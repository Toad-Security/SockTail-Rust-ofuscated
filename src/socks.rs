use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

use tokio::io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt};
use tokio::net::{lookup_host, TcpListener, TcpStream};

use crate::tailscale::TailnetDialer;

const SOCKS_VERSION: u8 = 0x05;
const NO_AUTH: u8 = 0x00;
const NO_ACCEPTABLE_AUTH: u8 = 0xFF;
const CMD_CONNECT: u8 = 0x01;
const ATYP_IPV4: u8 = 0x01;
const ATYP_DOMAIN: u8 = 0x03;
const ATYP_IPV6: u8 = 0x04;
const REP_SUCCEEDED: u8 = 0x00;
const REP_GENERAL_FAILURE: u8 = 0x01;
const REP_COMMAND_NOT_SUPPORTED: u8 = 0x07;
const REP_ADDRESS_TYPE_NOT_SUPPORTED: u8 = 0x08;

pub async fn run(dialer: Arc<TailnetDialer>, bind_addr: &str, port: u16) -> io::Result<()> {
    let listener = TcpListener::bind((bind_addr, port)).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let dialer = Arc::clone(&dialer);
        tokio::spawn(async move {
            let _ = handle_connection(stream, dialer).await;
        });
    }
}

async fn handle_connection(mut client: TcpStream, dialer: Arc<TailnetDialer>) -> io::Result<()> {
    if !handle_auth(&mut client).await? {
        return Ok(());
    }

    let request = match parse_connect_request(&mut client).await {
        Ok(request) => request,
        Err(status) => {
            let _ = send_connect_response(&mut client, status).await;
            return Ok(());
        }
    };

    let remote = match resolve_target(&request.host, request.port).await {
        Ok(remote) => remote,
        Err(_) => {
            let _ = send_connect_response(&mut client, REP_GENERAL_FAILURE).await;
            return Ok(());
        }
    };

    let mut upstream = match dialer.dial_tcp(remote).await {
        Ok(stream) => stream,
        Err(_) => {
            let _ = send_connect_response(&mut client, REP_GENERAL_FAILURE).await;
            return Ok(());
        }
    };

    send_connect_response(&mut client, REP_SUCCEEDED).await?;
    let _ = copy_bidirectional(&mut client, &mut upstream).await;
    Ok(())
}

async fn resolve_target(host: &str, port: u16) -> io::Result<SocketAddr> {
    if let Ok(addr) = format!("{host}:{port}").parse::<SocketAddr>() {
        return Ok(addr);
    }

    let mut resolved = lookup_host((host, port)).await?;
    resolved
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "target not resolved"))
}

struct ConnectRequest {
    host: String,
    port: u16,
}

async fn handle_auth(stream: &mut TcpStream) -> io::Result<bool> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header).await?;

    if header[0] != SOCKS_VERSION {
        return Ok(false);
    }

    let method_count = header[1] as usize;
    let mut methods = vec![0u8; method_count];
    stream.read_exact(&mut methods).await?;

    if methods.contains(&NO_AUTH) {
        stream.write_all(&[SOCKS_VERSION, NO_AUTH]).await?;
        return Ok(true);
    }

    stream
        .write_all(&[SOCKS_VERSION, NO_ACCEPTABLE_AUTH])
        .await?;
    Ok(false)
}

async fn parse_connect_request(stream: &mut TcpStream) -> Result<ConnectRequest, u8> {
    let mut req = [0u8; 4];
    stream
        .read_exact(&mut req)
        .await
        .map_err(|_| REP_GENERAL_FAILURE)?;

    if req[0] != SOCKS_VERSION {
        return Err(REP_GENERAL_FAILURE);
    }

    if req[1] != CMD_CONNECT {
        return Err(REP_COMMAND_NOT_SUPPORTED);
    }

    let host = match req[3] {
        ATYP_IPV4 => {
            let mut octets = [0u8; 4];
            stream
                .read_exact(&mut octets)
                .await
                .map_err(|_| REP_GENERAL_FAILURE)?;
            Ipv4Addr::from(octets).to_string()
        }
        ATYP_DOMAIN => {
            let mut len = [0u8; 1];
            stream
                .read_exact(&mut len)
                .await
                .map_err(|_| REP_GENERAL_FAILURE)?;
            let mut domain = vec![0u8; len[0] as usize];
            stream
                .read_exact(&mut domain)
                .await
                .map_err(|_| REP_GENERAL_FAILURE)?;
            String::from_utf8_lossy(&domain).into_owned()
        }
        ATYP_IPV6 => {
            let mut octets = [0u8; 16];
            stream
                .read_exact(&mut octets)
                .await
                .map_err(|_| REP_GENERAL_FAILURE)?;
            Ipv6Addr::from(octets).to_string()
        }
        _ => return Err(REP_ADDRESS_TYPE_NOT_SUPPORTED),
    };

    let mut port_buf = [0u8; 2];
    stream
        .read_exact(&mut port_buf)
        .await
        .map_err(|_| REP_GENERAL_FAILURE)?;
    let port = u16::from_be_bytes(port_buf);

    Ok(ConnectRequest { host, port })
}

async fn send_connect_response(stream: &mut TcpStream, status: u8) -> io::Result<()> {
    let mut response = [0u8; 10];
    response[0] = SOCKS_VERSION;
    response[1] = status;
    response[2] = 0x00;
    response[3] = ATYP_IPV4;
    response[4..8].copy_from_slice(&[0, 0, 0, 0]);
    response[8..10].copy_from_slice(&0u16.to_be_bytes());
    stream.write_all(&response).await
}
