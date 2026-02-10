// TODO: Rewrite these tests after stub is replaced.
#[cfg(test)]
mod tests {
    use crate::models::MediaMetadata;
    use crate::services::metadata::{
        MediaSearchFilters, MediaSortField, MetadataConfig, MetadataService, SortOrder,
        StubMetadataService,
    };
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_service(cache_dir: PathBuf) -> StubMetadataService {
        StubMetadataService::new(MetadataConfig { cache_dir })
    }

    #[tokio::test]
    async fn test_get_media_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let service = create_service(temp_dir.path().to_path_buf());
        let metadata = service.get_media_metadata("any_id").await;

        assert!(metadata.is_some());
        match metadata.unwrap() {
            MediaMetadata::Movie(movie) => {
                assert_eq!(movie.title.original, "Unknown Movie");
            }
            _ => panic!("Expected Movie metadata"),
        }
    }

    #[tokio::test]
    async fn test_search_media_pagination_first_page() {
        let temp_dir = TempDir::new().unwrap();
        let service = create_service(temp_dir.path().to_path_buf());

        let connection = service
            .search_media(
                Some(2), // limit
                None,    // after
                None,    // last
                None,    // before
                MediaSortField::Title,
                SortOrder::Asc,
                MediaSearchFilters {
                    media_type: None,
                    genre: None,
                    year: None,
                    year_from: None,
                    year_to: None,
                    query: None,
                    min_rating: None,
                },
            )
            .await;

        assert_eq!(connection.edges.len(), 2);
        assert!(connection.page_info.has_next_page);
        assert!(!connection.page_info.has_previous_page);
        assert_eq!(connection.edges[0].node.title().original, "Sample Movie 1");
        assert_eq!(connection.edges[1].node.title().original, "Sample Movie 2");
    }

    #[tokio::test]
    async fn test_search_media_pagination_next_page() {
        let temp_dir = TempDir::new().unwrap();
        let service = create_service(temp_dir.path().to_path_buf());

        // First page to get cursor
        let first_page = service
            .search_media(
                Some(1),
                None,
                None,
                None,
                MediaSortField::Title,
                SortOrder::Asc,
                MediaSearchFilters {
                    media_type: None,
                    genre: None,
                    year: None,
                    year_from: None,
                    year_to: None,
                    query: None,
                    min_rating: None,
                },
            )
            .await;

        let cursor = first_page.page_info.end_cursor.unwrap();

        // Second page
        let second_page = service
            .search_media(
                Some(2),
                Some(cursor),
                None,
                None,
                MediaSortField::Title,
                SortOrder::Asc,
                MediaSearchFilters {
                    media_type: None,
                    genre: None,
                    year: None,
                    year_from: None,
                    year_to: None,
                    query: None,
                    min_rating: None,
                },
            )
            .await;

        assert_eq!(second_page.edges.len(), 2);
        // mocked data has 3 items: movie1, movie2, movie3.
        // Page 1 (limit 1): movie1. Cursor -> movie1.
        // Page 2 (limit 2, after movie1): movie2, movie3.

        assert_eq!(second_page.edges[0].node.title().original, "Sample Movie 2");
        assert_eq!(second_page.edges[1].node.title().original, "Sample Movie 3");
        assert!(!second_page.page_info.has_next_page); // No more after movie3
    }

    #[tokio::test]
    async fn test_search_media_pagination_bounds() {
        let temp_dir = TempDir::new().unwrap();
        let service = create_service(temp_dir.path().to_path_buf());

        // Request more than available
        let connection = service
            .search_media(
                Some(100),
                None,
                None,
                None,
                MediaSortField::Title,
                SortOrder::Asc,
                MediaSearchFilters {
                    media_type: None,
                    genre: None,
                    year: None,
                    year_from: None,
                    year_to: None,
                    query: None,
                    min_rating: None,
                },
            )
            .await;

        assert_eq!(connection.edges.len(), 3); // Total mock items
        assert!(!connection.page_info.has_next_page);
    }
}
