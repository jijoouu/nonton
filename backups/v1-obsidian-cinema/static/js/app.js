/**
 * Nonton Rust Engine — Zero-Framework Client Interactivity & Animations
 * Mengelola LocalStorage (Watchlist, History, Watched Episodes, Backup JSON),
 * Command Palette Search (Ctrl+K), Hero Carousel Auto-Slide, dan Player Controls.
 */

const STORAGE_KEY = "nonton-user-storage-rust";

const NontonStore = {
  getState() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) {
        return { watchlist: [], history: [], watchedEpisodes: {}, defaultAudio: "sub" };
      }
      return JSON.parse(raw);
    } catch {
      return { watchlist: [], history: [], watchedEpisodes: {}, defaultAudio: "sub" };
    }
  },

  saveState(state) {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    this.updateNavbarBadge();
  },

  isInWatchlist(id, type) {
    const state = this.getState();
    return state.watchlist.some((w) => w.id === String(id) && w.media_type === type);
  },

  toggleWatchlist(item) {
    const state = this.getState();
    const exists = state.watchlist.some(
      (w) => w.id === String(item.id) && w.media_type === item.media_type
    );
    if (exists) {
      state.watchlist = state.watchlist.filter(
        (w) => !(w.id === String(item.id) && w.media_type === item.media_type)
      );
    } else {
      state.watchlist.unshift(item);
    }
    this.saveState(state);
    return !exists;
  },

  upsertHistory(entry) {
    const state = this.getState();
    state.history = (state.history || []).filter(
      (h) => !(h.id === String(entry.id) && h.media_type === entry.media_type)
    );
    state.history.unshift({
      ...entry,
      updatedAt: Date.now(),
    });
    state.history = state.history.slice(0, 50);

    const key = `${entry.media_type}-${entry.id}`;
    state.watchedEpisodes = state.watchedEpisodes || {};
    const epList = state.watchedEpisodes[key] || [];
    const epStr = String(entry.episode || 1);
    if (!epList.includes(epStr)) {
      epList.push(epStr);
    }
    state.watchedEpisodes[key] = epList;

    this.saveState(state);
  },

  updateNavbarBadge() {
    const badge = document.getElementById("nav-watchlist-count");
    if (!badge) return;
    const count = this.getState().watchlist.length;
    if (count > 0) {
      badge.textContent = count;
      badge.classList.remove("hidden");
    } else {
      badge.classList.add("hidden");
    }
  },
};

window.NontonStore = NontonStore;

document.addEventListener("DOMContentLoaded", () => {
  NontonStore.updateNavbarBadge();
  initSearchModal();
  initHeroCarousel();
  initContinueWatchingRow();
  initWatchlistButtons();
});

function initSearchModal() {
  const modal = document.getElementById("search-modal");
  const openBtn = document.getElementById("open-search-btn");
  const closeBtn = document.getElementById("close-search-btn");
  const backdrop = document.getElementById("search-modal-backdrop");
  const input = document.getElementById("instant-search-input");
  const resultsBox = document.getElementById("instant-search-results");

  if (!modal) return;

  const openModal = () => {
    modal.classList.remove("hidden");
    setTimeout(() => input && input.focus(), 50);
  };

  const closeModal = () => {
    modal.classList.add("hidden");
  };

  if (openBtn) openBtn.addEventListener("click", openModal);
  if (closeBtn) closeBtn.addEventListener("click", closeModal);
  if (backdrop) backdrop.addEventListener("click", closeModal);

  window.addEventListener("keydown", (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      if (modal.classList.contains("hidden")) openModal();
      else closeModal();
    } else if (e.key === "Escape" && !modal.classList.contains("hidden")) {
      closeModal();
    }
  });

  let timer = null;
  if (input && resultsBox) {
    input.addEventListener("input", () => {
      clearTimeout(timer);
      const q = input.value.trim();
      if (q.length < 2) {
        resultsBox.innerHTML = `<div class="py-8 text-center text-xs text-zinc-400">Ketik minimal 2 huruf untuk mencari Film, Serial TV, dan Anime secara instan...</div>`;
        return;
      }
      resultsBox.innerHTML = `<div class="py-8 text-center text-xs text-rose-400 animate-pulse">Mencari "${q}" di engine Rust...</div>`;
      timer = setTimeout(async () => {
        try {
          const res = await fetch(`/api/search?q=${encodeURIComponent(q)}`);
          const data = await res.json();
          if (!data.results || data.results.length === 0) {
            resultsBox.innerHTML = `<div class="py-8 text-center text-xs text-zinc-400">Tidak ada hasil untuk "${q}".</div>`;
            return;
          }
          resultsBox.innerHTML =
            `<a href="/search?q=${encodeURIComponent(q)}" class="flex items-center justify-between rounded-xl border border-rose-500/40 bg-rose-500/15 px-3.5 py-2.5 text-xs font-bold text-rose-300 hover:bg-rose-500/25 transition-colors">
              <span>🔎 Lihat semua hasil lengkap untuk "${q}" (Film, K-Drama & Anime)</span>
              <span class="font-mono">Enter ↵</span>
            </a>` +
            data.results
              .map(
                (item) => `
              <div class="flex items-center justify-between gap-3 rounded-xl bg-zinc-900/70 border border-white/5 p-2.5 hover:border-rose-500/40 hover:bg-zinc-900 transition-all">
                <a href="/${item.media_type}/${item.id}" class="flex items-center gap-3 flex-1 min-w-0">
                  <img src="${item.poster_url}" alt="${item.title}" class="h-14 w-10 rounded-lg object-cover shrink-0 bg-zinc-800" />
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="rounded bg-rose-500/20 border border-rose-500/40 px-1.5 py-0.5 font-mono text-[10px] font-bold uppercase text-rose-300">${item.media_type}</span>
                      <span class="font-mono text-xs text-amber-400">★ ${item.rating}</span>
                      <span class="font-mono text-xs text-zinc-400">${item.release_year}</span>
                    </div>
                    <h4 class="text-sm font-bold text-white truncate mt-0.5">${item.title}</h4>
                  </div>
                </a>
                <a href="${item.media_type === "movie" ? `/watch/movie/${item.id}` : item.media_type === "tv" ? `/watch/tv/${item.id}/1/1` : `/watch/anime/${item.id}/1`}" class="rounded-lg bg-rose-600 px-3 py-2 text-xs font-bold text-white hover:bg-rose-500 shrink-0">
                  ▶ Tonton
                </a>
              </div>
            `
              )
              .join("");
        } catch {
          resultsBox.innerHTML = `<div class="py-8 text-center text-xs text-rose-400">Gagal memuat pencarian.</div>`;
        }
      }, 220);
    });
  }
}

