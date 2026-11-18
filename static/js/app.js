/**
 * N' — Nonton Creative Streaming Studio (Exact noth.in Blueprint)
 * 1. Full Splash Screen Preloader (N + 3D Balloon Dog + ' + 000->100 counter + Notice Card)
 * 2. Sticky 100vh White Hero with Giant NONTON' Scroll Zoom-Through
 * 3. Lenis Smooth Inertial Scroll + Asymmetric 6-Column Works Parallax
 * 4. Glitch Text-Scramble Manifesto + Monumental Footer Letter Scaling
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
      badge.textContent = `(${count})`;
      badge.classList.remove("hidden");
    } else {
      badge.classList.add("hidden");
    }
  },
};

window.NontonStore = NontonStore;

let lenisInstance = null;
let splashRafId = null;

document.addEventListener("DOMContentLoaded", () => {
  NontonStore.updateNavbarBadge();
  initNothSplashLoader(false);
  initLenisAndScrollChoreography();
  initNothMenuDrawer();
  initGlitchTextScramble();
  initSearchModal();
  initHeroCarousel();
  initContinueWatchingRow();
  initWatchlistButtons();
});

/* =========================================================
   1. NOTH.IN SPLASH SCREEN / PRELOADER (000 -> 100)
   ========================================================= */
const SPLASH_SHOWN_KEY = "noth_splash_shown";

function completeSplashReveal() {
  const loader = document.getElementById("noth-loader");
  const counterEl = document.getElementById("noth-loader-number");
  const cardEl = document.getElementById("noth-splash-card");
  const htmlEl = document.documentElement;

  if (splashRafId) {
    cancelAnimationFrame(splashRafId);
    splashRafId = null;
  }
  try {
    sessionStorage.setItem(SPLASH_SHOWN_KEY, "1");
  } catch (e) {}

  if (counterEl) counterEl.textContent = "100";
  if (cardEl) cardEl.classList.add("is-hidden");

  setTimeout(() => {
    if (loader) loader.classList.add("is-done");
    htmlEl.classList.remove("lenis-stopped");
    document.body.classList.add("splash-ready");
    if (lenisInstance) lenisInstance.start();
  }, 180);
}

window.finishNothSplashNow = function () {
  completeSplashReveal();
};

window.dismissSplashCard = function () {
  const cardEl = document.getElementById("noth-splash-card");
  if (cardEl) cardEl.classList.add("is-hidden");
};

function initNothSplashLoader(forceFull = false) {
  const loader = document.getElementById("noth-loader");
  const counterEl = document.getElementById("noth-loader-number");
  const cardEl = document.getElementById("noth-splash-card");
  const htmlEl = document.documentElement;

  if (!loader || !counterEl) {
    document.body.classList.add("splash-ready");
    htmlEl.classList.remove("lenis-stopped");
    return;
  }

  let alreadyShown = false;
  try {
    alreadyShown = sessionStorage.getItem(SPLASH_SHOWN_KEY) === "1";
  } catch (e) {}

  // Only show splash screen on the very first visit to the website (unless manually replayed)
  if (alreadyShown && !forceFull) {
    htmlEl.classList.add("splash-seen");
    htmlEl.classList.remove("lenis-stopped");
    document.body.classList.add("splash-ready");
    loader.classList.add("is-done");
    return;
  }

  // Mark as shown immediately so navigating away even mid-load won't re-trigger splash
  try {
    sessionStorage.setItem(SPLASH_SHOWN_KEY, "1");
  } catch (e) {}

  htmlEl.classList.remove("splash-seen");
  if (cardEl) cardEl.classList.remove("is-hidden");

  const duration = 2100;

  loader.classList.remove("is-done");
  document.body.classList.remove("splash-ready");
  htmlEl.classList.add("lenis-stopped");
  if (lenisInstance) lenisInstance.stop();

  const startTime = performance.now();

  function tick(now) {
    const elapsed = now - startTime;
    const rawProgress = Math.min(1, elapsed / duration);
    const eased =
      rawProgress < 0.5
        ? 4 * rawProgress * rawProgress * rawProgress
        : 1 - Math.pow(-2 * rawProgress + 2, 3) / 2;
    const currentNum = Math.min(100, Math.floor(eased * 100));

    counterEl.textContent = String(currentNum).padStart(3, "0");

    if (rawProgress < 1) {
      splashRafId = requestAnimationFrame(tick);
    } else {
      completeSplashReveal();
    }
  }

  splashRafId = requestAnimationFrame(tick);
}

