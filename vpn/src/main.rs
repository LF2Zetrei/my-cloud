use chacha20poly1305::ChaCha20Poly1305;
use chacha20poly1305::Key;
use chacha20poly1305::KeyInit;
use chacha20poly1305::Nonce;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::aead::AeadCore;
use chacha20poly1305::aead::OsRng;
use common::error::CloudError;
use common::error::Result;
use std::io::Read;
use std::io::Write;
use std::net::UdpSocket;

/// Taille du nonce transmis en clair devant chaque paquet chiffré.
const NONCE_LEN: usize = 12;

/// Chiffre un paquet avec un nonce aléatoire de 96 bits.
/// Retourne [nonce 12 octets][ciphertext + tag 16 octets].
/// Un nonce aléatoire par paquet évite toute réutilisation (même clé des deux côtés,
/// redémarrages), ce qui casserait ChaCha20-Poly1305.
fn chiffrer(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| CloudError::Crypto(e.to_string()))?;

    let mut packet = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    packet.extend_from_slice(&nonce);
    packet.extend_from_slice(&ciphertext);
    Ok(packet)
}

fn dechiffrer(packet: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if packet.len() < NONCE_LEN {
        return Err(CloudError::Crypto(format!("paquet trop court ({} octets)", packet.len())));
    }
    let (nonce, ciphertext) = packet.split_at(NONCE_LEN);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| CloudError::Crypto(e.to_string()))
}

/// Clé partagée : VPN_KEY = 64 caractères hexadécimaux (32 octets).
/// Sans VPN_KEY, on garde l'ancienne clé de démo — à ne pas utiliser en vrai.
fn load_key() -> Result<[u8; 32]> {
    let hex = match std::env::var("VPN_KEY") {
        Ok(h) => h,
        Err(_) => {
            eprintln!("[VPN] VPN_KEY absent : utilisation de la clé de démo (non sécurisée)");
            return Ok([0x42; 32]);
        }
    };
    let hex = hex.trim();
    if hex.len() != 64 {
        return Err(CloudError::Crypto("VPN_KEY doit faire 64 caractères hex".to_string()));
    }
    let mut key = [0u8; 32];
    for (i, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|e| CloudError::Crypto(e.to_string()))?;
    }
    Ok(key)
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let tun_addr = std::env::var("VPN_TUN_ADDR").unwrap_or_else(|_| "10.0.0.1".to_string());
    let mut config = tun::Configuration::default();
    config.address(tun_addr.as_str()).netmask("255.255.255.0").up();

    let dest = std::env::var("VPN_DEST_ADDR").unwrap_or_else(|_| "127.0.0.1:9999".to_string());
    let send_addr = std::env::var("VPN_SEND_ADDR").unwrap_or_else(|_| "0.0.0.0:9998".to_string());
    let recv_addr = std::env::var("VPN_RECV_ADDR").unwrap_or_else(|_| "0.0.0.0:9999".to_string());

    let socket1 = UdpSocket::bind(&send_addr)?;
    let socket2 = UdpSocket::bind(&recv_addr)?;

    let device = tun::create(&config).map_err(|e| {
        CloudError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    // Lecture (TUN → UDP) dans le thread principal, écriture (UDP → TUN) dans l'autre
    let (mut tun_reader, mut tun_writer) = device.split();

    let key = load_key()?;
    let key_clone = key;

    println!("[VPN] Interface TUN créée ({})", tun_addr);

    // Réception : [nonce][ciphertext + tag] → déchiffrement → TUN
    std::thread::spawn(move || {
        loop {
            let mut buf = [0u8; 2048];
            let (n, _addr) = match socket2.recv_from(&mut buf) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Recv error: {}", e);
                    continue;
                }
            };
            let dechiffre = match dechiffrer(&buf[..n], &key_clone) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Decrypt error: {}", e);
                    continue;
                }
            };
            if let Err(e) = tun_writer.write_all(&dechiffre) {
                eprintln!("TUN write error: {}", e);
            }
        }
    });

    // Émission : TUN → chiffrement → [nonce][ciphertext] → UDP
    loop {
        let mut buf = [0u8; 2048];
        let n = match tun_reader.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Read error: {}", e);
                continue;
            }
        };
        let packet = match chiffrer(&buf[..n], &key) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Encrypt error: {}", e);
                continue;
            }
        };
        if let Err(e) = socket1.send_to(&packet, dest.as_str()) {
            eprintln!("Send error: {}", e);
        }
    }
}
