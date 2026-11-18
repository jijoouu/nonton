/**
 * Nonton Rust Engine — Criterion Program Index & Flight Scroll Choreography
 * Palette: Pantone P 1-9 C (#FBF9ED), Pantone P 112-14 C (#009AD0), Pantone P 40-16 C (#BE432C)
 * Scroll Reference: https://i-type-ny.groznov.co/flight/
 */

const STORAGE_KEY = "nonton-user-storage-rust";

const SVG_PLAY = `<svg class="w-3.5 h-3.5 fill-current inline-block shrink-0" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>`;
const SVG_SEARCH = `<svg class="w-3.5 h-3.5 stroke-current inline-block shrink-0" viewBox="0 0 24 24" fill="none" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>`;
const SVG_CHECK = `<svg class="w-3.5 h-3.5 stroke-current inline-block shrink-0" viewBox="0 0 24 24" fill="none" stroke-width="2.5"><path d="M20 6 9 17l-5-5"/></svg>`;
const SVG_PLUS = `<svg class="w-3.5 h-3.5 stroke-current inline-block shrink-0" viewBox="0 0 24 24" fill="none" stroke-width="2.2"><path d="M12 5v14M5 12h14"/></svg>`;

const NontonStore = {
  getState() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) {
        return {
          watchlist: [],
          history: [],
          watchedEpisodes: {},
          recentSearches: ["Godzilla", "Solo Leveling", "Queen of Tears", "Dune"],
          defaultAudio: "sub",
        };
      }
      const parsed = JSON.parse(raw);
      parsed.recentSearches = parsed.recentSearches || [
        "Godzilla",
        "Solo Leveling",
        "Queen of Tears",
        "Dune",
      ];
      return parsed;
    } catch {
      return {
        watchlist: [],
        history: [],
        watchedEpisodes: {},
        recentSearches: ["Godzilla", "Solo Leveling", "Queen of Tears"],
        defaultAudio: "sub",
      };
    }
  },

  saveState(state) {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    this.updateNavbarBadge();
  },

  addRecentSearch(keyword) {
    const clean = (keyword || "").trim();
    if (clean.length < 2) return;
    const state = this.getState();
    const list = (state.recentSearches || []).filter(
      (k) => k.toLowerCase() !== clean.toLowerCase()
    );
    list.unshift(clean);
    state.recentSearches = list.slice(0, 6);
    this.saveState(state);
  },

  isInWatchlist(id, type) {
    const state = this.getState();
    return state.watchlist.some(
      (w) => w.id === String(id) && w.media_type === type
    );
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
  initFlightScrollEngine();
  initSearchModal();
  initHeroCarousel();
  initContinueWatchingRow();
  initWatchlistButtons();
});

/**
 * Flight Inertial Scroll Engine (i-type-ny.groznov.co/flight style)
 */
function initFlightScrollEngine() {
  let currentY = window.scrollY || 0;
  let targetY = window.scrollY || 0;
  let ticking = false;

  const updateScrollVars = () => {
    const diff = targetY - currentY;
    currentY += diff * 0.12;

    if (Math.abs(diff) < 0.1) {
      currentY = targetY;
    }

    const maxScroll = Math.max(
      document.documentElement.scrollHeight - window.innerHeight,
      1
    );
    const progress = Math.min(Math.max(currentY / maxScroll, 0), 1);
    const velocity = Math.min(Math.max(diff / 60, -1), 1);

    const root = document.documentElement;
    root.style.setProperty("--scroll-y", currentY.toFixed(1));
    root.style.setProperty("--scroll-progress", progress.toFixed(4));
    root.style.setProperty("--scroll-velocity", velocity.toFixed(3));

    if (Math.abs(targetY - currentY) > 0.15) {
      requestAnimationFrame(updateScrollVars);
    } else {
      ticking = false;
    }
  };

  window.addEventListener(
    "scroll",
    () => {
      targetY = window.scrollY || 0;
      if (!ticking) {
        ticking = true;
        requestAnimationFrame(updateScrollVars);
      }
    },
    { passive: true }
  );

  updateScrollVars();
}