window.replayNothSplash = function () {
  const drawer = document.getElementById("noth-menu-drawer");
  const menuBtn = document.getElementById("noth-menu-btn");
  if (drawer && !drawer.classList.contains("hidden")) {
    drawer.classList.add("hidden");
    drawer.classList.remove("flex");
    if (menuBtn) menuBtn.classList.remove("is-open");
  }
  window.scrollTo({ top: 0, behavior: "instant" });
  initNothSplashLoader(true);
};

/* =========================================================
   2. LENIS SMOOTH INERTIAL SCROLL & PARALLAX CHOREOGRAPHY
   ========================================================= */
function initLenisAndScrollChoreography() {
  if (typeof window.Lenis !== "undefined") {
    lenisInstance = new window.Lenis({
      lerp: 0.085,
      smoothWheel: true,
      wheelMultiplier: 1.0,
    });

    function raf(time) {
      lenisInstance.raf(time);
      requestAnimationFrame(raf);
    }
    requestAnimationFrame(raf);
  }

  const isHomePage = window.location.pathname === "/";
  const headerLogo = document.getElementById("header-n-logo");
  const heroTrack = document.getElementById("hero-scroll-track");
  const wordmark = document.getElementById("hero-zoom-wordmark");
  const showreelCard = document.getElementById("hero-showreel-card");
  const parallaxImgs = document.querySelectorAll("[data-parallax-img]");
  const footerWordmark = document.querySelector("[data-footer-wordmark]");
  const footerLetters = footerWordmark
    ? footerWordmark.querySelectorAll(".noth-footer-letter")
    : [];

  function updateScrollTransforms() {
    const scrollY = window.scrollY || window.pageYOffset;
    const vh = window.innerHeight || 900;

    // 1. Hide top-left N' logo at y=0 on homepage so Hero tagline sits cleanly in top-left (like step_89.png)
    if (headerLogo) {
      if (isHomePage && scrollY < vh * 0.45) {
        headerLogo.style.opacity = "0";
        headerLogo.style.pointerEvents = "none";
      } else {
        headerLogo.style.opacity = "1";
        headerLogo.style.pointerEvents = "auto";
      }
    }

    // 2. Hero Sticky Giant NONTON' Wordmark Zoom-Through (step_89 -> step_6)
    if (wordmark && heroTrack) {
      const maxScroll = Math.max(1, heroTrack.offsetHeight - vh);
      const progress = Math.min(1, Math.max(0, scrollY / maxScroll));
      // Exponential zoom from scale(1) to scale(5.8) matching step_6_shot_1_hero_desktop_1440.png
      const scale = 1 + Math.pow(progress, 1.35) * 4.8;
      wordmark.style.transform = `scale(${scale.toFixed(3)})`;
    }

    // 3. Showreel Card Subtle Scale-In on Scroll
    if (showreelCard) {
      const rect = showreelCard.getBoundingClientRect();
      const centerProgress = Math.min(1, Math.max(0, (vh - rect.top) / (vh * 0.9)));
      const cardScale = 0.95 + centerProgress * 0.05;
      showreelCard.style.transform = `scale(${cardScale.toFixed(3)})`;
    }

    // 4. Internal Image Parallax inside Asymmetric Works Cards
    parallaxImgs.forEach((img) => {
      const frame = img.parentElement;
      if (!frame) return;
      const rect = frame.getBoundingClientRect();
      if (rect.bottom < 0 || rect.top > vh) return;
      const ratio = (rect.top + rect.height * 0.5 - vh * 0.5) / vh;
      const shiftY = Math.max(-9, Math.min(9, ratio * -13));
      img.style.transform = `translate3d(0, ${shiftY.toFixed(2)}%, 0) scale(1.04)`;
    });

    // 5. Giant Footer Letters Independent Scale Reveal
    if (footerWordmark && footerLetters.length > 0) {
      const rect = footerWordmark.getBoundingClientRect();
      const visibleRatio = Math.min(1, Math.max(0, (vh - rect.top) / (rect.height + vh * 0.25)));
      footerLetters.forEach((letter, idx) => {
        const stagger = idx * 0.05;
        const localP = Math.min(1, Math.max(0.2, (visibleRatio - stagger) / (1 - stagger)));
        letter.style.transform = `scaleY(${localP.toFixed(3)})`;
      });
    }
  }

  window.addEventListener("scroll", updateScrollTransforms, { passive: true });
  window.addEventListener("resize", updateScrollTransforms);
  updateScrollTransforms();
}

