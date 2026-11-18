use reqwest::Client;
use serde_json::{json, Value};

use crate::models::{CastMember, ScheduleItem, SeasonInfo, UnifiedMedia};

const ONESHOWS_API: &str = "https://www.1shows.org/api";

#[derive(Clone)]
pub struct ApiClient {
    http: Client,
    #[allow(dead_code)]
    tmdb_api_key: Option<String>,
}

impl ApiClient {
    pub fn new(tmdb_api_key: Option<String>) -> Self {
        let http = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_default();
        Self { http, tmdb_api_key }
    }

    pub async fn get_trending_anime(&self, per_page: u32) -> Vec<UnifiedMedia> {
        self.discover_anime(None, None, None, "TRENDING_DESC", per_page)
            .await
    }

    pub async fn discover_anime(
        &self,
        search: Option<&str>,
        genre: Option<&str>,
        status: Option<&str>,
        sort: &str,
        per_page: u32,
    ) -> Vec<UnifiedMedia> {
        let query = r#"
        query ($perPage: Int, $search: String, $genre: String, $status: MediaStatus, $sort: [MediaSort]) {
          Page(page: 1, perPage: $perPage) {
            media(type: ANIME, isAdult: false, search: $search, genre: $genre, status: $status, sort: $sort) {
              id
              idMal
              title { romaji english native }
              coverImage { extraLarge large }
              bannerImage
              description(asHtml: false)
              averageScore
              seasonYear
              startDate { year }
              status
              genres
              episodes
              duration
              studios(isMain: true) { nodes { name } }
              trailer { id site }
              nextAiringEpisode { episode airingAt timeUntilAiring }
            }
          }
        }
        "#;

        let mut vars = serde_json::Map::new();
        vars.insert("perPage".into(), json!(per_page));
        vars.insert("sort".into(), json!([sort]));

        if let Some(s) = search.filter(|s| !s.trim().is_empty()) {
            vars.insert("search".into(), json!(s));
        }
        if let Some(g) = genre.filter(|g| !g.is_empty() && *g != "ALL") {
            vars.insert("genre".into(), json!(g));
        }
        if let Some(st) = status.filter(|st| !st.is_empty() && *st != "ALL") {
            vars.insert("status".into(), json!(st));
        }

        if let Ok(resp) = self
            .http
            .post("https://graphql.anilist.co")
            .json(&json!({ "query": query, "variables": vars }))
            .send()
            .await
        {
            if let Ok(body) = resp.json::<Value>().await {
                if let Some(arr) = body
                    .pointer("/data/Page/media")
                    .and_then(|v| v.as_array())
                {
                    let list: Vec<UnifiedMedia> =
                        arr.iter().map(parse_anilist_media).collect();
                    if !list.is_empty() {
                        return list;
                    }
                }
            }
        }

        let fallback = fallback_anime_list();
        if let Some(q) = search.filter(|s| !s.trim().is_empty()) {
            let q_low = q.to_lowercase();
            return fallback
                .into_iter()
                .filter(|m| m.title.to_lowercase().contains(&q_low))
                .collect();
        }
        fallback
    }

    pub async fn get_anime_by_id(&self, id: &str) -> UnifiedMedia {
        let id_num: i64 = id.parse().unwrap_or(154587);
        let query = r#"
        query ($id: Int) {
          Media(id: $id, type: ANIME) {
            id
            idMal
            title { romaji english native }
            coverImage { extraLarge large }
            bannerImage
            description(asHtml: false)
            averageScore
            seasonYear
            startDate { year }
            status
            genres
            episodes
            duration
            studios(isMain: true) { nodes { name } }
            trailer { id site }
            nextAiringEpisode { episode airingAt timeUntilAiring }
            characters(sort: [ROLE, RELEVANCE], perPage: 10) {
              edges {
                role
                node {
                  id
                  name { full }
                  image { large }
                }
              }
            }
            recommendations(sort: RATING_DESC, perPage: 8) {
              nodes {
                mediaRecommendation {
                  id
                  idMal
                  title { romaji english }
                  coverImage { extraLarge large }
                  bannerImage
                  description(asHtml: false)
                  averageScore
                  seasonYear
                  status
                  genres
                  episodes
                  duration
                }
              }
            }
          }
        }
        "#;

        if let Ok(resp) = self
            .http
            .post("https://graphql.anilist.co")
            .json(&json!({ "query": query, "variables": { "id": id_num } }))
            .send()
            .await
        {
            if let Ok(body) = resp.json::<Value>().await {
                if let Some(media_val) = body.pointer("/data/Media") {
                    if !media_val.is_null() {
                        return parse_anilist_media(media_val);
                    }
                }
            }
        }

        fallback_anime_list()
            .into_iter()
            .find(|a| a.id == id)
            .unwrap_or_else(|| fallback_anime_list().remove(0))
    }