function initSearchModal() {
  const modal = document.getElementById("search-modal");
  const openBtn = document.getElementById("open-search-btn");
  const closeBtn = document.getElementById("close-search-btn");
  const backdrop = document.getElementById("search-modal-backdrop");
  const form = document.getElementById("instant-search-form");
  const input = document.getElementById("instant-search-input");
  const resultsBox = document.getElementById("instant-search-results");

  if (!modal) return;

  const renderEmptyState = () => {
    if (!resultsBox) return;
    const recent = NontonStore.getState().recentSearches || [];
    resultsBox.innerHTML = `
      <div class="p-4 space-y-4">
        <div>
          <p class="font-mono text-[11px] uppercase tracking-wider text-[#009AD0] mb-2.5">RIWAYAT KATA KUNCI POPULER</p>
          <div class="flex flex-wrap gap-2">
            ${recent
              .map(
                (kw) => `
              <a href="/search?q=${encodeURIComponent(kw)}" class="inline-flex items-center gap-1.5 border border-[#FBF9ED]/15 bg-[#0F1E2B] px-3 py-1.5 text-xs font-semibold text-[#FBF9ED] hover:border-[#009AD0] hover:text-[#009AD0] transition-colors">
                ${SVG_SEARCH} <span>${kw}</span>
              </a>
            `
              )
              .join("")}
          </div>
        </div>
        <div class="border-t border-[#FBF9ED]/10 pt-3 text-xs text-[#FBF9ED]/70">
          Ketik judul Film, Anime, atau K-Drama lalu tekan <strong>Enter</strong> untuk membuka indeks pencarian lengkap.
        </div>
      </div>
    `;
  };

  const openModal = () => {
    modal.classList.remove("hidden");
    if (input && input.value.trim().length < 2) {
      renderEmptyState();
    }
    setTimeout(() => input && input.focus(), 50);
  };

  const closeModal = () => {
    modal.classList.add("hidden");
  };

  if (openBtn) openBtn.addEventListener("click", openModal);
  if (closeBtn) closeBtn.addEventListener("click", closeModal);
  if (backdrop) backdrop.addEventListener("click", closeModal);

  if (form && input) {
    form.addEventListener("submit", () => {
      NontonStore.addRecentSearch(input.value);
    });
  }

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
        renderEmptyState();
        return;
      }
      resultsBox.innerHTML = `<div class="py-8 text-center font-mono text-xs text-[#009AD0]">MEMINDAI INDEKS "${q.toUpperCase()}"...</div>`;
      timer = setTimeout(async () => {
        try {
          const res = await fetch(`/api/search?q=${encodeURIComponent(q)}`);
          const data = await res.json();
          if (!data.results || data.results.length === 0) {
            resultsBox.innerHTML = `<div class="py-8 text-center text-xs text-[#FBF9ED]/70">Tidak ada hasil untuk "${q}". Tekan Enter untuk pencarian penuh.</div>`;
            return;
          }
          resultsBox.innerHTML =
            `<a href="/search?q=${encodeURIComponent(q)}" class="flex items-center justify-between border border-[#009AD0] bg-[#009AD0]/15 px-3.5 py-2.5 text-xs font-bold text-[#FBF9ED] hover:bg-[#009AD0]/25 transition-colors">
              <span>Buka seluruh hasil indeks untuk "${q}"</span>
              <span class="font-mono text-[#009AD0]">ENTER</span>
            </a>` +
            data.results
              .map(
                (item, idx) => `
              <div class="flex items-center justify-between gap-3 bg-[#0F1E2B] border border-[#FBF9ED]/12 p-2.5 hover:border-[#009AD0] transition-all">
                <a href="/${item.media_type}/${item.id}" class="flex items-center gap-3 flex-1 min-w-0">
                  <img src="${item.poster_url}" alt="${item.title}" onerror="this.src='https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?w=600'" class="h-14 w-10 object-cover shrink-0 bg-[#060D13] border border-[#FBF9ED]/15" />
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2 font-mono text-[10px]">
                      <span class="text-[#009AD0] font-bold">IDX-${String(idx + 1).padStart(2, "0")}</span>
                      <span class="bg-[#060D13] border border-[#FBF9ED]/20 px-1.5 py-0.5 uppercase text-[#FBF9ED]">${item.media_type}</span>
                      <span class="text-[#FBF9ED]">SCORE ${item.rating}</span>
                      <span class="text-[#FBF9ED]/65">${item.release_year}</span>
                    </div>
                    <h4 class="text-sm font-bold text-[#FBF9ED] truncate mt-0.5">${item.title}</h4>
                  </div>
                </a>
                <a href="${item.media_type === "movie" ? `/watch/movie/${item.id}` : item.media_type === "tv" ? `/watch/tv/${item.id}/1/1` : `/watch/anime/${item.id}/1`}" class="inline-flex items-center gap-1.5 bg-[#BE432C] px-3 py-2 text-xs font-bold text-[#FBF9ED] hover:opacity-95 shrink-0">
                  ${SVG_PLAY} <span>Putar</span>
                </a>
              </div>
            `
              )
              .join("");
        } catch {
          resultsBox.innerHTML = `<div class="py-8 text-center text-xs text-[#BE432C]">Gagal memuat indeks pencarian.</div>`;
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
        d.className = "h-2 w-10 bg-[#BE432C] transition-all";
      } else {
        d.className =
          "h-2 w-3 bg-[#FBF9ED]/30 hover:bg-[#009AD0] transition-all";
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
      .map((item, idx) => {
        const href =
          item.media_type === "movie"
            ? `/watch/movie/${item.id}`
            : item.media_type === "tv"
            ? `/watch/tv/${item.id}/${item.season || 1}/${item.episode || 1}`
            : `/watch/anime/${item.id}/${item.episode || 1}`;
        const label =
          item.media_type === "movie"
            ? "FULL FEATURE"
            : item.media_type === "tv"
            ? `S${item.season || 1} • EP ${item.episode || 1}`
            : `EPISODE ${item.episode || 1}`;

        return `
          <a href="${href}" class="poster-card group relative w-[270px] shrink-0 overflow-hidden block">
            <div class="relative aspect-video w-full overflow-hidden bg-[#060D13]">
              <img src="${item.backdrop_url || item.poster_url}" alt="${item.title}" onerror="this.src='https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?w=600'" class="poster-img h-full w-full object-cover" />
              <div class="absolute inset-0 bg-gradient-to-t from-[#060D13] via-[#060D13]/30 to-transparent"></div>
              <div class="absolute top-2 left-2.5 bg-[#060D13]/90 border border-[#FBF9ED]/20 px-2 py-0.5 font-mono text-[10px] text-[#009AD0]">
                LOG-${String(idx + 1).padStart(2, "0")}
              </div>
              <div class="absolute bottom-2 left-3 right-3 flex items-center justify-between font-mono text-xs text-[#FBF9ED]">
                <span>${label}</span>
                <span class="text-[#009AD0] font-bold">LANJUTKAN</span>
              </div>
              <div class="absolute bottom-0 inset-x-0 h-1 bg-[#060D13]">
                <div class="h-full w-2/3 bg-[#BE432C]"></div>
              </div>
            </div>
            <div class="p-3 border-t border-[#FBF9ED]/10">
              <h4 class="truncate text-sm font-bold text-[#FBF9ED] group-hover:text-[#009AD0]">${item.title}</h4>
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
      btn.innerHTML = saved
        ? `${SVG_CHECK} <span>Tersimpan</span>`
        : `${SVG_PLUS} <span>Simpan Indeks</span>`;
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
  track.scrollBy({
    left: dir === "left" ? -amount : amount,
    behavior: "smooth",
  });
};
