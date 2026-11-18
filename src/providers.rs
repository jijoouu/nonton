use crate::models::StreamServer;

pub fn get_movie_servers(tmdb_id: &str) -> Vec<StreamServer> {
    vec![
        StreamServer {
            id: "vidfast".into(),
            name: "Server 1 • VidFast Pro".into(),
            badge: "Utama • Super Cepat".into(),
            quality: "4K".into(),
            embed_url: format!(
                "https://vidfast.pro/movie/{}?autoPlay=true&fullscreenButton=true&theme=ffffff",
                tmdb_id
            ),
        },
        StreamServer {
            id: "vidlink".into(),
            name: "Server 2 • VidLink Ultra".into(),
            badge: "Auto-Sub • 4K".into(),
            quality: "4K".into(),
            embed_url: format!(
                "https://vidlink.pro/movie/{}?primaryColor=ffffff&autoplay=true",
                tmdb_id
            ),
        },
        StreamServer {
            id: "vidy".into(),
            name: "Server 3 • Vidy ST".into(),
            badge: "Multi-Audio • HD".into(),
            quality: "1080p".into(),
            embed_url: format!("https://www.vidy.st/movie/{}", tmdb_id),
        },
        StreamServer {
            id: "vidzee".into(),
            name: "Server 4 • VidZee Player".into(),
            badge: "Low Buffer".into(),
            quality: "1080p".into(),
            embed_url: format!("https://player.vidzee.wtf/embed/movie/{}", tmdb_id),
        },
        StreamServer {
            id: "vidsrc-net".into(),
            name: "Server 5 • VidSrc Net".into(),
            badge: "Stabil • Multi-Sub".into(),
            quality: "1080p".into(),
            embed_url: format!("https://vidsrc.net/embed/movie/{}", tmdb_id),
        },
    ]
}

pub fn get_tv_servers(tmdb_id: &str, season: u32, episode: u32) -> Vec<StreamServer> {
    vec![
        StreamServer {
            id: "vidfast-tv".into(),
            name: "Server 1 • VidFast Pro".into(),
            badge: "Utama • Super Cepat".into(),
            quality: "4K".into(),
            embed_url: format!(
                "https://vidfast.pro/tv/{}/{}/{}?autoPlay=true&fullscreenButton=true&theme=ffffff",
                tmdb_id, season, episode
            ),
        },
        StreamServer {
            id: "vidlink-tv".into(),
            name: "Server 2 • VidLink Ultra".into(),
            badge: "Auto-Next • 4K".into(),
            quality: "4K".into(),
            embed_url: format!(
                "https://vidlink.pro/tv/{}/{}/{}?primaryColor=ffffff&autoplay=true",
                tmdb_id, season, episode
            ),
        },
        StreamServer {
            id: "vidy-tv".into(),
            name: "Server 3 • Vidy ST".into(),
            badge: "Stabil • Multi-Sub".into(),
            quality: "1080p".into(),
            embed_url: format!("https://www.vidy.st/tv/{}/{}/{}", tmdb_id, season, episode),
        },
        StreamServer {
            id: "vidzee-tv".into(),
            name: "Server 4 • VidZee Player".into(),
            badge: "Cepat • HD".into(),
            quality: "1080p".into(),
            embed_url: format!(
                "https://player.vidzee.wtf/embed/tv/{}/{}/{}",
                tmdb_id, season, episode
            ),
        },
        StreamServer {
            id: "vidsrc-net-tv".into(),
            name: "Server 5 • VidSrc Net".into(),
            badge: "Multi-Sub".into(),
            quality: "1080p".into(),
            embed_url: format!(
                "https://vidsrc.net/embed/tv?tmdb={}&season={}&episode={}",
                tmdb_id, season, episode
            ),
        },
    ]
}

pub fn get_anime_servers(
    tmdb_id: &str,
    season: u32,
    is_movie: bool,
    mal_id: Option<u32>,
    episode: u32,
    audio: &str,
) -> Vec<StreamServer> {
    let audio_norm = if audio.eq_ignore_ascii_case("dub") {
        "dub"
    } else {
        "sub"
    };
    let audio_upper = audio_norm.to_uppercase();

    if is_movie {
        return get_movie_servers(tmdb_id);
    }

    let mut list = vec![
        StreamServer {
            id: "vidfast-anime".into(),
            name: format!("Server 1 • VidFast Pro ({})", audio_upper),
            badge: "Utama • Super Cepat".into(),
            quality: "4K".into(),
            embed_url: format!(
                "https://vidfast.pro/tv/{}/{}/{}?autoPlay=true&fullscreenButton=true&theme=ffffff",
                tmdb_id, season, episode
            ),
        },
        StreamServer {
            id: "vidlink-tv-anime".into(),
            name: format!("Server 2 • VidLink Ultra ({})", audio_upper),
            badge: "Auto-Sub • 4K".into(),
            quality: "4K".into(),
            embed_url: format!(
                "https://vidlink.pro/tv/{}/{}/{}?primaryColor=ffffff&autoplay=true",
                tmdb_id, season, episode
            ),
        },
        StreamServer {
            id: "vidy-anime".into(),
            name: format!("Server 3 • Vidy ST ({})", audio_upper),
            badge: "Multi-Audio • HD".into(),
            quality: "1080p".into(),
            embed_url: format!("https://www.vidy.st/tv/{}/{}/{}", tmdb_id, season, episode),
        },
        StreamServer {
            id: "vidzee-anime".into(),
            name: format!("Server 4 • VidZee Player ({})", audio_upper),
            badge: "Low Buffer".into(),
            quality: "1080p".into(),
            embed_url: format!(
                "https://player.vidzee.wtf/embed/tv/{}/{}/{}",
                tmdb_id, season, episode
            ),
        },
        StreamServer {
            id: "vidsrc-net-anime".into(),
            name: format!("Server 5 • VidSrc Net ({})", audio_upper),
            badge: "Multi-Sub".into(),
            quality: "1080p".into(),
            embed_url: format!(
                "https://vidsrc.net/embed/tv?tmdb={}&season={}&episode={}",
                tmdb_id, season, episode
            ),
        },
    ];

    if let Some(mal) = mal_id {
        list.push(StreamServer {
            id: "vidlink-mal".into(),
            name: format!("Server 6 • Direct MAL ({})", audio_upper),
            badge: "AniSkip • Direct".into(),
            quality: "1080p".into(),
            embed_url: format!(
                "https://vidlink.pro/anime/{}/{}/{}?primaryColor=ffffff&fallback=true&autoplay=true",
                mal, episode, audio_norm
            ),
        });
    }

    list
}
