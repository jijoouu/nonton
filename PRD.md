# Product Requirements Document (PRD)
## Project Name: **Nonton** — Unified Movie, TV Series & Anime Streaming Platform

| Metadata | Detail |
| :--- | :--- |
| **Versi Dokumen** | 1.0 (MVP) |
| **Status** | Ready for Development |
| **Platform** | Web Application (Responsive Desktop, Tablet, Mobile & Smart TV Browser) |
| **Tech Stack Utama** | Edisi 1: Next.js (App Router) + TypeScript / Edisi 2 (`nonton-rust/`): Rust (`Axum` + `Askama` SSR) |
| **Model Autentikasi** | Zero-Auth / Frictionless (LocalStorage & IndexedDB Persistence) |

---

## 1. Executive Summary & Product Vision
**Nonton** adalah platform web streaming modern satu pintu (*all-in-one*) yang menggabungkan katalog **Film (Movies)**, **Serial TV (TV Shows)**, dan **Anime** terlengkap di dunia.

Platform ini dirancang dengan filosofi **Zero-Friction Entertainment**: pengguna dapat langsung mencari dan menonton konten favorit mereka tanpa harus mendaftar akun atau login, namun tetap menikmati fitur personalisasi modern seperti *Continue Watching* (riwayat tontonan hingga detik terakhir), *Watchlist* (daftar simpanan), pemilihan *Multi-Server*, serta fitur khusus pecinta anime (*Sub/Dub toggle*, *Jadwal Rilis Mingguan*, *Auto-Next Episode*, dan *Skip Intro*).

---

## 2. Target Pengguna (User Personas)
1. **Binge-Watcher Anime:** Pengguna yang mengikuti anime musiman (*ongoing*), membutuhkan jadwal rilis harian yang akurat, sensitif terhadap pilihan *Sub* (bahasa asli + teks) vs *Dub*, dan menyukai fitur *Auto-Next* & *Skip Intro/Outro*.
2. **Casual Movie/Series Viewer:** Pengguna yang ingin menonton film box office atau serial barat/drama Asia di akhir pekan dengan cepat, antarmuka bersih seperti Netflix/Apple TV+, dan opsi pergantian server otomatis jika satu server mengalami *buffering*.
3. **Mobile Commuter:** Pengguna yang mengakses lewat smartphone dan membutuhkan UI yang ringan, hemat kuota, serta kontrol pemutar video yang ramah sentuhan (*touch-friendly*).

---

## 3. Arsitektur Data & Sumber Konten (Data & Streaming Architecture)

### 3.1 Sumber Metadata Katalog (Public APIs)
Website ini tidak menyimpan database film secara manual, melainkan melakukan *fetching* dan *caching* (ISR/Server Cache) dari API publik terpercaya:
1. **TMDB API (The Movie Database):**
   - Sumber utama untuk **Movies** dan **TV Series** (Barat, Korea, Indonesia, dll.).
   - Menyediakan: Poster HD, Backdrop, Sinopsis, Rating, Cast & Crew, Trailer YouTube, Daftar Season & Episode, serta Rekomendasi serupa.
2. **AniList GraphQL API (dengan Jikan/MyAnimeList API sebagai Fallback):**
   - Sumber utama untuk **Anime** (TV, Movie, OVA, ONA).
   - Menyediakan: Judul (Romaji, English, Native), Sinopsis, Studio, Karakter & Seiyuu, *Airing Schedule* (jadwal tayang), relasi Prekuel/Sekuel, serta pemetaan ID (AniList ID & MAL ID).

### 3.2 Arsitektur Video Player & Multi-Server Streaming
Pemutar video mendukung 2 mode pemutaran yang dapat dipilih pengguna melalui tab **Pilih Server**:
1. **Multi-Server Embed Player (Sandboxed Iframe):**
   - Mengintegrasikan penyedia streaming berbasis ID (`TMDB_ID`, `IMDB_ID`, `ANILIST_ID`, `MAL_ID`).
   - Memiliki minimal 3–5 opsi server cadangan (misal: *Server Utama (VidSrc)*, *Server Cepat (SuperEmbed)*, *Server Anime HD*, dll.) sehingga jika satu server mati, pengguna cukup klik server lain tanpa *reload* halaman.
2. **Custom HLS/M3U8 Player (Vidstack / Artplayer / HLS.js):**
   - Digunakan ketika sumber stream menyediakan tautan langsung `.m3u8` / `.mp4` dan file subtitle `.vtt` / `.srt`.
   - Mendukung kustomisasi subtitle (ukuran font, warna, background), pemilihan resolusi (360p, 720p, 1080p, Auto), kecepatan putar (0.5x – 2.0x), *Picture-in-Picture (PiP)*, serta *Skip Intro/Outro* berbasis *timestamp* (Aniskip API).

---

## 4. Fitur Utama (Core Features - MVP)