    pub async fn get_weekly_schedule(&self) -> Vec<ScheduleItem> {
        let query = r#"
        query {
          Page(page: 1, perPage: 36) {
            media(type: ANIME, status: RELEASING, sort: POPULARITY_DESC, isAdult: false) {
              id
              idMal
              title { romaji english }
              coverImage { extraLarge large }
              bannerImage
              description(asHtml: false)
              averageScore
              seasonYear
              status
              genres
              episodes
              duration
              nextAiringEpisode { episode airingAt timeUntilAiring }
            }
          }
        }
        "#;

        if let Ok(resp) = self
            .http
            .post("https://graphql.anilist.co")
            .json(&json!({ "query": query }))
            .send()
            .await
        {
            if let Ok(body) = resp.json::<Value>().await {
                if let Some(arr) = body.pointer("/data/Page/media").and_then(|v| v.as_array()) {
                    let mut items = Vec::new();
                    for (idx, val) in arr.iter().enumerate() {
                        let media = parse_anilist_media(val);
                        let ep = val
                            .pointer("/nextAiringEpisode/episode")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1) as u32;
                        let secs = val
                            .pointer("/nextAiringEpisode/timeUntilAiring")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(7200);
                        let airing_at = val
                            .pointer("/nextAiringEpisode/airingAt")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);

                        let day_index = if airing_at > 0 {
                            (((airing_at / 86400) + 4) % 7) as u32
                        } else {
                            (idx % 7) as u32
                        };

                        let hour = if airing_at > 0 {
                            ((airing_at / 3600 + 7) % 24).abs()
                        } else {
                            19 + (idx as i64 % 5)
                        };

                        items.push(ScheduleItem {
                            id: format!("sched-{}-{}", media.id, idx),
                            episode: ep,
                            day_index,
                            time_wib: format!("{:02}:00 WIB", hour),
                            countdown_label: format_countdown(secs),
                            media,
                        });
                    }
                    if !items.is_empty() {
                        return items;
                    }
                }
            }
        }

        fallback_anime_list()
            .into_iter()
            .enumerate()
            .map(|(idx, media)| ScheduleItem {
                id: format!("sched-fb-{}", idx),
                episode: media.total_episodes.max(1),
                day_index: (idx % 7) as u32,
                time_wib: format!("{:02}:30 WIB", 18 + (idx % 5)),
                countdown_label: format!("{}j {}m lagi", (idx + 1) * 2, 30),
                media,
            })
            .collect()
    }

    /// Live Unified Search (Movies + TV Series + K-Drama from TMDB/1Shows + Anime from AniList)
    pub async fn search_all(&self, query: &str) -> Vec<UnifiedMedia> {
        let q = query.trim();
        if q.is_empty() {
            return vec![];
        }

        let tmdb_fut = self.search_tmdb_multi(q);
        let anime_fut = self.discover_anime(Some(q), None, None, "POPULARITY_DESC", 12);

        let (mut tmdb_items, anime_items) = tokio::join!(tmdb_fut, anime_fut);

        // Combine TMDB (Movies, TV, Drakor) first, then Anime
        let mut combined = Vec::with_capacity(tmdb_items.len() + anime_items.len());
        combined.append(&mut tmdb_items);
        for a in anime_items {
            if !combined.iter().any(|x| x.title.eq_ignore_ascii_case(&a.title)) {
                combined.push(a);
            }
        }
        combined
    }

    pub async fn search_tmdb_multi(&self, query: &str) -> Vec<UnifiedMedia> {
        let url = format!(
            "{}/search/query?query={}",
            ONESHOWS_API,
            urlencoding(query)
        );

        if let Ok(resp) = self
            .http
            .get(&url)
            .header("Referer", "https://www.1shows.org/")
            .header("Origin", "https://www.1shows.org")
            .send()
            .await
        {
            if let Ok(body) = resp.json::<Value>().await {
                if let Some(arr) = body.get("results").and_then(|v| v.as_array()) {
                    let parsed: Vec<UnifiedMedia> = arr
                        .iter()
                        .filter_map(|item| {
                            let mtype = item
                                .get("media_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or_else(|| {
                                    if item.get("title").is_some() {
                                        "movie"
                                    } else {
                                        "tv"
                                    }
                                });
                            if mtype != "movie" && mtype != "tv" {
                                return None;
                            }
                            // Skip items without poster & backdrop so UI stays clean
                            if item.get("poster_path").and_then(|p| p.as_str()).is_none()
                                && item.get("backdrop_path").and_then(|p| p.as_str()).is_none()
                            {
                                return None;
                            }
                            Some(parse_tmdb_json(item, mtype))
                        })
                        .collect();

                    if !parsed.is_empty() {
                        return parsed;
                    }
                }
            }
        }

        let q_low = query.to_lowercase();
        let mut fb = fallback_movies();
        fb.extend(fallback_tv_shows());
        fb.into_iter()
            .filter(|m| m.title.to_lowercase().contains(&q_low))
            .collect()
    }

    pub async fn discover_movies(
        &self,
        genre: Option<&str>,
        search: Option<&str>,
    ) -> Vec<UnifiedMedia> {
        if let Some(q) = search.filter(|s| !s.trim().is_empty()) {
            let all = self.search_tmdb_multi(q).await;
            let movies: Vec<UnifiedMedia> = all
                .into_iter()
                .filter(|m| m.media_type == "movie")
                .collect();
            if !movies.is_empty() {
                return movies;
            }
        }

        let genre_id = genre.and_then(genre_name_to_tmdb_id);
        let mut url = format!(
            "{}/search/filter?media_type=movie&sort_by=popularity.desc",
            ONESHOWS_API
        );
        if let Some(gid) = genre_id {
            url.push_str(&format!("&with_genres={}", gid));
        }

        if let Ok(resp) = self
            .http
            .get(&url)
            .header("Referer", "https://www.1shows.org/")
            .send()
            .await
        {
            if let Ok(body) = resp.json::<Value>().await {
                if let Some(arr) = body.get("results").and_then(|v| v.as_array()) {
                    let list: Vec<UnifiedMedia> = arr
                        .iter()
                        .filter(|i| i.get("poster_path").and_then(|p| p.as_str()).is_some())
                        .map(|i| parse_tmdb_json(i, "movie"))
                        .collect();
                    if !list.is_empty() {
                        return list;
                    }
                }
            }
        }

        fallback_movies()
    }

    pub async fn discover_tv(
        &self,
        genre: Option<&str>,
        search: Option<&str>,
    ) -> Vec<UnifiedMedia> {
        if let Some(q) = search.filter(|s| !s.trim().is_empty()) {
            let all = self.search_tmdb_multi(q).await;
            let shows: Vec<UnifiedMedia> =
                all.into_iter().filter(|m| m.media_type == "tv").collect();
            if !shows.is_empty() {
                return shows;
            }
        }

        let is_kdrama = genre
            .map(|g| g.eq_ignore_ascii_case("K-Drama") || g.eq_ignore_ascii_case("Drakor"))
            .unwrap_or(false);

        let mut url = format!(
            "{}/search/filter?media_type=tv&sort_by=popularity.desc",
            ONESHOWS_API
        );
        if is_kdrama {
            url.push_str("&with_original_language=ko");
        } else if let Some(gid) = genre.and_then(genre_name_to_tmdb_id) {
            url.push_str(&format!("&with_genres={}", gid));
        }

        if let Ok(resp) = self
            .http
            .get(&url)
            .header("Referer", "https://www.1shows.org/")
            .send()
            .await
        {
            if let Ok(body) = resp.json::<Value>().await {
                if let Some(arr) = body.get("results").and_then(|v| v.as_array()) {
                    let list: Vec<UnifiedMedia> = arr
                        .iter()
                        .filter(|i| i.get("poster_path").and_then(|p| p.as_str()).is_some())
                        .map(|i| parse_tmdb_json(i, "tv"))
                        .collect();
                    if !list.is_empty() {
                        return list;
                    }
                }
            }
        }

        fallback_tv_shows()
    }

    pub async fn get_movie_by_id(&self, id: &str) -> UnifiedMedia {
        let url = format!("{}/movie/{}", ONESHOWS_API, id);
        if let Ok(resp) = self
            .http
            .get(&url)
            .header("Referer", "https://www.1shows.org/")
            .send()
            .await
        {
            if let Ok(body) = resp.json::<Value>().await {
                if body.get("id").is_some() {
                    return parse_tmdb_json(&body, "movie");
                }
            }
        }

        let all = fallback_movies();
        let mut found = all
            .iter()
            .find(|m| m.id == id)
            .cloned()
            .unwrap_or_else(|| all[0].clone());
        found.id = id.to_string();
        found.recommendations = all.into_iter().filter(|m| m.id != id).collect();
        found
    }

    pub async fn get_tv_by_id(&self, id: &str) -> UnifiedMedia {
        let url = format!("{}/tv/{}", ONESHOWS_API, id);
        if let Ok(resp) = self
            .http
            .get(&url)
            .header("Referer", "https://www.1shows.org/")
            .send()
            .await
        {
            if let Ok(body) = resp.json::<Value>().await {
                if body.get("id").is_some() {
                    return parse_tmdb_json(&body, "tv");
                }
            }
        }

        let all = fallback_tv_shows();
        let mut found = all
            .iter()
            .find(|m| m.id == id)
            .cloned()
            .unwrap_or_else(|| all[0].clone());
        found.id = id.to_string();
        found.recommendations = all.into_iter().filter(|m| m.id != id).collect();
        found
    }
}

