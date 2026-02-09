use beam_stream::entities::{
    episode, indexed_file, library, movie, movie_file, season, show, stream_cache,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, Database, DbErr};

const DB_URL: &str = "postgres://beam:password@localhost:5432";
const TEST_DB: &str = "beam_test";

// TODO: This isn't done. Need to verify migrations matches all entity definitions

async fn prepare_db() -> Result<String, DbErr> {
    let url = DB_URL;
    let db = Database::connect(url).await?;

    let _ = db
        .execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!("DROP DATABASE IF EXISTS \"{}\";", TEST_DB),
        ))
        .await?;

    let _ = db
        .execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!("CREATE DATABASE \"{}\";", TEST_DB),
        ))
        .await?;

    let url = format!("{}/{}", DB_URL, TEST_DB);
    let db = Database::connect(&url).await?;

    Migrator::up(&db, None).await?;

    Ok(url)
}

#[tokio::test]
async fn verify_schema_integrity() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Prepare clean test database
    let db_url = prepare_db().await?;
    let db = Database::connect(&db_url).await?;

    // 2. Insert Library
    let library = library::ActiveModel {
        name: ActiveValue::Set("Test Library".to_owned()),
        root_path: ActiveValue::Set("/tmp/test_library".to_owned()),
        media_type: ActiveValue::Set("mixed".to_owned()),
        ..Default::default()
    };
    let library = library.insert(&db).await?;
    println!("Inserted Library: {:?}", library.id);

    // 3. Insert IndexedFile
    let indexed_file = indexed_file::ActiveModel {
        library_id: ActiveValue::Set(library.id),
        file_path: ActiveValue::Set("/tmp/test_library/movie.mp4".to_owned()),
        file_hash: ActiveValue::Set("abcdef123456".to_owned()),
        file_size: ActiveValue::Set(1024 * 1024),
        mime_type: ActiveValue::Set(Some("video/mp4".to_owned())),
        ..Default::default()
    };
    let indexed_file = indexed_file.insert(&db).await?;
    println!("Inserted IndexedFile: {:?}", indexed_file.id);

    // 4. Insert Movie with Array types (Postgres specific)
    let movie = movie::ActiveModel {
        library_id: ActiveValue::Set(library.id),
        title: ActiveValue::Set("Test Movie".to_owned()),
        title_alternatives: ActiveValue::Set(Some(vec![
            "Alternate Title 1".to_owned(),
            "Alternate Title 2".to_owned(),
        ])),
        genres: ActiveValue::Set(Some(vec!["Action".to_owned(), "Sci-Fi".to_owned()])),
        ..Default::default()
    };
    let movie = movie.insert(&db).await?;
    println!("Inserted Movie: {:?}", movie.id);

    // Verify Array types
    assert_eq!(
        movie.title_alternatives,
        Some(vec![
            "Alternate Title 1".to_owned(),
            "Alternate Title 2".to_owned()
        ])
    );
    assert_eq!(
        movie.genres,
        Some(vec!["Action".to_owned(), "Sci-Fi".to_owned()])
    );

    // 5. Insert MovieFile (Junction)
    let movie_file = movie_file::ActiveModel {
        movie_id: ActiveValue::Set(movie.id),
        file_id: ActiveValue::Set(indexed_file.id),
        is_primary: ActiveValue::Set(true),
    };
    let _ = movie_file.insert(&db).await?;
    println!("Linked Movie to File");

    // 6. Insert Show, Season, Episode
    let show = show::ActiveModel {
        library_id: ActiveValue::Set(library.id),
        title: ActiveValue::Set("Test Show".to_owned()),
        ..Default::default()
    };
    let show = show.insert(&db).await?;

    let season = season::ActiveModel {
        show_id: ActiveValue::Set(show.id),
        season_number: ActiveValue::Set(1),
        genres: ActiveValue::Set(Some(vec!["Drama".to_owned()])),
        ..Default::default()
    };
    let season = season.insert(&db).await?;

    let episode = episode::ActiveModel {
        season_id: ActiveValue::Set(season.id),
        episode_number: ActiveValue::Set(1),
        title: ActiveValue::Set("Pilot".to_owned()),
        ..Default::default()
    };
    let _episode = episode.insert(&db).await?;
    println!("Inserted Show/Season/Episode");

    // 7. Insert StreamCache with JSONB (Postgres specific)
    let config_json = serde_json::json!({
        "codec": "h264",
        "bitrate": 5000,
        "audio_tracks": ["aac"]
    });

    let stream_cache = stream_cache::ActiveModel {
        file_id: ActiveValue::Set(indexed_file.id),
        stream_config: ActiveValue::Set(config_json.clone()),
        cache_path: ActiveValue::Set("/cache/stream_1".to_owned()),
        ..Default::default()
    };
    let stream_cache = stream_cache.insert(&db).await?;
    println!("Inserted StreamCache");

    // Verify JSONB
    assert_eq!(stream_cache.stream_config, config_json);

    // 8. Verify Constraints (Unique)
    // Try inserting duplicate IndexedFile (same library + path)
    let duplicate_file = indexed_file::ActiveModel {
        library_id: ActiveValue::Set(library.id),
        file_path: ActiveValue::Set("/tmp/test_library/movie.mp4".to_owned()),
        file_hash: ActiveValue::Set("otherhash".to_owned()),
        file_size: ActiveValue::Set(100),
        ..Default::default()
    };
    let result = duplicate_file.insert(&db).await;
    assert!(result.is_err(), "Duplicate file path should fail");

    println!("Schema verification successful!");
    Ok(())
}
