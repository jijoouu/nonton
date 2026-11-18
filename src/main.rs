mod api;
mod models;
mod providers;

use std::net::SocketAddr;
use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::{compression::CompressionLayer, services::ServeDir};

use crate::api::ApiClient;
use crate::models::{ScheduleItem, StreamServer, UnifiedMedia};
use crate::providers::{get_anime_servers, get_movie_servers, get_tv_servers};

#[derive(Clone)]
pub struct AppState {
    pub api: Arc<ApiClient>,
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    hero_items: Vec<UnifiedMedia>,
    trending_anime: Vec<UnifiedMedia>,
    trending_movies: Vec<UnifiedMedia>,
    trending_tv: Vec<UnifiedMedia>,
}

#[derive(Template)]
#[template(path = "catalog.html")]
struct CatalogTemplate {
    #[allow(dead_code)]
    category_kicker: &'static str,
    page_title: &'static str,
    page_subtitle: &'static str,
    base_path: &'static str,
    active_genre: String,
    genres: Vec<String>,
    items: Vec<UnifiedMedia>,
}

#[derive(Template)]
#[template(path = "schedule.html")]
struct ScheduleTemplate {
    active_day: u32,
    schedules: Vec<ScheduleItem>,
}

#[derive(Template)]
#[template(path = "watchlist.html")]
struct WatchlistTemplate {}

#[derive(Template)]
#[template(path = "detail.html")]
struct DetailTemplate {
    media: UnifiedMedia,
    episode_preview_limit: u32,
}

#[derive(Template)]
#[template(path = "watch.html")]
struct WatchTemplate {
    media: UnifiedMedia,
    servers: Vec<StreamServer>,
    current_season: u32,
    current_episode: u32,
    episode_preview_limit: u32,
    active_audio: String,
    prev_href: String,
    next_href: String,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchPageTemplate {
    query: String,
    active_type: String,
    total_count: usize,
    items: Vec<UnifiedMedia>,
}

#[derive(Deserialize)]
struct CatalogQuery {
    genre: Option<String>,
    q: Option<String>,
}

#[derive(Deserialize)]
struct SearchPageQuery {
    q: Option<String>,
    #[serde(rename = "type")]
    media_type: Option<String>,
}

#[derive(Deserialize)]
struct ScheduleQuery {
    day: Option<u32>,
}

#[derive(Deserialize)]
struct WatchAudioQuery {
    audio: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let tmdb_key = std::env::var("TMDB_API_KEY")
        .ok()
        .or_else(|| std::env::var("NEXT_PUBLIC_TMDB_API_KEY").ok());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let state = AppState {
        api: Arc::new(ApiClient::new(tmdb_key)),
    };

