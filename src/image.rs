use aes::cipher::{BlockDecryptMut, KeyIvInit};
use aes_gcm::{aead::Aead, Aes256Gcm, AesGcm, KeyInit};

type PKCS7128CbcDec = cbc::Decryptor<aes::Aes128>;

pub enum ImageError {
    Io(std::io::Error),
    Image(image::ImageError),
    RSA(crate::kp::RSAError),
    AESGCM(aes_gcm::Error),
    AESLength(aes::cipher::InvalidLength),
    AESCBCUnpad(aes::cipher::block_padding::UnpadError),
}

pub(crate) fn decrypt_image(
    image: &[u8],
    aes_key: &[u8],
) -> Result<image::DynamicImage, ImageError> {
    let decrypted = decrypt_data(image, aes_key)?;

    let read_img = image::load_from_memory(&decrypted)?;
    Ok(read_img)
}

fn decrypt_data(image: &[u8], aes_key: &[u8]) -> Result<Vec<u8>, ImageError> {
    if image[0] == 2 {
        decrypt_with_aes_gcm(image, aes_key)
    } else {
        decrypt_with_aes_cbc(image, aes_key)
    }
}

fn decrypt_with_aes_gcm(image: &[u8], aes_key: &[u8]) -> Result<Vec<u8>, ImageError> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(aes_key);
    let cipher = Aes256Gcm::new(&key);

    let nonce = &image[2..18];
    let ciphertext = &image[18..];
    let decrypted = cipher.decrypt(nonce.into(), ciphertext)?;

    Ok(decrypted)
}

fn decrypt_with_aes_cbc(image: &[u8], aes_key: &[u8]) -> Result<Vec<u8>, ImageError> {
    // # AES-CBC
    // iv = encrypted_image[0:16]
    // ciphertext = encrypted_image[16:]
    // cipher = Cipher(algorithms.AES(aes_key), modes.CBC(iv))
    // decryptor = cipher.decryptor()
    // unpadder = sym_padding.PKCS7(128).unpadder()
    // padded_data = decryptor.update(ciphertext) + decryptor.finalize()
    // decrypted_image = unpadder.update(padded_data) + unpadder.finalize()
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