/* =========================================================
   3. NOTH.IN MENU DRAWER TOGGLE
   ========================================================= */
function initNothMenuDrawer() {
  const btn = document.getElementById("noth-menu-btn");
  const drawer = document.getElementById("noth-menu-drawer");
  if (!btn || !drawer) return;

  btn.addEventListener("click", () => {
    const isHidden = drawer.classList.contains("hidden");
    if (isHidden) {
      drawer.classList.remove("hidden");
      drawer.classList.add("flex");
      btn.classList.add("is-open");
    } else {
      drawer.classList.add("hidden");
      drawer.classList.remove("flex");
      btn.classList.remove("is-open");
    }
  });
}

/* =========================================================
   4. GLITCH TEXT-SCRAMBLE MANIFESTO DECODER
   ========================================================= */
function initGlitchTextScramble() {
  const el = document.getElementById("noth-glitch-manifesto");
  if (!el) return;

  const finalText = el.dataset.glitchTarget || el.textContent.trim();
  const glyphs = "Pj(è!?”U§hs·01/_[]{}*#@%&";
  let isAnimating = false;

  const triggerScramble = () => {
    if (isAnimating) return;
    isAnimating = true;
    let iteration = 0;
    const totalSteps = finalText.length;

    const interval = setInterval(() => {
      el.textContent = finalText
        .split("")
        .map((char, idx) => {
          if (char === " ") return " ";
          if (idx < iteration) return finalText[idx];
          return glyphs[Math.floor(Math.random() * glyphs.length)];
        })
        .join("");

      iteration += 1.6;
      if (iteration >= totalSteps) {
        el.textContent = finalText;
        clearInterval(interval);
        isAnimating = false;
      }
    }, 24);
  };

  if ("IntersectionObserver" in window) {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            triggerScramble();
          }
        });
      },
      { threshold: 0.35 }
    );
    observer.observe(el);
  }
}

/* =========================================================
   5. INSTANT COMMAND SEARCH MODAL (CTRL + K)
   ========================================================= */
window.openNothSearch = function () {
  const modal = document.getElementById("search-modal");
  const input = document.getElementById("instant-search-input");
  if (!modal) return;
  modal.classList.remove("hidden");
  setTimeout(() => input && input.focus(), 50);
};

