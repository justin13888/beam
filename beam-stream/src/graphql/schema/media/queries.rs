use async_graphql::*;

use crate::models::{MediaMetadata, SeasonMetadata, ShowDates, ShowMetadata, Title};

pub struct MediaQuery;

#[Object]
impl MediaQuery {
    /// Fetch media metadata by ID
    async fn metadata(&self, ctx: &Context<'_>, id: ID) -> Result<Option<MediaMetadata>> {
        let media_metadata = MediaMetadata::Show(ShowMetadata {
            title: Title {
                original: String::from("Unknown Title"),
                localized: None,
                alternatives: None,
            },
            description: None,
            year: None,
            seasons: vec![SeasonMetadata {
                season_number: 1,
                dates: ShowDates {
                    first_aired: None,
                    last_aired: None,
                },
                episode_runtime: None,
                episodes: vec![],
                poster_url: None,
                genres: vec![],
                ratings: None,
            }],
            identifiers: None,
        });

        Ok(Some(media_metadata))
    }
}
