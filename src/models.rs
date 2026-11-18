use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CastMember {
    pub id: String,
    pub name: String,
    pub role: String,
    pub image_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeasonInfo {
    pub season_number: u32,
    pub name: String,
    pub episode_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedSeasonItem {
    pub id: String,
    pub season_number: u32,
    pub media_type: String,
    pub title: String,
    pub poster_url: String,
    pub release_year: String,
    pub total_episodes: u32,
    pub rating: String,
    pub relation_label: String,
    pub is_current: bool,
}

impl RelatedSeasonItem {
    pub fn watch_href(&self) -> String {
        match self.media_type.as_str() {
            "movie" => format!("/watch/movie/{}", self.id),
            "tv" => format!("/watch/tv/{}/{}/1", self.id, self.season_number.max(1)),
            _ => format!("/watch/anime/{}/1", self.id),
        }
    }

    pub fn detail_href(&self) -> String {
        match self.media_type.as_str() {
            "tv" => format!("/tv/{}?season={}", self.id, self.season_number.max(1)),
            _ => format!("/{}/{}", self.media_type, self.id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMedia {
    pub id: String,
    pub mal_id: Option<u32>,
    pub media_type: String, // "movie" | "tv" | "anime"
    pub title: String,
    pub original_title: String,
    pub poster_url: String,
    pub backdrop_url: String,
    pub synopsis: String,
    pub rating: String, // Formatted e.g. "8.9"
    pub release_year: String,
    pub status: String, // "Ongoing" | "Completed"
    pub genres: Vec<String>,
    pub total_episodes: u32,
    pub duration: u32,
    pub studio: String,
    pub trailer_youtube_id: Option<String>,
    pub next_airing_label: Option<String>,
    pub cast: Vec<CastMember>,
    pub seasons: Vec<SeasonInfo>,
    pub related_seasons: Vec<RelatedSeasonItem>,
    pub recommendations: Vec<UnifiedMedia>,
}

impl UnifiedMedia {
    pub fn watch_href(&self) -> String {
        match self.media_type.as_str() {
            "movie" => format!("/watch/movie/{}", self.id),
            "tv" => format!("/watch/tv/{}/1/1", self.id),
            _ => format!("/watch/anime/{}/1", self.id),
        }
    }

    pub fn detail_href(&self) -> String {
        format!("/{}/{}", self.media_type, self.id)
    }

    pub fn badge_label(&self) -> &'static str {
        match self.media_type.as_str() {
            "movie" => "MOVIE",
            "tv" => "TV SERIES",
            _ => "ANIME",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamServer {
    pub id: String,
    pub name: String,
    pub badge: String,
    pub quality: String,
    pub embed_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleItem {
    pub id: String,
    pub episode: u32,
    pub day_index: u32, // 0 = Minggu, 1 = Senin .. 6 = Sabtu
    pub time_wib: String,
    pub countdown_label: String,
    pub media: UnifiedMedia,
}
