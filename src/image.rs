use std::io::Write;

use aes::{
    cipher::{consts::U16, BlockDecryptMut, KeyIvInit},
    Aes256,
};
use aes_gcm::{aead::Aead, AesGcm, KeyInit};

type Aes256Gcm16 = AesGcm<Aes256, U16>;
type PKCS7128CbcDec = cbc::Decryptor<aes::Aes128>;

pub enum ImageError {
    Io(std::io::Error),
    Image(image::ImageError),
    RSA(crate::kp::RSAError),
    AESGCM(aes_gcm::Error),
    AESLength(aes::cipher::InvalidLength),
    AESCBCUnpad(aes::cipher::block_padding::UnpadError),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::Io(e) => write!(f, "IO Error: {}", e),
            ImageError::Image(e) => write!(f, "Image Error: {}", e),
            ImageError::RSA(e) => write!(f, "RSA Error: {}", e),
            ImageError::AESGCM(e) => write!(f, "AES GCM Error: {}", e),
            ImageError::AESLength(e) => write!(f, "AES Length Error: {}", e),
            ImageError::AESCBCUnpad(e) => write!(f, "AES CBC Unpad Error: {}", e),
        }
    }
}

impl std::fmt::Debug for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::Io(e) => write!(f, "IO Error: {}", e),
            ImageError::Image(e) => write!(f, "Image Error: {}", e),
            ImageError::RSA(e) => write!(f, "RSA Error: {}", e),
            ImageError::AESGCM(e) => write!(f, "AES GCM Error: {}", e),
            ImageError::AESLength(e) => write!(f, "AES Length Error: {}", e),
            ImageError::AESCBCUnpad(e) => write!(f, "AES CBC Unpad Error: {}", e),
        }
    }
}

fn decrypt_data(image: &[u8], aes_key: &[u8]) -> Result<Vec<u8>, ImageError> {
    if image[0] == 2 {
        decrypt_with_aes_gcm(image, aes_key)
    } else {
        decrypt_with_aes_cbc(image, aes_key)
    }
}

pub(crate) fn load_and_save_image(
    image: &[u8],
    aes_key: &[u8],
    target_dir: &std::path::Path,
) -> Result<(), ImageError> {
    let decrypted = decrypt_data(image, aes_key)?;
    let extension = image::guess_format(&decrypted).unwrap_or(image::ImageFormat::Png);

    // Try loading the image
    image::load_from_memory(&decrypted)?;

    // Save the image
    // Path is already name but without extension, so we add the extension
    let ext_str = extension.extensions_str()[0];
    let path = target_dir.with_extension(ext_str);

    // Open the file and write the image
    let file = std::fs::File::create(&path)?;
    let mut writer = std::io::BufWriter::new(file);
    writer.write_all(&decrypted)?;

    Ok(())
}

fn decrypt_with_aes_gcm(image: &[u8], aes_key: &[u8]) -> Result<Vec<u8>, ImageError> {
    let key = aes_gcm::Key::<Aes256Gcm16>::from_slice(aes_key);
    let cipher = Aes256Gcm16::new(&key);

    let nonce = &image[2..18];
    let ciphertext = &image[18..];
    let decrypted = cipher.decrypt(nonce.into(), ciphertext)?;

    Ok(decrypted)
}

fn decrypt_with_aes_cbc(image: &[u8], aes_key: &[u8]) -> Result<Vec<u8>, ImageError> {
    let iv = &image[0..16];
    let mut ciphertext = image[16..].to_vec();
    let cipher = PKCS7128CbcDec::new(aes_key.into(), iv.into());

    let decrypt =
        cipher.decrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(&mut ciphertext)?;
    Ok(decrypt.to_vec())
}

impl From<std::io::Error> for ImageError {
    fn from(e: std::io::Error) -> Self {
        ImageError::Io(e)
    }
}

impl From<image::ImageError> for ImageError {
    fn from(e: image::ImageError) -> Self {
        ImageError::Image(e)
    }
}

impl From<crate::kp::RSAError> for ImageError {
    fn from(e: crate::kp::RSAError) -> Self {
        ImageError::RSA(e)
    }
}

impl From<aes_gcm::Error> for ImageError {
    fn from(e: aes_gcm::Error) -> Self {
        ImageError::AESGCM(e)
    }
}

impl From<aes::cipher::InvalidLength> for ImageError {
    fn from(e: aes::cipher::InvalidLength) -> Self {
        ImageError::AESLength(e)
    }
}

impl From<aes::cipher::block_padding::UnpadError> for ImageError {
    fn from(e: aes::cipher::block_padding::UnpadError) -> Self {
        ImageError::AESCBCUnpad(e)
    }
}
