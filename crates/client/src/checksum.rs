use sha2::{Digest, Sha256};
use std::path::Path;

pub async fn verify_checksum(path: &Path, expected_hex: &str) -> anyhow::Result<bool> {
    let expected_hex = expected_hex.trim().to_lowercase();
    let path = path.to_owned();
    let digest = tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<u8>> {
        let mut file = std::fs::File::open(&path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 1024 * 64];
        loop {
            let read = std::io::Read::read(&mut file, &mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(hasher.finalize().to_vec())
    })
    .await??;

    let actual_hex = hex_encode(&digest);
    Ok(actual_hex == expected_hex)
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

