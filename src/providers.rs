use crate::models::StreamServer;

pub fn get_movie_servers(tmdb_id: &str) -> Vec<StreamServer> {
    vec![
        StreamServer {
            id: "vidfast".into(),
            name: "Server 1 • VidFast Pro (1Shows)".into(),
            badge: "Utama • Super Cepat".into(),
            quality: "4K".into(),
            embed_url: format!(
                "https://vidfast.pro/movie/{}?autoPlay=true&theme=e11d48",
                tmdb_id
            ),
        },
        StreamServer {
            id: "vidlink".into(),
            name: "Server 2 • VidLink Ultra (1Shows)".into(),
            badge: "Auto-Sub • 4K".into(),
            quality: "4K".into(),
            embed_url: format!(
                "https://vidlink.pro/movie/{}?primaryColor=e11d48&autoplay=true",
                tmdb_id
            ),
        },
        StreamServer {
            id: "vidy".into(),
            name: "Server 3 • Vidy ST (1Shows)".into(),
            badge: "Multi-Audio • HD".into(),
            quality: "1080p".into(),
            embed_url: format!("https://www.vidy.st/movie/{}", tmdb_id),
        },
        StreamServer {
            id: "vidzee".into(),
            name: "Server 4 • VidZee Player (1Shows)".into(),
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
        StreamServer {
            id: "vidsrc-su".into(),
            name: "Server 6 • VidSrc SU".into(),
            badge: "Cadangan VIP".into(),
            quality: "1080p".into(),
            embed_url: format!("https://vidsrc.su/embed/movie/{}", tmdb_id),
        },
    ]
}

pub fn get_tv_servers(tmdb_id: &str, season: u32, episode: u32) -> Vec<StreamServer> {
    vec![
        StreamServer {
            id: "vidfast-tv".into(),
            name: "Server 1 • VidFast Pro (1Shows)".into(),
            badge: "Utama • Drakor/TV/Anime".into(),
            quality: "4K".into(),
            embed_url: format!(
                "https://vidfast.pro/tv/{}/{}/{}?autoPlay=true&theme=e11d48",
                tmdb_id, season, episode
            ),
        },
        StreamServer {
            id: "vidlink-tv".into(),
            name: "Server 2 • VidLink Ultra (1Shows)".into(),
            badge: "Auto-Next • 4K".into(),
            quality: "4K".into(),
            embed_url: format!(
                "https://vidlink.pro/tv/{}/{}/{}?primaryColor=e11d48&autoplay=true",
                tmdb_id, season, episode
            ),
        },
        StreamServer {
            id: "vidy-tv".into(),
            name: "Server 3 • Vidy ST (1Shows)".into(),
            badge: "Stabil • Multi-Sub".into(),
            quality: "1080p".into(),
            embed_url: format!("https://www.vidy.st/tv/{}/{}/{}", tmdb_id, season, episode),
        },
        StreamServer {
            id: "vidzee-tv".into(),
            name: "Server 4 • VidZee Player (1Shows)".into(),
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
    anilist_id: &str,
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

    let primary_embed = match mal_id {
        Some(mal) => format!(
            "https://vidlink.pro/anime/{}/{}/{}?primaryColor=e11d48&fallback=true&autoplay=true",
            mal, episode, audio_norm
        ),
        None => format!("https://vidsrc.su/embed/anime/{}/{}", anilist_id, episode),
    };

    let mut list = vec![
        StreamServer {
            id: "vidlink-anime".into(),
            name: format!("Server 1 • VidLink Anime ({})", audio_upper),
            badge: "Utama • AniSkip".into(),
            quality: "1080p".into(),
            embed_url: primary_embed,
        },
        StreamServer {
            id: "vidsrc-su-anime".into(),
            name: format!("Server 2 • VidSrc SU ({})", audio_upper),
            badge: "Cepat • 1080p".into(),
            quality: "1080p".into(),
            embed_url: format!("https://vidsrc.su/embed/anime/{}/{}", anilist_id, episode),
        },
        StreamServer {
            id: "vidsrc-rip-anime".into(),
            name: format!("Server 3 • VidSrc Rip ({})", audio_upper),
            badge: "Low Buffer".into(),
            quality: "1080p".into(),
            embed_url: format!("https://vidsrc.rip/embed/anime/{}/{}", anilist_id, episode),
        },
    ];

    if let Some(mal) = mal_id {
        list.push(StreamServer {
            id: "multiembed-mal".into(),
            name: format!("Server 4 • SuperEmbed MAL ({})", audio_upper),
            badge: "Cadangan MAL".into(),
            quality: "1080p".into(),
            embed_url: format!(
                "https://multiembed.mov/?video_id={}&mal=1&e={}",
                mal, episode
            ),
        });
    }

    list
}
