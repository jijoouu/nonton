mod api;
mod models;
mod providers;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tower_http::{compression::CompressionLayer, services::ServeDir};

use crate::api::ApiClient;
use crate::models::{ScheduleItem, StreamServer, UnifiedMedia};
use crate::providers::{get_anime_servers, get_movie_servers, get_tv_servers};

#[derive(Clone, Serialize, Deserialize)]
pub struct PartyMessage {
    pub id: u64,
    pub sender: String,
    pub text: String,
    pub is_system: bool,
    pub kind: String, // "chat" | "system" | "reaction"
    pub time_label: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PartyMember {
    pub client_id: String,
    pub nickname: String,
    pub last_seen_ms: u64,
    pub is_host: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WatchPartyRoom {
    pub room_code: String,
    pub host_id: String,
    pub host_name: String,
    pub media_title: String,
    pub current_path: String,
    pub active_server_idx: usize,
    pub active_server_name: String,
    pub active_server_url: String,
    pub is_playing: bool,
    pub base_elapsed_sec: u64,
    pub last_timer_update_ms: u64,
    pub sync_signal_id: u64,
    pub sync_signal_type: String, // "none" | "play_countdown" | "pause" | "seek"
    pub sync_target_sec: u64,
    pub members: Vec<PartyMember>,
    pub messages: Vec<PartyMessage>,
    pub next_msg_id: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub api: Arc<ApiClient>,
    pub rooms: Arc<RwLock<HashMap<String, WatchPartyRoom>>>,
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
    current_season: u32,
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

#[derive(Deserialize)]
struct TvDetailQuery {
    season: Option<u32>,
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
        rooms: Arc::new(RwLock::new(HashMap::new())),
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
        .route("/api/party/join", post(party_join_handler))
        .route("/api/party/poll", get(party_poll_handler))
        .route("/api/party/action", post(party_action_handler))
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
        current_season: 1,
        episode_preview_limit: 1,
    }
}

async fn tv_detail_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<TvDetailQuery>,
) -> DetailTemplate {
    let mut media = state.api.get_tv_by_id(&id).await;
    let active_season = q.season.unwrap_or(1).max(1);
    let season_eps = media
        .seasons
        .iter()
        .find(|s| s.season_number == active_season)
        .map(|s| s.episode_count)
        .or_else(|| media.seasons.first().map(|s| s.episode_count))
        .unwrap_or(media.total_episodes)
        .max(1);

    for rel in &mut media.related_seasons {
        rel.is_current = rel.season_number == active_season;
        if rel.is_current {
            rel.relation_label = format!("SEASON {} (AKTIF)", rel.season_number);
        } else if rel.season_number < active_season {
            rel.relation_label = format!("SEASON {} (SEBELUMNYA)", rel.season_number);
        } else {
            rel.relation_label = format!("SEASON {} (SELANJUTNYA)", rel.season_number);
        }
    }

    DetailTemplate {
        media,
        current_season: active_season,
        episode_preview_limit: season_eps,
    }
}

async fn anime_detail_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> DetailTemplate {
    let media = state.api.get_anime_by_id(&id).await;
    let limit = media.total_episodes.max(1);
    DetailTemplate {
        media,
        current_season: 1,
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
    let mut media = state.api.get_tv_by_id(&id).await;
    let servers = get_tv_servers(&id, season, ep);
    let season_eps = media
        .seasons
        .iter()
        .find(|s| s.season_number == season)
        .map(|s| s.episode_count)
        .or_else(|| media.seasons.first().map(|s| s.episode_count))
        .unwrap_or(media.total_episodes)
        .max(ep);

    for rel in &mut media.related_seasons {
        rel.is_current = rel.season_number == season;
        if rel.is_current {
            rel.relation_label = format!("SEASON {} (SEDANG DITONTON)", rel.season_number);
        } else if rel.season_number < season {
            rel.relation_label = format!("SEASON {} (SEBELUMNYA)", rel.season_number);
        } else {
            rel.relation_label = format!("SEASON {} (SELANJUTNYA)", rel.season_number);
        }
    }

    let prev_ep = ep.saturating_sub(1).max(1);
    let next_ep = ep + 1;

    WatchTemplate {
        media,
        servers,
        current_season: season,
        current_episode: ep,
        episode_preview_limit: season_eps,
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
    let (tmdb_id, tmdb_season, is_movie) = state.api.resolve_anime_tmdb(&media).await;
    let servers = get_anime_servers(&tmdb_id, tmdb_season, is_movie, media.mal_id, ep, &active_audio);
    let limit = media.total_episodes.max(ep);
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

/* =========================================================
   ZERO-LOGIN WATCH PARTY (NONTON BARENG) ENGINE
   ========================================================= */

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn now_time_label() -> String {
    let secs = (now_epoch_ms() / 1000) + 8 * 3600; // WITA / UTC+8
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    format!("{:02}:{:02}", h, m)
}

fn compute_live_elapsed(room: &WatchPartyRoom, now_ms: u64) -> u64 {
    if !room.is_playing {
        return room.base_elapsed_sec;
    }
    if now_ms > room.last_timer_update_ms {
        room.base_elapsed_sec + (now_ms - room.last_timer_update_ms) / 1000
    } else {
        room.base_elapsed_sec
    }
}

fn push_room_msg(room: &mut WatchPartyRoom, sender: &str, text: String, is_system: bool, kind: &str) {
    let id = room.next_msg_id;
    room.next_msg_id += 1;
    room.messages.push(PartyMessage {
        id,
        sender: sender.to_string(),
        text,
        is_system,
        kind: kind.to_string(),
        time_label: now_time_label(),
    });
    if room.messages.len() > 80 {
        let excess = room.messages.len() - 80;
        room.messages.drain(0..excess);
    }
}

#[derive(Deserialize)]
struct PartyJoinReq {
    room_code: String,
    client_id: String,
    nickname: String,
    media_title: String,
    current_path: String,
    server_idx: usize,
    server_name: String,
    server_url: String,
}

#[derive(Deserialize)]
struct PartyPollQuery {
    room: String,
    client_id: String,
    nickname: Option<String>,
}

#[derive(Deserialize)]
struct PartyActionReq {
    room_code: String,
    client_id: String,
    nickname: String,
    action: String, // "chat" | "reaction" | "switch_server" | "navigate" | "sync_play" | "sync_pause" | "sync_time"
    text: Option<String>,
    server_idx: Option<usize>,
    server_name: Option<String>,
    server_url: Option<String>,
    target_path: Option<String>,
    target_sec: Option<u64>,
}

fn build_room_snapshot(room: &WatchPartyRoom, now_ms: u64) -> Value {
    let live_sec = compute_live_elapsed(room, now_ms);
    json!({
        "ok": true,
        "room_code": room.room_code,
        "host_id": room.host_id,
        "host_name": room.host_name,
        "media_title": room.media_title,
        "current_path": room.current_path,
        "active_server_idx": room.active_server_idx,
        "active_server_name": room.active_server_name,
        "active_server_url": room.active_server_url,
        "is_playing": room.is_playing,
        "live_elapsed_sec": live_sec,
        "sync_signal_id": room.sync_signal_id,
        "sync_signal_type": room.sync_signal_type,
        "sync_target_sec": room.sync_target_sec,
        "members": room.members,
        "messages": room.messages,
    })
}

async fn party_join_handler(
    State(state): State<AppState>,
    Json(req): Json<PartyJoinReq>,
) -> Json<Value> {
    let code = req.room_code.trim().to_uppercase();
    if code.is_empty() {
        return Json(json!({ "ok": false, "error": "Invalid room code" }));
    }
    let nick = if req.nickname.trim().is_empty() {
        "Guest".to_string()
    } else {
        req.nickname.trim().chars().take(24).collect()
    };
    let now_ms = now_epoch_ms();

    let mut rooms = state.rooms.write().await;
    let room = rooms.entry(code.clone()).or_insert_with(|| {
        let mut r = WatchPartyRoom {
            room_code: code.clone(),
            host_id: req.client_id.clone(),
            host_name: nick.clone(),
            media_title: req.media_title.clone(),
            current_path: req.current_path.clone(),
            active_server_idx: req.server_idx,
            active_server_name: req.server_name.clone(),
            active_server_url: req.server_url.clone(),
            is_playing: false,
            base_elapsed_sec: 0,
            last_timer_update_ms: now_ms,
            sync_signal_id: 1,
            sync_signal_type: "none".into(),
            sync_target_sec: 0,
            members: vec![],
            messages: vec![],
            next_msg_id: 1,
        };
        push_room_msg(
            &mut r,
            "SYSTEM",
            format!("Room {} dibuat oleh {} (Host).", code, nick),
            true,
            "system",
        );
        r
    });

    // Prune inactive members (> 25s)
    room.members
        .retain(|m| now_ms.saturating_sub(m.last_seen_ms) < 25_000);

    // If previous host left and room has no active host, promote current user if needed
    if !room.members.iter().any(|m| m.client_id == room.host_id) && room.host_id != req.client_id {
        if room.members.is_empty() {
            room.host_id = req.client_id.clone();
            room.host_name = nick.clone();
        }
    }

    let is_host = room.host_id == req.client_id;
    if is_host {
        room.host_name = nick.clone();
        if !req.current_path.is_empty() && room.current_path != req.current_path {
            room.current_path = req.current_path.clone();
            room.media_title = req.media_title.clone();
            room.active_server_idx = req.server_idx;
            room.active_server_name = req.server_name.clone();
            room.active_server_url = req.server_url.clone();
            room.base_elapsed_sec = 0;
            room.last_timer_update_ms = now_ms;
            push_room_msg(
                &mut *room,
                "SYSTEM",
                format!("Host {} berpindah ke episode/judul baru: {}", nick, req.media_title),
                true,
                "system",
            );
        }
    }

    if let Some(existing) = room.members.iter_mut().find(|m| m.client_id == req.client_id) {
        existing.nickname = nick.clone();
        existing.last_seen_ms = now_ms;
        existing.is_host = is_host;
    } else {
        room.members.push(PartyMember {
            client_id: req.client_id.clone(),
            nickname: nick.clone(),
            last_seen_ms: now_ms,
            is_host,
        });
        if room.next_msg_id > 2 {
            push_room_msg(
                &mut *room,
                "SYSTEM",
                format!("{} bergabung ke Watch Party.", nick),
                true,
                "system",
            );
        }
    }

    Json(build_room_snapshot(room, now_ms))
}

async fn party_poll_handler(
    State(state): State<AppState>,
    Query(q): Query<PartyPollQuery>,
) -> Json<Value> {
    let code = q.room.trim().to_uppercase();
    let now_ms = now_epoch_ms();
    let mut rooms = state.rooms.write().await;

    let Some(room) = rooms.get_mut(&code) else {
        return Json(json!({ "ok": false, "error": "Room not found" }));
    };

    room.members
        .retain(|m| now_ms.saturating_sub(m.last_seen_ms) < 25_000);

    let is_host = room.host_id == q.client_id;
    if let Some(m) = room.members.iter_mut().find(|m| m.client_id == q.client_id) {
        m.last_seen_ms = now_ms;
        m.is_host = is_host;
        if let Some(nick) = q.nickname.as_ref().filter(|n| !n.trim().is_empty()) {
            m.nickname = nick.trim().chars().take(24).collect();
        }
    } else if let Some(nick) = q.nickname.filter(|n| !n.trim().is_empty()) {
        room.members.push(PartyMember {
            client_id: q.client_id.clone(),
            nickname: nick.trim().chars().take(24).collect(),
            last_seen_ms: now_ms,
            is_host,
        });
    }

    Json(build_room_snapshot(room, now_ms))
}

async fn party_action_handler(
    State(state): State<AppState>,
    Json(req): Json<PartyActionReq>,
) -> Json<Value> {
    let code = req.room_code.trim().to_uppercase();
    let now_ms = now_epoch_ms();
    let mut rooms = state.rooms.write().await;

    let Some(room) = rooms.get_mut(&code) else {
        return Json(json!({ "ok": false, "error": "Room not found" }));
    };

    let nick = if req.nickname.trim().is_empty() {
        "Guest".to_string()
    } else {
        req.nickname.trim().chars().take(24).collect()
    };

    if let Some(m) = room.members.iter_mut().find(|m| m.client_id == req.client_id) {
        m.last_seen_ms = now_ms;
        m.nickname = nick.clone();
    }

    match req.action.as_str() {
        "chat" => {
            if let Some(txt) = req.text.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()) {
                let clean: String = txt.chars().take(280).collect();
                push_room_msg(room, &nick, clean, false, "chat");
            }
        }
        "reaction" => {
            if let Some(label) = req.text.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()) {
                push_room_msg(room, &nick, label, false, "reaction");
            }
        }
        "switch_server" => {
            if let (Some(idx), Some(url)) = (req.server_idx, req.server_url) {
                let sname = req.server_name.unwrap_or_else(|| format!("Server {}", idx + 1));
                room.active_server_idx = idx;
                room.active_server_name = sname.clone();
                room.active_server_url = url;
                push_room_msg(
                    room,
                    "SYSTEM",
                    format!("{} mengganti server ke {}", nick, sname),
                    true,
                    "system",
                );
            }
        }
        "navigate" => {
            if let Some(path) = req.target_path.filter(|p| !p.trim().is_empty()) {
                room.current_path = path;
                room.base_elapsed_sec = 0;
                room.last_timer_update_ms = now_ms;
            }
        }
        "sync_play" => {
            let cur_sec = req.target_sec.unwrap_or_else(|| compute_live_elapsed(room, now_ms));
            room.base_elapsed_sec = cur_sec;
            // Countdown takes 3 seconds before timer starts ticking
            room.last_timer_update_ms = now_ms + 3000;
            room.is_playing = true;
            room.sync_signal_id += 1;
            room.sync_signal_type = "play_countdown".into();
            room.sync_target_sec = cur_sec;
            let mm = cur_sec / 60;
            let ss = cur_sec % 60;
            push_room_msg(
                room,
                "SYSTEM",
                format!("{} memulai aba-aba SYNC PLAY (3.. 2.. 1.. PLAY di {:02}:{:02})", nick, mm, ss),
                true,
                "system",
            );
        }
        "sync_pause" => {
            let cur_sec = compute_live_elapsed(room, now_ms);
            room.base_elapsed_sec = cur_sec;
            room.last_timer_update_ms = now_ms;
            room.is_playing = false;
            room.sync_signal_id += 1;
            room.sync_signal_type = "pause".into();
            room.sync_target_sec = cur_sec;
            let mm = cur_sec / 60;
            let ss = cur_sec % 60;
            push_room_msg(
                room,
                "SYSTEM",
                format!("{} mengirim aba-aba PAUSE di menit {:02}:{:02}", nick, mm, ss),
                true,
                "system",
            );
        }
        "sync_time" => {
            if let Some(sec) = req.target_sec {
                room.base_elapsed_sec = sec;
                room.last_timer_update_ms = now_ms;
                room.sync_signal_id += 1;
                room.sync_signal_type = "seek".into();
                room.sync_target_sec = sec;
                let mm = sec / 60;
                let ss = sec % 60;
                push_room_msg(
                    room,
                    "SYSTEM",
                    format!("{} menyamakan posisi waktu ke {:02}:{:02}", nick, mm, ss),
                    true,
                    "system",
                );
            }
        }
        _ => {}
    }

    Json(build_room_snapshot(room, now_ms))
}