fn urlencoding(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                out.push(b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn genre_name_to_tmdb_id(name: &str) -> Option<u32> {
    match name {
        "Action" => Some(28),
        "Adventure" => Some(12),
        "Animation" => Some(16),
        "Comedy" => Some(35),
        "Crime" => Some(80),
        "Drama" => Some(18),
        "Fantasy" => Some(14),
        "Horror" => Some(27),
        "Mystery" => Some(9648),
        "Romance" => Some(10749),
        "Sci-Fi" => Some(878),
        "Thriller" => Some(53),
        _ => None,
    }
}

fn tmdb_genre_id_to_name(id: i64) -> &'static str {
    match id {
        28 => "Action",
        12 => "Adventure",
        16 => "Animation",
        35 => "Comedy",
        80 => "Crime",
        99 => "Documentary",
        18 => "Drama",
        10751 => "Family",
        14 => "Fantasy",
        36 => "History",
        27 => "Horror",
        9648 => "Mystery",
        10749 => "Romance",
        878 => "Sci-Fi",
        53 => "Thriller",
        10759 => "Action & Adventure",
        10765 => "Sci-Fi & Fantasy",
        _ => "Cinema",
    }
}

fn parse_tmdb_json(v: &Value, media_type: &str) -> UnifiedMedia {
    let id = v
        .get("id")
        .and_then(|x| x.as_i64())
        .unwrap_or(693134)
        .to_string();

    let title = v
        .get("title")
        .and_then(|x| x.as_str())
        .or_else(|| v.get("name").and_then(|x| x.as_str()))
        .unwrap_or("Untitled")
        .to_string();

    let original_title = v
        .get("original_title")
        .and_then(|x| x.as_str())
        .or_else(|| v.get("original_name").and_then(|x| x.as_str()))
        .unwrap_or(&title)
        .to_string();

    let poster_url = v
        .get("poster_path")
        .and_then(|x| x.as_str())
        .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p))
        .unwrap_or_else(|| {
            "https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?w=600".to_string()
        });

    let backdrop_url = v
        .get("backdrop_path")
        .and_then(|x| x.as_str())
        .map(|p| format!("https://image.tmdb.org/t/p/original{}", p))
        .unwrap_or_else(|| poster_url.clone());

    let synopsis = v
        .get("overview")
        .and_then(|x| x.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Sinopsis bahasa Indonesia / Inggris siap ditonton di player.")
        .to_string();

    let rating_num = v
        .get("vote_average")
        .and_then(|x| x.as_f64())
        .filter(|&r| r > 0.0)
        .unwrap_or(7.8);

    let date_str = v
        .get("release_date")
        .and_then(|x| x.as_str())
        .or_else(|| v.get("first_air_date").and_then(|x| x.as_str()))
        .unwrap_or("2024");
    let release_year = date_str.chars().take(4).collect::<String>();

    let mut genres: Vec<String> = v
        .get("genres")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|g| g.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if genres.is_empty() {
        if let Some(ids) = v.get("genre_ids").and_then(|x| x.as_array()) {
            genres = ids
                .iter()
                .filter_map(|id| id.as_i64().map(|n| tmdb_genre_id_to_name(n).to_string()))
                .take(3)
                .collect();
        }
    }
    if genres.is_empty() {
        genres.push(if media_type == "movie" {
            "Movie".into()
        } else {
            "TV Series".into()
        });
    }

    let total_episodes = v
        .get("number_of_episodes")
        .and_then(|x| x.as_u64())
        .map(|x| x as u32)
        .unwrap_or(if media_type == "movie" { 1 } else { 16 });

    let duration = v
        .get("runtime")
        .and_then(|x| x.as_u64())
        .map(|x| x as u32)
        .unwrap_or(if media_type == "movie" { 120 } else { 45 });

    let studio = v
        .pointer("/networks/0/name")
        .and_then(|x| x.as_str())
        .or_else(|| v.pointer("/production_companies/0/name").and_then(|x| x.as_str()))
        .unwrap_or("Global Cinema Network")
        .to_string();

    UnifiedMedia {
        id,
        mal_id: None,
        media_type: media_type.to_string(),
        title,
        original_title,
        poster_url,
        backdrop_url,
        synopsis,
        rating: format!("{:.1}", rating_num),
        release_year: if release_year.is_empty() {
            "2024".into()
        } else {
            release_year
        },
        status: "Completed".into(),
        genres,
        total_episodes,
        duration,
        studio,
        trailer_youtube_id: None,
        next_airing_label: None,
        cast: vec![],
        seasons: vec![],
        recommendations: vec![],
    }
}

