use chacha20poly1305::ChaCha20Poly1305;
use chacha20poly1305::Key;
use chacha20poly1305::KeyInit;
use chacha20poly1305::Nonce;
use chacha20poly1305::aead::Aead;
use common::error::CloudError;
use common::error::Result;
use std::io::Read;
use std::net::UdpSocket;

fn chiffrer(data: &[u8], key: &[u8; 32], nonce: u64) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));

    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[..8].copy_from_slice(&nonce.to_le_bytes());
    cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), data)
        .map_err(|e| CloudError::Crypto(e.to_string()))
}

fn dechiffrer(data: &[u8], key: &[u8; 32], nonce: u64) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));

    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[..8].copy_from_slice(&nonce.to_le_bytes());
    cipher
        .decrypt(Nonce::from_slice(&nonce_bytes), data)
        .map_err(|e| CloudError::Crypto(e.to_string()))
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let mut config = tun::Configuration::default();
    config.address("10.0.0.1").netmask("255.255.255.0").up();

    let dest = std::env::var("VPN_DEST_ADDR").unwrap_or_else(|_| "127.0.0.1:9999".to_string());
    let send_addr = std::env::var("VPN_SEND_ADDR").unwrap_or_else(|_| "0.0.0.0:9998".to_string());
    let recv_addr = std::env::var("VPN_RECV_ADDR").unwrap_or_else(|_| "0.0.0.0:9999".to_string());

    let socket1 = UdpSocket::bind(&send_addr)?;
    let socket2 = UdpSocket::bind(&recv_addr)?;

    let mut device = tun::create(&config).map_err(|e| {
        CloudError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    let key: [u8; 32] = [0x42; 32];
    let key_clone = key.clone();

    println!("[VPN] Interface TUN créée");

    std::thread::spawn(move || {
        let mut nonce_recv: u64 = 0;
        loop {
            let mut buf = [0u8; 1504];
            let (n, _addr) = match socket2.recv_from(&mut buf) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Recv error: {}", e);
                    continue;
                }
            };
            let dechiffre = match dechiffrer(&buf[..n], &key_clone, nonce_recv) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Decrypt error: {}", e);
                    continue;
                }
            };
            nonce_recv += 1;
            let _ = dechiffre;
        }
    });

    let mut nonce_send: u64 = 0;
    loop {
        let mut buf = [0u8; 1504];
        let n = match device.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Read error: {}", e);
                continue;
            }
        };
        let chiffre = match chiffrer(&buf[..n], &key, nonce_send) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Encrypt error: {}", e);
                continue;
            }
        };
        nonce_send += 1;
        match socket1.send_to(&chiffre, dest.as_str()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Send error: {}", e);
                continue;
            }
        }
    }
}
