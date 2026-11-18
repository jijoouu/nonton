# AI Agent Instructions & Technical Blueprint (`AGENTS.md`)

Dokumen ini adalah panduan operasional utama (*System Prompt & Engineering Blueprint*) untuk AI Coding Agent yang bertugas membangun dan mengembangkan proyek **Nonton** — platform streaming terpadu untuk Film, Serial TV, K-Drama, dan Anime berbasis **Rust Full-Stack**.

> **ATURAN WAJIB UNTUK AI AGENT:**
> Setiap kali Anda diminta memulai tugas baru, menambahkan fitur, atau memperbaiki *bug* di repositori ini, baca dan patuhi arsitektur Rust, struktur direktori, serta kontrak desain di bawah ini bersama dengan dokumen [`PRD.md`](./PRD.md).

---

## 1. Tech Stack Resmi (100% Pure Rust Edition)
Seluruh aplikasi dibangun menggunakan ekosistem **Rust** agar menghasilkan 1 file binary `.exe` yang sangat kecil (~5–8 MB), konsumsi RAM super hemat (< 15 MB), dan waktu render halaman di bawah 1 milidetik:

- **Web Framework & Runtime:** `axum` 0.7 + `tokio` (Async Multi-threaded Rust Server)
- **SSR Template Engine:** `askama` + `askama_axum` (Type-safe HTML templates yang dikompilasi langsung ke dalam binary Rust)
- **HTTP Client:** `reqwest` (`rustls-tls`) untuk query paralel (`tokio::join!`) ke **AniList GraphQL API** & **TMDB / 1Shows Index API**
- **Middleware:** `tower-http` (`ServeDir` untuk file statis + `CompressionLayer` Brotli & Gzip otomatis)
- **Frontend Styling & Motion:**
  - Tailwind CSS + Custom Stylesheet (`static/css/cinema.css`)
  - **Design System (`noth.in` Edition):** Terinspirasi dari `https://www.noth.in` — kontras monokromatik murni (`#FFFFFF` Hero Stage & `#000000` Works Stage), header `mix-blend-mode: difference`, tipografi editorial (`PP Neue Montreal` / `Inter` + `IBM Plex Mono`), tag tanda kurung `( The Spotlight )`, tombol kapsul `.noth-pill`, dan **Asymmetric 6-Column Grid (`.noth-work-grid`)**.
  - **Splash Screen & Scroll Choreography (`static/js/app.js`):** Preloader `N'` dengan 3D Balloon Sculpture dan counter `000 -> 100`, dipadukan dengan **Lenis Smooth Inertial Scroll**, *Hero Giant Wordmark Zoom-Through*, *Internal Card Image Parallax*, *Glitch Text-Scramble Manifesto*, dan *Monumental Scaling Footer*.
- **State & Local Persistence:** Zero-Login `localStorage` Engine (`window.NontonStore` di `static/js/app.js`) untuk mengelola *Watchlist*, *Continue Watching History*, *Watched Episodes*, *Recent Searches*, dan *Export/Import JSON Backup*.

---

## 2. Struktur Direktori Proyek (Directory Structure)

```text
d:\jijo\Project\nonton\
├── Cargo.toml                          # Konfigurasi dependensi Rust & optimasi ukuran binary release
├── .env.example                        # Contoh konfigurasi PORT (default 8080) & TMDB_API_KEY
├── .gitignore                          # Git ignore untuk /target/ dan file environment
├── PRD.md                              # Product Requirements Document
├── AGENTS.md                           # Blueprint teknis untuk AI Agent (file ini)
├── src/
│   ├── main.rs                         # Entrypoint Axum router, route handlers, & Askama structs
│   ├── models.rs                       # Struct data terpadu (`UnifiedMedia`, `StreamServer`, `ScheduleItem`)
│   ├── api.rs                          # Fetcher paralel untuk AniList GraphQL & TMDB/1Shows Search/Catalog
│   └── providers.rs                    # Multi-Server streaming URL builders (VidFast, VidLink, Vidy, VidZee, VidSrc)
├── templates/
│   ├── layout.html                     # Master Layout (Header Cinema, Search Modal Ctrl+K, Mobile Nav)
│   ├── home.html                       # Beranda (Hero Banner, Continue Watching, Shelves/Carousels)
│   ├── search.html                     # Halaman Hasil Pencarian Penuh (`/search?q=...`) dengan filter kategori
│   ├── catalog.html                    # Halaman Katalog & Filter Genre (`/movies`, `/tv`, `/anime`, `K-Drama`)
│   ├── schedule.html                   # Halaman Jadwal Rilis Mingguan Anime (`/schedule` Senin–Minggu)
│   ├── watchlist.html                  # Halaman Arsip Watchlist & Export/Import Backup JSON (`/watchlist`)
│   ├── detail.html                     # Halaman Detail Konten (`/movie/:id`, `/tv/:id`, `/anime/:id`)
│   └── watch.html                      # Halaman Streaming Multi-Server (`/watch/...` + Theater & Lampu Mati)
├── static/
│   ├── css/
│   │   └── cinema.css                  # Custom Stylesheet Obsidian Cinema
│   └── js/
│       └── app.js                      # Search Modal Ctrl+K & NontonStore LocalStorage
```

---

## 3. Cara Menjalankan & Memverifikasi Proyek

1. **Verifikasi Kompilasi Cepat (Tanpa Menjalankan Server):**
   ```powershell
   cargo check
   ```
2. **Menjalankan Server Development:**
   ```powershell
   cargo run
   ```
   Akses langsung di **`http://localhost:8080`**.
3. **Membangun File Binary Produksi Super Kecil (`.exe`):**
   ```powershell
   cargo build --release
   ```
   File binary tunggal akan tersedia di `target/release/nonton-rust.exe`.