fn strip_html(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut inside_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&amp;", "&")
        .trim()
        .to_string()
}

pub fn format_countdown(seconds: i64) -> String {
    if seconds <= 0 {
        return "Baru saja tayang".into();
    }
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;
    if days > 0 {
        format!("{}h {}j lagi", days, hours)
    } else if hours > 0 {
        format!("{}j {}m lagi", hours, mins)
    } else {
        format!("{}m lagi", mins)
    }
}

fn parse_anilist_media(v: &Value) -> UnifiedMedia {
    let id = v
        .get("id")
        .and_then(|x| x.as_i64())
        .unwrap_or(154587)
        .to_string();
    let mal_id = v
        .get("idMal")
        .and_then(|x| x.as_u64())
        .map(|x| x as u32);

    let title = v
        .pointer("/title/english")
        .and_then(|x| x.as_str())
        .or_else(|| v.pointer("/title/romaji").and_then(|x| x.as_str()))
        .unwrap_or("Untitled Anime")
        .to_string();

    let original_title = v
        .pointer("/title/romaji")
        .and_then(|x| x.as_str())
        .unwrap_or(&title)
        .to_string();

    let poster_url = v
        .pointer("/coverImage/extraLarge")
        .and_then(|x| x.as_str())
        .or_else(|| v.pointer("/coverImage/large").and_then(|x| x.as_str()))
        .unwrap_or("https://images.unsplash.com/photo-1578632767115-351597cf2477?w=600")
        .to_string();

    let backdrop_url = v
        .get("bannerImage")
        .and_then(|x| x.as_str())
        .unwrap_or(&poster_url)
        .to_string();

    let synopsis = v
        .get("description")
        .and_then(|x| x.as_str())
        .map(strip_html)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Sinopsis belum tersedia untuk judul ini.".into());

    let score = v
        .get("averageScore")
        .and_then(|x| x.as_f64())
        .map(|s| s / 10.0)
        .unwrap_or(8.4);

    let release_year = v
        .get("seasonYear")
        .and_then(|x| x.as_i64())
        .or_else(|| v.pointer("/startDate/year").and_then(|x| x.as_i64()))
        .unwrap_or(2025)
        .to_string();

    let status_raw = v.get("status").and_then(|x| x.as_str()).unwrap_or("");
    let status = if status_raw == "RELEASING" {
        "Ongoing".to_string()
    } else {
        "Completed".to_string()
    };

    let genres: Vec<String> = v
        .get("genres")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|g| g.as_str().map(|s| s.to_string()))
                .take(4)
                .collect()
        })
        .unwrap_or_else(|| vec!["Anime".into(), "Action".into()]);

    let next_ep = v
        .pointer("/nextAiringEpisode/episode")
        .and_then(|x| x.as_u64())
        .map(|x| x as u32);
    let next_secs = v
        .pointer("/nextAiringEpisode/timeUntilAiring")
        .and_then(|x| x.as_i64());

    let total_episodes = v
        .get("episodes")
        .and_then(|x| x.as_u64())
        .map(|x| x as u32)
        .or_else(|| next_ep.map(|e| e.saturating_sub(1).max(1)))
        .unwrap_or(12);

    let next_airing_label = match (next_ep, next_secs) {
        (Some(ep), Some(secs)) => Some(format!("Ep {} • {}", ep, format_countdown(secs))),
        _ => None,
    };

    let studio = v
        .pointer("/studios/nodes/0/name")
        .and_then(|x| x.as_str())
        .unwrap_or("Animation Studio")
        .to_string();

    let trailer_youtube_id = v.get("trailer").and_then(|t| {
        let site = t.get("site").and_then(|s| s.as_str()).unwrap_or("");
        if site.eq_ignore_ascii_case("youtube") {
            t.get("id").and_then(|id| id.as_str()).map(|s| s.to_string())
        } else {
            None
        }
    });

    let cast: Vec<CastMember> = v
        .pointer("/characters/edges")
        .and_then(|x| x.as_array())
        .map(|edges| {
            edges
                .iter()
                .filter_map(|edge| {
                    let node = edge.get("node")?;
                    Some(CastMember {
                        id: node
                            .get("id")
                            .and_then(|i| i.as_i64())
                            .unwrap_or(0)
                            .to_string(),
                        name: node
                            .pointer("/name/full")
                            .and_then(|n| n.as_str())
                            .unwrap_or("Character")
                            .to_string(),
                        role: edge
                            .get("role")
                            .and_then(|r| r.as_str())
                            .unwrap_or("MAIN")
                            .to_string(),
                        image_url: node
                            .pointer("/image/large")
                            .and_then(|img| img.as_str())
                            .unwrap_or("")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let recommendations: Vec<UnifiedMedia> = v
        .pointer("/recommendations/nodes")
        .and_then(|x| x.as_array())
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(|n| n.get("mediaRecommendation"))
                .filter(|m| !m.is_null())
                .map(parse_anilist_media)
                .collect()
        })
        .unwrap_or_default();

    UnifiedMedia {
        id,
        mal_id,
        media_type: "anime".into(),
        title,
        original_title,
        poster_url,
        backdrop_url,
        synopsis,
        rating: format!("{:.1}", score),
        release_year,
        status,
        genres,
        total_episodes,
        duration: v.get("duration").and_then(|x| x.as_u64()).unwrap_or(24) as u32,
        studio,
        trailer_youtube_id,
        next_airing_label,
        cast,
        seasons: vec![],
        recommendations,
    }
}

pub fn fallback_movies() -> Vec<UnifiedMedia> {
    vec![
        UnifiedMedia {
            id: "693134".into(),
            mal_id: None,
            media_type: "movie".into(),
            title: "Dune: Part Two".into(),
            original_title: "Dune: Part Two".into(),
            poster_url: "https://image.tmdb.org/t/p/w500/1pdfLvkbY9ohJlCjQH2CZjjYVvJ.jpg".into(),
            backdrop_url: "https://image.tmdb.org/t/p/original/xOMo8BRK7PfcJv9JCnx7s5hj0PX.jpg".into(),
            synopsis: "Paul Atreides bersatu dengan Chani dan bangsa Fremen sambil menuntut balas terhadap para konspirator yang menghancurkan keluarganya di planet gurun Arrakis.".into(),
            rating: "8.4".into(),
            release_year: "2024".into(),
            status: "Completed".into(),
            genres: vec!["Sci-Fi".into(), "Adventure".into(), "Action".into()],
            total_episodes: 1,
            duration: 166,
            studio: "Legendary Pictures".into(),
            trailer_youtube_id: Some("Way9Dexny3w".into()),
            next_airing_label: None,
            cast: vec![],
            seasons: vec![],
            recommendations: vec![],
        },
        UnifiedMedia {
            id: "872585".into(),
            mal_id: None,
            media_type: "movie".into(),
            title: "Oppenheimer".into(),
            original_title: "Oppenheimer".into(),
            poster_url: "https://image.tmdb.org/t/p/w500/8Gxv8gSFCU0XGDykEGv7zR1n2ua.jpg".into(),
            backdrop_url: "https://image.tmdb.org/t/p/original/fm6KqXpk3M2HVveHwCrBSSBaO0V.jpg".into(),
            synopsis: "Kisah fisikawan teoretis Amerika J. Robert Oppenheimer dan perannya dalam Proyek Manhattan yang mengembangkan senjata nuklir pertama di dunia.".into(),
            rating: "8.3".into(),
            release_year: "2023".into(),
            status: "Completed".into(),
            genres: vec!["Drama".into(), "History".into()],
            total_episodes: 1,
            duration: 181,
            studio: "Universal Pictures".into(),
            trailer_youtube_id: Some("uYPbbksJxIg".into()),
            next_airing_label: None,
            cast: vec![],
            seasons: vec![],
            recommendations: vec![],
        },
    ]
}

pub fn fallback_tv_shows() -> Vec<UnifiedMedia> {
    vec![
        UnifiedMedia {
            id: "94605".into(),
            mal_id: None,
            media_type: "tv".into(),
            title: "Arcane".into(),
            original_title: "Arcane: League of Legends".into(),
            poster_url: "https://image.tmdb.org/t/p/w500/fqldf2t8ztc9aiwn3k6mlX3tvRT.jpg".into(),
            backdrop_url: "https://image.tmdb.org/t/p/original/rkB4LyZHo1NHXFEDHl9vSD9r1lI.jpg".into(),
            synopsis: "Di tengah ketegangan tajam antara kota makmur Piltover dan daerah kumuh Zaun, dua saudari Vi dan Jinx bertarung di pihak berseberangan.".into(),
            rating: "8.9".into(),
            release_year: "2021".into(),
            status: "Completed".into(),
            genres: vec!["Animation".into(), "Sci-Fi".into(), "Action".into()],
            total_episodes: 18,
            duration: 42,
            studio: "Netflix / Fortiche".into(),
            trailer_youtube_id: None,
            next_airing_label: None,
            cast: vec![],
            seasons: vec![
                SeasonInfo { season_number: 1, name: "Season 1".into(), episode_count: 9 },
                SeasonInfo { season_number: 2, name: "Season 2".into(), episode_count: 9 },
            ],
            recommendations: vec![],
        },
    ]
}

pub fn fallback_anime_list() -> Vec<UnifiedMedia> {
    vec![
        UnifiedMedia {
            id: "154587".into(),
            mal_id: Some(52991),
            media_type: "anime".into(),
            title: "Frieren: Beyond Journey's End".into(),
            original_title: "Sousou no Frieren".into(),
            poster_url: "https://s4.anilist.co/file/anilistcdn/media/anime/cover/large/bx154587-gHSraOSa0nBG.jpg".into(),
            backdrop_url: "https://s4.anilist.co/file/anilistcdn/media/anime/banner/154587-ivXNJ23SM1xB.jpg".into(),
            synopsis: "Setelah mengalahkan Raja Iblis bersama kelompok pahlawan, penyihir elf Frieren memulai perjalanan baru untuk memahami arti kehidupan.".into(),
            rating: "9.1".into(),
            release_year: "2023".into(),
            status: "Completed".into(),
            genres: vec!["Adventure".into(), "Drama".into(), "Fantasy".into()],
            total_episodes: 28,
            duration: 24,
            studio: "MADHOUSE".into(),
            trailer_youtube_id: None,
            next_airing_label: None,
            cast: vec![],
            seasons: vec![],
            recommendations: vec![],
        },
    ]
}
