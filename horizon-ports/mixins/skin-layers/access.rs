// The decoded 64x64 RGBA of a downloaded skin, for 3D Skin Layers.
pub fn image<'a>(downloads: &'a super::Downloader, location: &str) -> Option<&'a [u8]> {
    downloads.images.get(location).map(Vec::as_slice)
}
