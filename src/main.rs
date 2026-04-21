mod obf;
mod secrets;
mod socks;
mod tailscale;

use std::env;
use std::net::ToSocketAddrs;
use std::process::ExitCode;
use std::sync::Arc;

use tailscale::TailnetDialer;

fn usage(bin: &str) {
    println!("Usage: {bin} [hostname] [authkey] [control-url]");
}

fn sanitize_hostname(value: &str) -> Option<String> {
    let mut out = value
        .trim()
        .chars()
        .map(|c| if c == '_' { '-' } else { c })
        .collect::<String>();
    out.truncate(63);
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn default_hostname() -> String {
    env::var("HOSTNAME")
        .ok()
        .and_then(|h| sanitize_hostname(&h))
        .unwrap_or_else(|| "socktail-rs".to_string())
}

fn get_bind_config() -> (String, u16) {
    let addr = obfstr!("0.0.0.0").into_string();
    let port = obfstr!("1080")
        .as_str()
        .parse::<u16>()
        .ok()
        .unwrap_or(1080);
    (addr, port)
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();

    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        usage(&args[0]);
        return ExitCode::SUCCESS;
    }

    if args.len() > 4 {
        usage(&args[0]);
        return ExitCode::from(1);
    }

    let hostname = args.get(1).and_then(|s| sanitize_hostname(s)).unwrap_or_else(default_hostname);
    let auth_key = args.get(2).cloned().or_else(secrets::build_auth_key);
    let control_url = args.get(3).cloned().or_else(secrets::build_control_url);

    let dialer = match TailnetDialer::new(hostname, auth_key, control_url) {
        Ok(dialer) => Arc::new(dialer),
        Err(_) => return ExitCode::from(1),
    };

    let (bind_addr, port) = get_bind_config();

    if (bind_addr.as_str(), port).to_socket_addrs().is_err() {
        return ExitCode::from(1);
    }

    tokio::select! {
        res = socks::run(dialer, &bind_addr, port) => {
            if res.is_err() {
                return ExitCode::from(1);
            }
        }
        _ = tokio::signal::ctrl_c() => {}
    }

    ExitCode::SUCCESS
}