### 4.1 Halaman Utama (Home & Discovery)
- **Hero Spotlight Carousel:** Banner layar penuh untuk 5–8 konten paling populer minggu ini (gabungan Film, Series, & Anime) lengkap dengan logo/judul, badge (Rating, Tahun, Kualitas HD, Genre), sinopsis singkat, tombol **Tonton Sekarang**, dan **+ Watchlist**.
- **Continue Watching Row:** Baris khusus yang hanya muncul jika pengguna memiliki riwayat tontonan di `localStorage`. Menampilkan *progress bar* durasi, episode terakhir yang ditonton (misal: `S1:E4` atau `Ep 12`), dan tombol hapus dari riwayat.
- **Categorized Content Rows:**
  - *Sedang Trending Hari Ini* (Semua Kategori)
  - *Anime Ongoing Terpopuler Musim Ini*
  - *Film Box Office Terbaru*
  - *Serial TV & Drama Terpopuler*
  - *Top Rated Sepanjang Masa*

### 4.2 Pencarian Instan & Eksplorasi (Unified Search & Advanced Filter)
- **Command Palette / Instant Search (`Ctrl + K` atau klik ikon cari):**
  - *Debounced search* (300ms) yang menampilkan hasil langsung dari TMDB dan AniList secara paralel dalam satu dropdown.
  - Menampilkan *thumbnail*, judul, tahun rilis, tipe badge (`MOVIE`, `TV`, `ANIME`), dan skor rating.
- **Halaman Katalog & Filter Lanjutan (`/movies`, `/tv`, `/anime`, `/explore`):**
  - Filter berdasarkan: Genre, Tahun Rilis, Status (*Ongoing / Completed*), Negara/Tipe, dan Urutan (*Populer, Rating Tertinggi, Terbaru*).
  - *Infinite Scroll* atau *Pagination* yang mulus dengan *Skeleton Loading*.

### 4.3 Fitur Khusus Anime (Anime-Centric Suite)
1. **Jadwal Rilis Mingguan (`/schedule`):**
   - Tampilan tab harian (**Senin s/d Minggu**), otomatis menyorot hari ini sesuai zona waktu lokal pengguna.
   - Menampilkan kartu anime yang tayang hari tersebut lengkap dengan jam tayang dan *countdown timer* menuju episode berikutnya.
2. **Sub / Dub Toggle:**
   - Tombol switch instan di halaman *Watch* khusus anime untuk berpindah antara versi **SUB (Japanese Audio + Subtitle)** dan **DUB (English/Localized Audio)**, yang disimpan sebagai preferensi default di `localStorage`.
3. **Auto-Next Episode & Auto-Play:**
   - Toggle on/off di bawah player. Jika aktif, otomatis berpindah ke episode berikutnya saat episode selesai (atau lewat tombol *Next Episode* yang muncul di akhir video).
4. **Skip Intro / Outro:**
   - Integrasi dengan *AniSkip API* (untuk player HLS) atau tombol *Quick Skip (+85 detik)* untuk melewati lagu pembuka (*Opening*) anime dengan satu klik.
5. **Filler Episode Indicator (Opsional/Bonus):**
   - Menandai episode filler pada daftar episode anime yang panjang.

### 4.4 Halaman Detail & Halaman Nonton (Detail & Watch Experience)
- **Halaman Detail (`/movie/[id]`, `/tv/[id]`, `/anime/[id]`):**
  - Backdrop sinematik dengan *gradient vignette*.
  - Informasi komprehensif: Sinopsis, Genre, Rating, Durasi, Studio/Sutradara, Pemeran/Karakter, Trailer Modal (YouTube), Daftar Season/Episode, serta *Rekomendasi Serupa*.
- **Halaman Nonton (`/watch/movie/[id]`, `/watch/tv/[id]/[season]/[ep]`, `/watch/anime/[id]/[ep]`):**
  - **Video Player Container (16:9):** Responsif dan berada di posisi paling atas.
  - **Player Control Bar (di bawah video):**
    - Tombol **Prev / Next Episode**.
    - Toggle **Auto-Next** & **Sub/Dub** (untuk Anime).
    - Tombol **Theater Mode** (melebarkan player) & **Lampu Mati (Cinema Dimming)** (menggelapkan seluruh layar kecuali player).
    - Tombol **Laporkan Server Rusak / Ganti Server**.
  - **Server Selector Panel:** Pilihan daftar server streaming dengan indikator status/kecepatan.
  - **Episode Selector Sidebar / Grid:**
    - Untuk TV Series: Dropdown pilih Season + daftar episode lengkap dengan judul & *thumbnail*.
    - Untuk Anime dengan ratusan episode (misal: *One Piece*): Pengelompokan *range* episode (`1-100`, `101-200`, dst.) + kolom cari nomor episode + penanda episode yang **Sudah Ditonton (Watched)**.

