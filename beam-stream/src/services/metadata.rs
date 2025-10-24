use crate::models::MediaMetadata;
use std::sync::{Arc, LazyLock};

/// Global metadata service instance.
///
/// This is initialized lazily on first use and provides access to the metadata service
/// for retrieving media metadata information.
pub static METADATA_SERVICE: LazyLock<Arc<MetadataService>> =
    LazyLock::new(|| Arc::new(MetadataService::new()));

#[derive(Debug)]
pub struct MetadataService {}

impl MetadataService {
    fn new() -> Self {
        tracing::info!("Initialized MetadataService");
        MetadataService {}
    }

    /// Get media metadata by media ID
    pub fn get_media_metadata(&self, media_id: &str) -> Option<MediaMetadata> {
        todo!()
    }
}