function initHeroCarousel() {
  const slides = document.querySelectorAll("[data-hero-slide]");
  const dots = document.querySelectorAll("[data-hero-dot]");
  if (slides.length <= 1) return;

  let current = 0;
  const showSlide = (idx) => {
    slides.forEach((s, i) => {
      if (i === idx) {
        s.classList.remove("hidden");
      } else {
        s.classList.add("hidden");
      }
    });
    dots.forEach((d, i) => {
      if (i === idx) {
        d.className = "h-1.5 w-8 rounded-full bg-rose-500 transition-all";
      } else {
        d.className = "h-1.5 w-2 rounded-full bg-white/30 hover:bg-white/60 transition-all";
      }
    });
    current = idx;
  };

  dots.forEach((dot, i) => {
    dot.addEventListener("click", () => showSlide(i));
  });

  setInterval(() => {
    showSlide((current + 1) % slides.length);
  }, 6500);
}

function initContinueWatchingRow() {
  const container = document.getElementById("continue-watching-section");
  const listEl = document.getElementById("continue-watching-list");
  const clearBtn = document.getElementById("clear-history-btn");
  if (!container || !listEl) return;

  const render = () => {
    const history = NontonStore.getState().history || [];
    if (history.length === 0) {
      container.classList.add("hidden");
      return;
    }
    container.classList.remove("hidden");
    listEl.innerHTML = history
      .slice(0, 10)
      .map((item) => {
        const href =
          item.media_type === "movie"
            ? `/watch/movie/${item.id}`
            : item.media_type === "tv"
            ? `/watch/tv/${item.id}/${item.season || 1}/${item.episode || 1}`
            : `/watch/anime/${item.id}/${item.episode || 1}`;
        const label =
          item.media_type === "movie"
            ? "Full Movie"
            : item.media_type === "tv"
            ? `S${item.season || 1} • Ep ${item.episode || 1}`
            : `Episode ${item.episode || 1}`;

        return `
          <a href="${href}" class="poster-card group relative w-[260px] shrink-0 overflow-hidden rounded-xl border border-white/10 bg-zinc-900/80 block">
            <div class="relative aspect-video w-full overflow-hidden bg-zinc-950">
              <img src="${item.backdrop_url || item.poster_url}" alt="${item.title}" class="poster-img h-full w-full object-cover" />
              <div class="absolute inset-0 bg-gradient-to-t from-zinc-950 via-zinc-950/30 to-transparent"></div>
              <div class="absolute bottom-2 left-3 right-3 flex items-center justify-between font-mono text-xs text-zinc-200">
                <span>${label}</span>
                <span class="text-rose-400 font-bold">Lanjut →</span>
              </div>
              <div class="absolute bottom-0 inset-x-0 h-1 bg-zinc-800">
                <div class="h-full w-2/3 bg-rose-500"></div>
              </div>
            </div>
            <div class="p-3">
              <h4 class="truncate text-sm font-bold text-white group-hover:text-rose-400">${item.title}</h4>
            </div>
          </a>
        `;
      })
      .join("");
  };

  render();
  if (clearBtn) {
    clearBtn.addEventListener("click", () => {
      const st = NontonStore.getState();
      st.history = [];
      NontonStore.saveState(st);
      render();
    });
  }
}

function initWatchlistButtons() {
  document.querySelectorAll("[data-watchlist-btn]").forEach((btn) => {
    const item = {
      id: btn.dataset.id,
      media_type: btn.dataset.type,
      title: btn.dataset.title,
      poster_url: btn.dataset.poster,
      backdrop_url: btn.dataset.backdrop,
      rating: btn.dataset.rating,
      release_year: btn.dataset.year,
    };

    const updateLabel = () => {
      const saved = NontonStore.isInWatchlist(item.id, item.media_type);
      btn.textContent = saved ? "✓ Tersimpan di Watchlist" : "+ Watchlist";
    };

    updateLabel();
    btn.addEventListener("click", (e) => {
      e.preventDefault();
      NontonStore.toggleWatchlist(item);
      updateLabel();
    });
  });
}

window.scrollMediaRow = function (btn, dir) {
  const section = btn.closest("section");
  const track = section ? section.querySelector("[data-row-track]") : null;
  if (!track) return;
  const amount = track.clientWidth * 0.75;
  track.scrollBy({ left: dir === "left" ? -amount : amount, behavior: "smooth" });
};
