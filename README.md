# FastDistribution

Local-network file distribution using BitTorrent with a simple control plane.

## Workspace layout

- `crates/common`: shared models, config, helpers
- `crates/server`: control-plane API server
- `crates/client`: client polling + progress reporting

## Quick start

Generate a self-signed cert and key (required by the client):

```zsh
openssl req -x509 -newkey rsa:4096 -keyout certificate.key -out certificate.crt -sha256 -days 365 -nodes -subj "/CN=operator.local"
```

Run the server (HTTPS enabled):

```zsh
cargo run -p fast_distribution_server -- --bind 0.0.0.0:12405 --tls-cert certificate.crt --tls-key certificate.key
```

Run a client:

```zsh
cargo run -p fast_distribution_client -- --client-id client-1 --cert-path certificate.crt --download-dir downloads
```

Add a file from another terminal:

```zsh
curl -k -X POST https://operator.local:12405/api/files \
  -H "content-type: application/json" \
  -d '{"file_name":"example.pdf","magnet_link":"magnet:?xt=urn:btih:...","total_bytes":123456,"checksum_hex":null}'

Fetch current progress for all files:

```zsh
curl -k https://operator.local:12405/api/status
```

## Notes

- The client trusts the self-signed cert in `certificate.crt`.
- Progress reporting includes both torrent state and on-disk file state.
- The client uses `librqbit` to manage torrents and report live progress.
- SHA-256 checksums are verified once per file when a checksum is provided.



