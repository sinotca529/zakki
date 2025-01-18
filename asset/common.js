// util

function loadScriptLazily(script_path) {
  return new Promise((resolve, reject) => {
    const loaded = document.querySelector(`script[src="${script_path}"]`) !== null;
    if (loaded) { resolve(); return; }

    const script = document.createElement("script");
    script.src = script_path;
    script.onload = resolve;
    script.onerror = () => reject(new Error(`Failed to load script: ${script_path}`));

    document.head.appendChild(script);
  });
}

// theme

function toggleTheme() {
  const currentTheme = document.documentElement.getAttribute("theme");
  setTheme(currentTheme === "dark" ? "" : "dark");
}

function setTheme(theme) {
  if (theme === '') document.documentElement.removeAttribute('theme');
  else document.documentElement.setAttribute('theme', 'dark');
  localStorage.setItem('theme', theme);
}

// search

function toggleSearchInput() {
  document.getElementById("searchbar").classList.toggle("hidden");
}

let debounceTimer;
function searchAndRender() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    const path_to_root = document.head.querySelector('meta[name="path_to_root"]').content ?? "";
    const scripts = [
      `${path_to_root}/segmenter.js`,
      `${path_to_root}/bloom_filter.js`,
      `${path_to_root}/search.js`,
    ];
    Promise.all(scripts.map(loadScriptLazily)).then(() => {
      debounseTimer = null;
      const query = document.getElementById("search-input").value;
      searchAndRenderCore(query);
    });
  }, 300);
}

setTheme(localStorage.getItem('theme') ?? '');
window.addEventListener("DOMContentLoaded", () => {
  document.getElementById("theme-toggle").addEventListener("click", toggleTheme);
  document.getElementById("search-input").addEventListener("keyup", searchAndRender);
});