### 4.5 Penyimpanan Lokal Tanpa Login (LocalStorage State Management)
Semua data personal pengguna disimpan di sisi klien (`localStorage`) dengan skema JSON terstruktur:
- **Watchlist (`nonton_watchlist`):** Menyimpan daftar Film, TV Series, dan Anime yang ditandai pengguna, dapat difilter di halaman `/watchlist`.
- **Watch History (`nonton_history`):** Menyimpan ID konten, tipe, nomor season/episode terakhir, *timestamp* posisi video terakhir (detik), dan waktu terakhir ditonton.
- **Watched Episodes (`nonton_watched_eps`):** Menyimpan array episode mana saja yang sudah selesai ditonton per serial/anime agar tampil dengan warna redup/centang di daftar episode.
- **User Preferences (`nonton_prefs`):** Menyimpan pilihan default server, preferensi `sub` vs `dub`, dan status `autoNext`.
- **Fitur Export / Import Data (Backup JSON):** Di halaman `/watchlist`, pengguna dapat mengunduh file `.json` backup riwayat & watchlist mereka atau mengunggahnya ke perangkat lain tanpa perlu akun.

---

## 5. Desain UI/UX & Identitas Visual (Design System)
- **Tema Utama:** *Ultra-Dark Cinema Mode* (`#09090b` / `zinc-950` sebagai warna latar utama, `#18181b` / `zinc-900` untuk kartu/panel) untuk kenyamanan mata saat menonton di ruangan gelap.
- **Warna Aksen (Brand Accent):** *Crimson Violet / Electric Indigo* atau *Cinema Amber* dengan gradien halus untuk elemen interaktif utama.
- **Tipografi:** `Plus Jakarta Sans` atau `Inter` untuk teks UI, dan `JetBrains Mono` untuk angka durasi, episode, dan badge teknis.
- **Micro-Interactions:**
  - Efek *hover scale* halus pada poster kartu film disertai tampilan cepat (*Quick Play*, rating, tahun).
  - *Glassmorphism* (`backdrop-blur-md bg-black/60`) pada *Navbar*, *Command Search*, dan kontrol overlay.
  - *Skeleton Shimmer* di setiap area yang sedang memuat data API (dilarang menggunakan *blank screen* atau sekadar tulisan "Loading...").

---

## 6. Kebutuhan Non-Fungsional (Non-Functional Requirements)
1. **Performa & Kecepatan:**
   - Skor Google Lighthouse Performance > 85.
   - Penggunaan `next/image` dengan optimasi ukuran dan *lazy loading* untuk semua poster TMDB & AniList.
   - *Data Caching* menggunakan fitur `fetch` cache Next.js (`revalidate: 3600` untuk halaman populer/jadwal) agar hemat kuota rate-limit API dan pemuatan halaman instan.
2. **Keamanan & Kenyamanan Iframe:**
   - Memberikan atribut `sandbox` atau filter keamanan yang tepat serta peringatan/pemblokir pop-up dasar jika menggunakan server embed pihak ketiga.
   - Menyembunyikan API Key sensitif menggunakan *Environment Variables* (`.env.local`) dan memanggil API melalui Server Components atau Route Handlers (`/api/...`).
3. **Responsivitas:**
   - Navigasi *Top Navbar* untuk Desktop, dan *Bottom Navigation Bar* (Home, Explore, Anime Schedule, Watchlist) untuk layar Mobile (`< 768px`).

---

## 7. Peta Jalan Pengembangan (Development Roadmap)
- **Phase 1 — Foundation & UI Shell:** Setup Next.js App Router, Tailwind, Shadcn UI, konfigurasi tema Dark Cinema, Navbar, Bottom Nav Mobile, dan *types/interfaces* data terpadu (`MediaItem`).
- **Phase 2 — API Service Layer:** Integrasi klien API untuk TMDB (Movies & TV) dan AniList/Jikan (Anime), termasuk normalisasi data agar komponen UI dapat merender kartu Film, TV, dan Anime secara seragam.
- **Phase 3 — Discovery & Catalog Pages:** Pembuatan halaman Home (Hero Carousel + Content Rows), Instant Search Modal (`Ctrl+K`), halaman Explore/Filter (`/movies`, `/tv`, `/anime`), dan halaman Jadwal Rilis Anime (`/schedule`).
- **Phase 4 — Detail & Watch Engine:** Pembuatan halaman Detail konten dan halaman Watch lengkap dengan Multi-Server Switcher, Sub/Dub Toggle, Theater/Cinema Dimming Mode, dan Episode Selector (mendukung *chunking* 100 episode per tab).
- **Phase 5 — LocalStorage Persistence & Polish:** Integrasi penuh *Continue Watching*, *Watchlist* (`/watchlist`), *Watched Episode Badges*, fitur Export/Import JSON backup, serta optimasi SEO & *error boundaries*.
