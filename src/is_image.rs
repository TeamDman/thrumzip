use crate::PathInsideZip;

pub fn is_image_extension(path: &PathInsideZip) -> bool {
    path.inner
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            matches!(
                ext_lower.as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "tif"
            )
        })
        .unwrap_or(false)
}