function initSearchModal() {
  const modal = document.getElementById("search-modal");
  const openBtn = document.getElementById("open-search-btn");
  const closeBtn = document.getElementById("close-search-btn");
  const backdrop = document.getElementById("search-modal-backdrop");
  const input = document.getElementById("instant-search-input");
  const resultsBox = document.getElementById("instant-search-results");

  if (!modal) return;

  const openModal = () => window.openNothSearch();
  const closeModal = () => modal.classList.add("hidden");

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
        resultsBox.innerHTML = `<div class="py-10 text-center font-mono text-xs text-neutral-500">( Type at least 2 characters to search Movies, Series & Anime )</div>`;
        return;
      }
      resultsBox.innerHTML = `<div class="py-10 text-center font-mono text-xs text-white animate-pulse">( SEARCHING "${q.toUpperCase()}" IN RUST INDEX... )</div>`;
      timer = setTimeout(async () => {
        try {
          const res = await fetch(`/api/search?q=${encodeURIComponent(q)}`);
          const data = await res.json();
          if (!data.results || data.results.length === 0) {
            resultsBox.innerHTML = `<div class="py-10 text-center font-mono text-xs text-neutral-500">( NO WORKS FOUND FOR "${q.toUpperCase()}" )</div>`;
            return;
          }
          resultsBox.innerHTML =
            `<a href="/search?q=${encodeURIComponent(q)}" class="flex items-center justify-between rounded-full border border-white/30 bg-white text-black px-5 py-3 text-xs font-mono font-bold uppercase hover:bg-neutral-200 transition-colors">
              <span>VIEW FULL INDEX RESULTS FOR "${q}"</span>
              <span>ENTER →</span>
            </a>` +
            data.results
              .map(
                (item) => `
              <div class="flex items-center justify-between gap-4 rounded-2xl border border-white/15 bg-neutral-950 p-3 hover:border-white transition-all">
                <a href="/${item.media_type}/${item.id}" class="flex items-center gap-3.5 flex-1 min-w-0">
                  <img src="${item.poster_url}" alt="${item.title}" class="h-14 w-10 rounded-lg object-cover shrink-0 bg-neutral-900" />
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2 font-mono text-[11px] text-neutral-400">
                      <span class="uppercase text-white">(${item.media_type})</span>
                      <span>${item.rating}</span>
                      <span>· ${item.release_year}</span>
                    </div>
                    <h4 class="text-sm font-medium text-white truncate mt-0.5">${item.title}</h4>
                  </div>
                </a>
                <a href="${item.media_type === "movie" ? `/watch/movie/${item.id}` : item.media_type === "tv" ? `/watch/tv/${item.id}/1/1` : `/watch/anime/${item.id}/1`}" class="noth-pill noth-pill-outline-light !py-2 !px-4 !text-[10px] shrink-0">
                  <span>WATCH →</span>
                </a>
              </div>
            `
              )
              .join("");
        } catch {
          resultsBox.innerHTML = `<div class="py-10 text-center font-mono text-xs text-neutral-400">( ERROR FETCHING SEARCH INDEX )</div>`;
        }
      }, 220);
    });
  }
}

/* =========================================================
   6. HERO SPOTLIGHT CAROUSEL & CONTINUE WATCHING
   ========================================================= */
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
        d.className = "h-1.5 w-8 rounded-full bg-white transition-all";
      } else {
        d.className = "h-1.5 w-2 rounded-full bg-white/35 hover:bg-white/60 transition-all";
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
            ? "MOVIE"
            : item.media_type === "tv"
            ? `S${item.season || 1} · EP ${item.episode || 1}`
            : `EPISODE ${item.episode || 1}`;

        return `
          <a href="${href}" class="noth-media-card group w-[280px] shrink-0 block">
            <div class="noth-media-frame aspect-video border border-white/15">
              <img src="${item.backdrop_url || item.poster_url}" alt="${item.title}" class="noth-media-img" />
              <div class="absolute inset-0 bg-gradient-to-t from-black/85 via-black/25 to-transparent"></div>
              <div class="absolute bottom-3 left-3.5 right-3.5 flex items-center justify-between font-mono text-[11px] text-white">
                <span>( ${label} )</span>
                <span>RESUME →</span>
              </div>
            </div>
            <div class="pt-2.5">
              <h4 class="truncate text-sm font-medium text-white group-hover:opacity-70">${item.title}</h4>
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
      btn.textContent = saved ? "IN WATCHLIST" : "+ WATCHLIST";
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
  const section = btn.closest("div.space-y-8") || btn.closest("section");
  const track = section ? section.querySelector("[data-row-track]") : null;
  if (!track) return;
  const amount = track.clientWidth * 0.75;
  track.scrollBy({ left: dir === "left" ? -amount : amount, behavior: "smooth" });
};
