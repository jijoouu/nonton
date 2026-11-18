use reqwest::Client;
use serde_json::{json, Value};

use crate::models::{CastMember, RelatedSeasonItem, ScheduleItem, SeasonInfo, UnifiedMedia};

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
            relations {
              edges {
                relationType(version: 2)
                node {
                  id
                  idMal
                  type
                  format
                  title { romaji english }
                  coverImage { extraLarge large }
                  averageScore
                  seasonYear
                  startDate { year }
                  episodes
                  relations {
                    edges {
                      relationType(version: 2)
                      node {
                        id
                        idMal
                        type
                        format
                        title { romaji english }
                        coverImage { extraLarge large }
                        averageScore
                        seasonYear
                        startDate { year }
                        episodes
                      }
                    }
                  }
                }
              }
            }
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
            recommendations(sort: RATING_DESC, perPage: 10) {
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
                        let mut media = parse_anilist_media(media_val);

                        // If this anime belongs to a multi-season franchise (like JoJo's Bizarre Adventure),
                        // also fetch any other seasons matching the franchise root name so ALL seasons appear in order!
                        if let Some(root_prefix) = extract_franchise_root(&media.title)
                            .or_else(|| extract_franchise_root(&media.original_title))
                        {
                            let franchise_list = self
                                .discover_anime(Some(&root_prefix), None, None, "START_DATE", 18)
                                .await;
                            let root_low = root_prefix.to_lowercase();
                            for item in franchise_list {
                                let t_low = item.title.to_lowercase();
                                let o_low = item.original_title.to_lowercase();
                                if (t_low.contains(&root_low) || o_low.contains(&root_low))
                                    && !media.related_seasons.iter().any(|r| r.id == item.id)
                                {
                                    let is_curr = item.id == media.id;
                                    let cur_year: i32 = media.release_year.parse().unwrap_or(2020);
                                    let item_year: i32 = item.release_year.parse().unwrap_or(2020);
                                    let rel_label = if is_curr {
                                        "CURRENT".to_string()
                                    } else if item.total_episodes == 1 {
                                        "MOVIE / OVA".to_string()
                                    } else if item_year < cur_year {
                                        "PREVIOUS SEASON".to_string()
                                    } else if item_year > cur_year {
                                        "NEXT SEASON".to_string()
                                    } else {
                                        "SERIES".to_string()
                                    };
                                    media.related_seasons.push(RelatedSeasonItem {
                                        id: item.id,
                                        season_number: 1,
                                        media_type: "anime".into(),
                                        title: item.title,
                                        poster_url: item.poster_url,
                                        release_year: item.release_year,
                                        total_episodes: item.total_episodes,
                                        rating: item.rating,
                                        relation_label: rel_label,
                                        is_current: is_curr,
                                    });
                                }
                            }
                            media.related_seasons.sort_by(|a, b| {
                                let ya: i32 = a.release_year.parse().unwrap_or(9999);
                                let yb: i32 = b.release_year.parse().unwrap_or(9999);
                                ya.cmp(&yb).then_with(|| {
                                    let ia: i64 = a.id.parse().unwrap_or(0);
                                    let ib: i64 = b.id.parse().unwrap_or(0);
                                    ia.cmp(&ib)
                                })
                            });
                        }

                        return media;
                    }
                }
            }
        }

        fallback_anime_list()
            .into_iter()
            .find(|a| a.id == id)
            .unwrap_or_else(|| fallback_anime_list().remove(0))
    }

    /// Resolve an AniList Anime into its corresponding TMDB (tmdb_id, season_number, is_movie)
    /// so anime can stream on the exact same high-speed 4K servers (VidFast, VidLink TV, Vidy, VidZee) as Movies & K-Drama.
    pub async fn resolve_anime_tmdb(&self, media: &UnifiedMedia) -> (String, u32, bool) {
        let known = match media.id.as_str() {
            "21" => Some(("37854", 1, false)),       // One Piece
            "14719" => Some(("45790", 1, false)),    // JoJo's Bizarre Adventure (2012) S1
            "20474" => Some(("45790", 2, false)),    // JoJo's Bizarre Adventure: Stardust Crusaders
            "20799" => Some(("45790", 2, false)),    // JoJo's Bizarre Adventure: Stardust Crusaders - Battle in Egypt
            "21450" => Some(("45790", 3, false)),    // JoJo's Bizarre Adventure: Diamond is Unbreakable
            "102883" => Some(("45790", 4, false)),   // JoJo's Bizarre Adventure: Golden Wind
            "131942" | "146722" | "153359" => Some(("45790", 5, false)), // JoJo's Bizarre Adventure: Stone Ocean
            "151807" => Some(("127532", 1, false)),  // Solo Leveling S1
            "176496" => Some(("127532", 2, false)),  // Solo Leveling S2
            "154587" => Some(("209867", 1, false)),  // Frieren: Beyond Journey's End
            "171018" => Some(("240411", 1, false)),  // DAN DA DAN
            "185660" => Some(("240411", 2, false)),  // DAN DA DAN S2
            "113415" => Some(("95479", 1, false)),   // Jujutsu Kaisen S1
            "145064" => Some(("95479", 2, false)),   // Jujutsu Kaisen S2
            "101922" => Some(("85937", 1, false)),   // Demon Slayer S1
            "142329" => Some(("85937", 2, false)),   // Demon Slayer S2
            "145139" => Some(("85937", 3, false)),   // Demon Slayer S3
            "166240" => Some(("85937", 4, false)),   // Demon Slayer S4
            "161645" => Some(("220542", 1, false)),  // The Apothecary Diaries S1
            "176301" => Some(("220542", 2, false)),  // The Apothecary Diaries S2
            "177709" => Some(("244479", 1, false)),  // SAKAMOTO DAYS
            "170083" => Some(("240413", 1, false)),  // Dragon Ball DAIMA
            "137822" => Some(("131041", 1, false)),  // BLUELOCK S1
            "163146" => Some(("131041", 2, false)),  // BLUELOCK S2
            "153288" => Some(("207468", 1, false)),  // Kaiju No. 8
            "127230" => Some(("114410", 1, false)),  // Chainsaw Man
            "16498" => Some(("1429", 1, false)),     // Attack on Titan
            "20" => Some(("46260", 1, false)),       // Naruto
            "1735" => Some(("31910", 1, false)),     // Naruto Shippuden
            "269" => Some(("30984", 1, false)),      // Bleach
            "116674" | "159322" | "170943" => Some(("30984", 2, false)), // Bleach TYBW
            "140960" => Some(("120089", 1, false)),  // SPY x FAMILY
            "151970" => Some(("206469", 1, false)),  // Shangri-La Frontier S1
            "169559" => Some(("206469", 2, false)),  // Shangri-La Frontier S2
            "150672" => Some(("203737", 1, false)),  // Oshi no Ko S1
            "166531" => Some(("203737", 2, false)),  // Oshi no Ko S2
            "151801" => Some(("204832", 1, false)),  // MASHLE S1
            "166610" => Some(("204832", 2, false)),  // MASHLE S2
            "21459" => Some(("65930", 1, false)),    // My Hero Academia
            "11061" => Some(("46298", 1, false)),    // Hunter x Hunter (2011)
            "1535" => Some(("13916", 1, false)),     // Death Note
            "97940" => Some(("73223", 1, false)),    // Black Clover
            "21355" => Some(("65942", 1, false)),    // Re:ZERO S1
            "163134" => Some(("65942", 3, false)),   // Re:ZERO S3
            "170942" => Some(("239770", 1, false)),  // Blue Box (Ao no Hako)
            "163270" => Some(("223500", 1, false)),  // WIND BREAKER
            "105333" => Some(("86031", 1, false)),   // Dr. STONE
            "172019" => Some(("86031", 4, false)),   // Dr. STONE Science Future
            _ => None,
        };

        if let Some((tid, season, is_movie)) = known {
            return (tid.to_string(), season, is_movie);
        }

        let detected_season = detect_anime_season(&media.title)
            .or_else(|| detect_anime_season(&media.original_title))
            .unwrap_or(1);

        let is_likely_movie = media.total_episodes <= 1 && media.duration >= 55;

        let mut candidates = vec![
            clean_anime_title(&media.title),
            clean_anime_title(&media.original_title),
        ];
        if let Some((base, _)) = media.title.split_once(':') {
            candidates.push(clean_anime_title(base));
        }
        if let Some((base, _)) = media.original_title.split_once(':') {
            candidates.push(clean_anime_title(base));
        }

        for q in candidates {
            if q.trim().len() < 2 {
                continue;
            }
            let results = self.search_tmdb_multi(&q).await;
            if results.is_empty() {
                continue;
            }
            let target_type = if is_likely_movie { "movie" } else { "tv" };
            if let Some(matched) = results
                .iter()
                .find(|r| r.media_type == target_type)
                .or_else(|| results.first())
            {
                return (
                    matched.id.clone(),
                    detected_season,
                    matched.media_type == "movie",
                );
            }
        }

        // Default fallback to Solo Leveling TMDB ID if completely offline
        ("127532".to_string(), 1, false)
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

        // Combine Anime (with per-season entries) and TMDB (Movies, TV, Drakor)
        let mut combined = Vec::with_capacity(tmdb_items.len() + anime_items.len());
        for a in anime_items {
            combined.push(a);
        }
        combined.append(&mut tmdb_items);
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
                    let mut m = parse_tmdb_json(&body, "movie");
                    if m.recommendations.is_empty() {
                        let trending = self.discover_movies(m.genres.first().map(|s| s.as_str()), None).await;
                        m.recommendations = trending.into_iter().filter(|x| x.id != m.id).take(8).collect();
                    }
                    return m;
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
                    let mut tv = parse_tmdb_json(&body, "tv");
                    if tv.recommendations.is_empty() {
                        let trending = self.discover_tv(tv.genres.first().map(|s| s.as_str()), None).await;
                        tv.recommendations = trending.into_iter().filter(|x| x.id != tv.id).take(8).collect();
                    }
                    return tv;
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

fn detect_anime_season(title: &str) -> Option<u32> {
    let low = title.to_lowercase();
    if low.contains("season 4") || low.contains("4th season") {
        Some(4)
    } else if low.contains("season 3") || low.contains("3rd season") {
        Some(3)
    } else if low.contains("season 2") || low.contains("2nd season") || low.contains("part 2") {
        Some(2)
    } else {
        None
    }
}

fn clean_anime_title(title: &str) -> String {
    let mut s = title.to_string();
    for pat in [
        "Season 2",
        "Season 3",
        "Season 4",
        "2nd Season",
        "3rd Season",
        "4th Season",
        "Part 2",
        "-Arise from the Shadow-",
    ] {
        s = s.replace(pat, "");
    }
    if let Some((left, _)) = s.split_once(" -") {
        s = left.to_string();
    }
    s.trim().to_string()
}

fn extract_franchise_root(title: &str) -> Option<String> {
    let base = title
        .split([':', '–', '—'])
        .next()
        .unwrap_or(title)
        .trim();
    let cleaned = clean_anime_title(base);
    if cleaned.len() >= 4 {
        Some(cleaned)
    } else {
        None
    }
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

    let mut related_seasons: Vec<RelatedSeasonItem> = Vec::new();
    let seasons: Vec<SeasonInfo> = v
        .get("seasons")
        .and_then(|x| x.as_array())
        .map(|arr| {
            let valid_seasons: Vec<&Value> = arr
                .iter()
                .filter(|s| {
                    let snum = s.get("season_number").and_then(|n| n.as_u64()).unwrap_or(0);
                    let ep_cnt = s.get("episode_count").and_then(|e| e.as_u64()).unwrap_or(0);
                    snum > 0 && ep_cnt > 0
                })
                .collect();

            let has_multiple = valid_seasons.len() > 1;
            let mut out = Vec::new();

            for s in valid_seasons {
                let snum = s.get("season_number").and_then(|n| n.as_u64()).unwrap_or(1) as u32;
                let ep_cnt = s.get("episode_count").and_then(|e| e.as_u64()).unwrap_or(12) as u32;
                let name = s
                    .get("name")
                    .and_then(|n| n.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| format!("Season {}", snum));

                if has_multiple {
                    let s_poster = s
                        .get("poster_path")
                        .and_then(|p| p.as_str())
                        .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p))
                        .unwrap_or_else(|| poster_url.clone());
                    let s_year = s
                        .get("air_date")
                        .and_then(|d| d.as_str())
                        .map(|d| d.chars().take(4).collect::<String>())
                        .filter(|y| !y.is_empty())
                        .unwrap_or_else(|| release_year.clone());
                    let display_title = if name.to_lowercase().starts_with("season") {
                        format!("{} — {}", title, name)
                    } else {
                        format!("S{}: {}", snum, name)
                    };

                    related_seasons.push(RelatedSeasonItem {
                        id: id.clone(),
                        season_number: snum,
                        media_type: "tv".into(),
                        title: display_title,
                        poster_url: s_poster,
                        release_year: s_year,
                        total_episodes: ep_cnt,
                        rating: format!("{:.1}", rating_num),
                        relation_label: format!("SEASON {}", snum),
                        is_current: snum == 1,
                    });
                }

                out.push(SeasonInfo {
                    season_number: snum,
                    name,
                    episode_count: ep_cnt,
                });
            }
            out
        })
        .unwrap_or_default();

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
        seasons,
        related_seasons,
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

    // Parse 2-level AniList relations (PREQUEL, SEQUEL, SIDE_STORY, SPIN_OFF, PARENT, ALTERNATIVE)
    let mut related_seasons: Vec<RelatedSeasonItem> = Vec::new();
    if let Some(edges) = v.pointer("/relations/edges").and_then(|x| x.as_array()) {
        for edge in edges {
            let rel_type = edge
                .get("relationType")
                .and_then(|r| r.as_str())
                .unwrap_or("RELATED");
            if let Some(node) = edge.get("node") {
                push_anilist_relation_node(&mut related_seasons, node, rel_type, &id);
                // Also inspect 2nd-hop relations (e.g. Prequel's Prequel or Sequel's Sequel)
                if let Some(sub_edges) = node.pointer("/relations/edges").and_then(|x| x.as_array()) {
                    for sub_edge in sub_edges {
                        let sub_rel = sub_edge
                            .get("relationType")
                            .and_then(|r| r.as_str())
                            .unwrap_or("SERIES");
                        if sub_rel == "PREQUEL" || sub_rel == "SEQUEL" || sub_rel == "PARENT" {
                            if let Some(sub_node) = sub_edge.get("node") {
                                push_anilist_relation_node(
                                    &mut related_seasons,
                                    sub_node,
                                    "SERIES",
                                    &id,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    if !related_seasons.is_empty() && !related_seasons.iter().any(|r| r.id == id) {
        related_seasons.push(RelatedSeasonItem {
            id: id.clone(),
            season_number: 1,
            media_type: "anime".into(),
            title: title.clone(),
            poster_url: poster_url.clone(),
            release_year: release_year.clone(),
            total_episodes,
            rating: format!("{:.1}", score),
            relation_label: "CURRENT".into(),
            is_current: true,
        });
    }

    related_seasons.sort_by(|a, b| {
        let ya: i32 = a.release_year.parse().unwrap_or(9999);
        let yb: i32 = b.release_year.parse().unwrap_or(9999);
        ya.cmp(&yb).then_with(|| {
            let ia: i64 = a.id.parse().unwrap_or(0);
            let ib: i64 = b.id.parse().unwrap_or(0);
            ia.cmp(&ib)
        })
    });

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
        related_seasons,
        recommendations,
    }
}

fn push_anilist_relation_node(
    list: &mut Vec<RelatedSeasonItem>,
    node: &Value,
    rel_type: &str,
    current_id: &str,
) {
    let mtype = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    if mtype != "ANIME" {
        return;
    }
    let format = node.get("format").and_then(|f| f.as_str()).unwrap_or("TV");
    if format == "MUSIC" {
        return;
    }
    let nid = match node.get("id").and_then(|i| i.as_i64()) {
        Some(n) => n.to_string(),
        None => return,
    };
    if list.iter().any(|x| x.id == nid) {
        return;
    }
    let is_curr = nid == current_id;
    let title = node
        .pointer("/title/english")
        .and_then(|x| x.as_str())
        .or_else(|| node.pointer("/title/romaji").and_then(|x| x.as_str()))
        .unwrap_or("Related Season")
        .to_string();
    let poster_url = node
        .pointer("/coverImage/extraLarge")
        .and_then(|x| x.as_str())
        .or_else(|| node.pointer("/coverImage/large").and_then(|x| x.as_str()))
        .unwrap_or("")
        .to_string();
    if poster_url.is_empty() {
        return;
    }
    let release_year = node
        .get("seasonYear")
        .and_then(|x| x.as_i64())
        .or_else(|| node.pointer("/startDate/year").and_then(|x| x.as_i64()))
        .unwrap_or(2024)
        .to_string();
    let total_episodes = node
        .get("episodes")
        .and_then(|x| x.as_u64())
        .unwrap_or(12) as u32;
    let score = node
        .get("averageScore")
        .and_then(|x| x.as_f64())
        .map(|s| s / 10.0)
        .unwrap_or(8.2);

    let label = if is_curr {
        "CURRENT".to_string()
    } else {
        match rel_type {
            "PREQUEL" => "PREQUEL (SEASON SEBELUMNYA)".to_string(),
            "SEQUEL" => "SEQUEL (SEASON SELANJUTNYA)".to_string(),
            "SIDE_STORY" | "SPIN_OFF" => "SPIN-OFF / OVA".to_string(),
            _ if format == "MOVIE" => "MOVIE".to_string(),
            _ => "SERIES / SEASON".to_string(),
        }
    };

    list.push(RelatedSeasonItem {
        id: nid,
        season_number: 1,
        media_type: "anime".into(),
        title,
        poster_url,
        release_year,
        total_episodes,
        rating: format!("{:.1}", score),
        relation_label: label,
        is_current: is_curr,
    });
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
            related_seasons: vec![],
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
            related_seasons: vec![],
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
            related_seasons: vec![],
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
            related_seasons: vec![],
            recommendations: vec![],
        },
    ]
}
