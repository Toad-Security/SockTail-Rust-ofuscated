# SockTail (Rust)

Binario autocontenido para administración autorizada sobre Tailnet: se une a Tailscale y expone un proxy SOCKS5 en `0.0.0.0:1080`.

## Comportamiento

- CLI: `./socktail [hostname] [authkey] [control-url]`
- Los 3 argumentos son opcionales.
- Puerto SOCKS5 fijo: `1080`.
- SOCKS5 implementado manualmente (NO_AUTH + CONNECT para IPv4/IPv6/Dominio).
- Sin configuración persistente en disco.
- Salida silenciosa por defecto (sin logs verbosos).

## Dependencias de toolchain

- Rust estable

## Build

```bash
# Build release (sin secretos embebidos)
make build

# Build release con AUTH_KEY embebida (CONTROL_URL opcional)
make build-with-key AUTH_KEY=tskey-auth-xxxxx [CONTROL_URL=https://headscale.example.com]

# Build multi-plataforma con AUTH_KEY (si toolchains/linkers están disponibles)
make build-all-with-key AUTH_KEY=tskey-auth-xxxxx [CONTROL_URL=https://headscale.example.com]
```

## Build-time secrets

`build.rs` genera `src/secrets.rs` en compilación.

- Si defines `AUTH_KEY` y/o `CONTROL_URL` al compilar, se guardan ofuscados (XOR + variación por índice).
- Cada build cambia la clave interna de ofuscación para variar el binario.
- Si no defines variables, el binario no embebe secretos.

## Uso

```bash
# Todo por defecto
./target/release/socktail-rust-ofuscated

# Hostname custom
./target/release/socktail-rust-ofuscated admin-proxy

# Hostname + authkey en runtime
./target/release/socktail-rust-ofuscated admin-proxy tskey-auth-xxxxx

# Hostname + authkey + control-url en runtime
./target/release/socktail-rust-ofuscated admin-proxy tskey-auth-xxxxx https://headscale.example.com
```
