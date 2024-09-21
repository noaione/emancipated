use std::path::Path;

use base64::{engine::general_purpose, Engine as _};
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey};
use rsa::sha2::Sha256;
use rsa::{pkcs8::DecodePublicKey, RsaPrivateKey, RsaPublicKey};

pub(crate) enum RSAError {
    Io(std::io::Error),
    RSA(rsa::Error),
    PKCS8(rsa::pkcs8::Error),
    SpkiPKCS8(rsa::pkcs8::spki::Error),
    B64Decode(base64::DecodeError),
}

impl From<std::io::Error> for RSAError {
    fn from(e: std::io::Error) -> Self {
        RSAError::Io(e)
    }
}

impl From<rsa::Error> for RSAError {
    fn from(e: rsa::Error) -> Self {
        RSAError::RSA(e)
    }
}

impl From<rsa::pkcs8::Error> for RSAError {
    fn from(e: rsa::pkcs8::Error) -> Self {
        RSAError::PKCS8(e)
    }
}

impl From<rsa::pkcs8::spki::Error> for RSAError {
    fn from(e: rsa::pkcs8::spki::Error) -> Self {
        RSAError::SpkiPKCS8(e)
    }
}

impl From<base64::DecodeError> for RSAError {
    fn from(e: base64::DecodeError) -> Self {
        RSAError::B64Decode(e)
    }
}

impl std::fmt::Display for RSAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RSAError::Io(e) => write!(f, "IO Error: {}", e),
            RSAError::RSA(e) => write!(f, "RSA Error: {}", e),
            RSAError::PKCS8(e) => write!(f, "PKCS8 Error: {}", e),
            RSAError::SpkiPKCS8(e) => write!(f, "SPKI PKCS8 Error: {}", e),
            RSAError::B64Decode(e) => write!(f, "Base64 Decode Error: {}", e),
        }
    }
}

impl std::fmt::Debug for RSAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RSAError::Io(e) => write!(f, "IO Error: {}", e),
            RSAError::RSA(e) => write!(f, "RSA Error: {}", e),
            RSAError::PKCS8(e) => write!(f, "PKCS8 Error: {}", e),
            RSAError::SpkiPKCS8(e) => write!(f, "SPKI PKCS8 Error: {}", e),
            RSAError::B64Decode(e) => write!(f, "Base64 Decode Error: {}", e),
        }
    }
}

pub(crate) fn generate_key_pair() -> Result<(RsaPrivateKey, RsaPublicKey), RSAError> {
    // Generate Key-Pair of 2048 bits with 65537 exponent
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, 2048)?;
    let pub_key = RsaPublicKey::from(&priv_key);

    Ok((priv_key, pub_key))
}

pub(crate) fn load_key_pair(
    private_key: &Path,
    public_key: &Path,
) -> Result<(RsaPrivateKey, RsaPublicKey), RSAError> {
    let read_pk = RsaPrivateKey::read_pkcs8_pem_file(private_key)?;
    let read_pub = RsaPublicKey::read_public_key_pem_file(public_key)?;

    Ok((read_pk, read_pub))
}

pub(crate) fn write_key_pair(
    private_key: &Path,
    public_key: &Path,
    priv_key: &RsaPrivateKey,
    pub_key: &RsaPublicKey,
) -> Result<(), RSAError> {
    #[cfg(target_os = "windows")]
    let line_ending = rsa::pkcs8::LineEnding::CRLF;
    #[cfg(not(target_os = "windows"))]
    let line_ending = rsa::pkcs8::LineEnding::LF;

    let priv_key_pkcs8 = priv_key.to_pkcs8_pem(line_ending)?;
    let pub_key_pkcs1 = pub_key.to_public_key_pem(line_ending)?;

    std::fs::write(private_key, priv_key_pkcs8)?;
    std::fs::write(public_key, pub_key_pkcs1)?;

    Ok(())
}

pub(crate) fn create_xhash(public_key: &RsaPublicKey) -> Result<String, RSAError> {
    let spki_der = public_key.to_public_key_der()?;

    let spki_b64 = general_purpose::STANDARD.encode(spki_der.as_bytes());

    Ok(spki_b64)
}

pub(crate) fn hash_to_aes_key(
    private_key: &RsaPrivateKey,
    hash: &str,
) -> Result<Vec<u8>, RSAError> {
    let hash_bytes = general_purpose::STANDARD.decode(hash)?;

    let decryped_key = private_key.decrypt(rsa::Oaep::new::<Sha256>(), &hash_bytes)?;

    Ok(decryped_key)
}

pub(crate) fn hash_b64(data: &str) -> String {
    let data_bytes = data.as_bytes();

    general_purpose::STANDARD.encode(data_bytes)
}