    let app = Router::new()
        .route("/", get(home_handler))
        .route("/search", get(search_page_handler))
        .route("/movies", get(movies_catalog_handler))
        .route("/tv", get(tv_catalog_handler))
        .route("/anime", get(anime_catalog_handler))
        .route("/schedule", get(schedule_handler))
        .route("/watchlist", get(watchlist_handler))
        .route("/movie/:id", get(movie_detail_handler))
        .route("/tv/:id", get(tv_detail_handler))
        .route("/anime/:id", get(anime_detail_handler))
        .route("/watch/movie/:id", get(watch_movie_handler))
        .route("/watch/tv/:id/:season/:ep", get(watch_tv_handler))
        .route("/watch/anime/:id/:ep", get(watch_anime_handler))
        .route("/api/search", get(search_api_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CompressionLayer::new())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Nonton Rust Engine berjalan super cepat di http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn home_handler(State(state): State<AppState>) -> HomeTemplate {
    let (trending_anime, trending_movies, trending_tv) = tokio::join!(
        state.api.get_trending_anime(12),
        state.api.discover_movies(None, None),
        state.api.discover_tv(None, None)
    );

    let mut hero_items = Vec::new();
    if let Some(a) = trending_anime.first() {
        hero_items.push(a.clone());
    }
    if let Some(m) = trending_movies.first() {
        hero_items.push(m.clone());
    }
    if let Some(t) = trending_tv.first() {
        hero_items.push(t.clone());
    }
    if let Some(a2) = trending_anime.get(1) {
        hero_items.push(a2.clone());
    }

    HomeTemplate {
        hero_items,
        trending_anime,
        trending_movies,
        trending_tv,
    }
}

async fn movies_catalog_handler(
    State(state): State<AppState>,
    Query(q): Query<CatalogQuery>,
) -> CatalogTemplate {
    let active_genre = q.genre.unwrap_or_else(|| "ALL".into());
    let items = state
        .api
        .discover_movies(Some(&active_genre), q.q.as_deref())
        .await;

    CatalogTemplate {
        category_kicker: "Movie Catalog • 1080p & 4K",
        page_title: "Katalog Film Box Office",
        page_subtitle: "Streaming film layar lebar kualitas HD dengan dukungan Multi-Server instan.",
        base_path: "/movies",
        active_genre,
        genres: vec!["ALL", "Action", "Sci-Fi", "Adventure", "Drama", "Animation", "Crime"]
            .into_iter()
            .map(String::from)
            .collect(),
        items,
    }
}

async fn tv_catalog_handler(
    State(state): State<AppState>,
    Query(q): Query<CatalogQuery>,
) -> CatalogTemplate {
    let active_genre = q.genre.unwrap_or_else(|| "ALL".into());
    let items = state
        .api
        .discover_tv(Some(&active_genre), q.q.as_deref())
        .await;

    CatalogTemplate {
        category_kicker: "TV Series & K-Drama Hub",
        page_title: "Katalog Serial TV & K-Drama",
        page_subtitle: "Maraton serial barat, Drama Korea (Drakor), dan Asia lengkap per episode.",
        base_path: "/tv",
        active_genre,
        genres: vec!["ALL", "K-Drama", "Drama", "Sci-Fi", "Action", "Crime", "Animation"]
            .into_iter()
            .map(String::from)
            .collect(),
        items,
    }
}

async fn anime_catalog_handler(
    State(state): State<AppState>,
    Query(q): Query<CatalogQuery>,
) -> CatalogTemplate {
    let active_genre = q.genre.unwrap_or_else(|| "ALL".into());
    let items = state
        .api
        .discover_anime(
            q.q.as_deref(),
            Some(&active_genre),
            None,
            "TRENDING_DESC",
            24,
        )
        .await;

    CatalogTemplate {
        category_kicker: "AniList GraphQL Live • Sub & Dub",
        page_title: "Katalog Anime Terlengkap",
        page_subtitle: "Ribuan judul Anime Ongoing & Completed langsung dari AniList dengan Sub/Dub Toggle.",
        base_path: "/anime",
        active_genre,
        genres: vec![
            "ALL",
            "Action",
            "Adventure",
            "Fantasy",
            "Comedy",
            "Romance",
            "Sci-Fi",
            "Drama",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        items,
    }
}

async fn schedule_handler(
    State(state): State<AppState>,
    Query(q): Query<ScheduleQuery>,
) -> ScheduleTemplate {
    let active_day = q.day.unwrap_or(1);
    let all = state.api.get_weekly_schedule().await;
    let filtered: Vec<ScheduleItem> = all
        .iter()
        .filter(|s| s.day_index == active_day)
        .cloned()
        .collect();

    ScheduleTemplate {
        active_day,
        schedules: if filtered.is_empty() { all } else { filtered },
    }
}

async fn watchlist_handler() -> WatchlistTemplate {
    WatchlistTemplate {}
}

async fn movie_detail_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> DetailTemplate {
    let media = state.api.get_movie_by_id(&id).await;
    DetailTemplate {
        media,
        episode_preview_limit: 1,
    }
}

async fn tv_detail_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> DetailTemplate {
    let media = state.api.get_tv_by_id(&id).await;
    let limit = media.total_episodes.min(200).max(1);
    DetailTemplate {
        media,
        episode_preview_limit: limit,
    }
}

async fn anime_detail_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> DetailTemplate {
    let media = state.api.get_anime_by_id(&id).await;
    let limit = media.total_episodes.min(200).max(1);
    DetailTemplate {
        media,
        episode_preview_limit: limit,
    }
}

async fn watch_movie_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> WatchTemplate {
    let media = state.api.get_movie_by_id(&id).await;
    let servers = get_movie_servers(&id);
    WatchTemplate {
        media,
        servers,
        current_season: 1,
        current_episode: 1,
        episode_preview_limit: 1,
        active_audio: "sub".into(),
        prev_href: "#".into(),
        next_href: "#".into(),
    }
}

async fn watch_tv_handler(
    State(state): State<AppState>,
    Path((id, season, ep)): Path<(String, u32, u32)>,
) -> WatchTemplate {
    let media = state.api.get_tv_by_id(&id).await;
    let servers = get_tv_servers(&id, season, ep);
    let limit = media.total_episodes.min(200).max(1);
    let prev_ep = ep.saturating_sub(1).max(1);
    let next_ep = ep + 1;

    WatchTemplate {
        media,
        servers,
        current_season: season,
        current_episode: ep,
        episode_preview_limit: limit,
        active_audio: "sub".into(),
        prev_href: format!("/watch/tv/{}/{}/{}", id, season, prev_ep),
        next_href: format!("/watch/tv/{}/{}/{}", id, season, next_ep),
    }
}

async fn watch_anime_handler(
    State(state): State<AppState>,
    Path((id, ep)): Path<(String, u32)>,
    Query(q): Query<WatchAudioQuery>,
) -> WatchTemplate {
    let active_audio = q
        .audio
        .filter(|a| a == "dub" || a == "sub")
        .unwrap_or_else(|| "sub".into());
    let media = state.api.get_anime_by_id(&id).await;
    let servers = get_anime_servers(&media.id, media.mal_id, ep, &active_audio);
    let limit = media.total_episodes.min(200).max(ep);
    let prev_ep = ep.saturating_sub(1).max(1);
    let next_ep = ep + 1;

    WatchTemplate {
        media,
        servers,
        current_season: 1,
        current_episode: ep,
        episode_preview_limit: limit,
        active_audio: active_audio.clone(),
        prev_href: format!("/watch/anime/{}/{}?audio={}", id, prev_ep, active_audio),
        next_href: format!("/watch/anime/{}/{}?audio={}", id, next_ep, active_audio),
    }
}

async fn search_page_handler(
    State(state): State<AppState>,
    Query(q): Query<SearchPageQuery>,
) -> SearchPageTemplate {
    let query_str = q.q.unwrap_or_default().trim().to_string();
    let active_type = q
        .media_type
        .filter(|t| t == "movie" || t == "tv" || t == "anime")
        .unwrap_or_else(|| "all".into());

    let all_results = if query_str.is_empty() {
        state.api.discover_movies(None, None).await
    } else {
        state.api.search_all(&query_str).await
    };

    let total_count = all_results.len();
    let items: Vec<UnifiedMedia> = if active_type == "all" {
        all_results
    } else {
        all_results
            .into_iter()
            .filter(|item| item.media_type == active_type)
            .collect()
    };

    SearchPageTemplate {
        query: query_str,
        active_type,
        total_count,
        items,
    }
}

async fn search_api_handler(
    State(state): State<AppState>,
    Query(q): Query<CatalogQuery>,
) -> Json<Value> {
    let query_str = q.q.unwrap_or_default();
    if query_str.trim().len() < 2 {
        return Json(json!({ "results": [] }));
    }

    let all = state.api.search_all(&query_str).await;
    let results: Vec<UnifiedMedia> = all.into_iter().take(12).collect();

    Json(json!({ "results": results }))
}
