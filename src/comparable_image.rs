use std::sync::Arc;

use img_hash::{HashAlg, Hasher, HasherConfig, ImageHash};

pub enum ComparableImage {
    Bytes {
        bytes: Vec<u8>,
    },
    Image {
        image: image::DynamicImage,
    },
    HashedImage {
        image: image::DynamicImage,
        hash: ImageHash,
        hasher: Arc<Hasher>,
    },
}

/// Create a perceptual hasher
fn image_hasher() -> img_hash::Hasher {
    HasherConfig::new().hash_alg(HashAlg::Gradient).to_hasher()
}

impl ComparableImage {
    pub fn new_bytes(bytes: Vec<u8>) -> Self {
        Self::Bytes { bytes }
    }

    pub fn new_image(image: image::DynamicImage) -> Self {
        Self::Image { image }
    }

    pub fn new_hashed_image(image: image::DynamicImage, hasher: Arc<Hasher>) -> Self {
        let hash = hasher.hash_image(&image);
        Self::HashedImage {
            image,
            hash,
            hasher,
        }
    }
}