function createTagElem(tagName) {
  const a = document.createElement("a");
  a.className = "tag";
  a.href = `index.html?tag=${tagName}`;
  a.innerHTML = tagName;
  return a;
}

function toggleSearchInput() {
  const searchbar = document.getElementById("searchbar");
  const searchResult = document.getElementById("search-result");
  const toggleButton = document.getElementById("search-toggle");
  const isHidden = searchbar.classList.toggle("hidden");

  if (isHidden) {
    searchResult.classList.add("hidden");
    searchResult.innerHTML = "";
  } else {
    searchResult.classList.remove("hidden");
    document.getElementById("search-input").focus();
  }

  if (toggleButton) {
    toggleButton.setAttribute("aria-expanded", String(!isHidden));
    toggleButton.classList.toggle("is-active", !isHidden);
  }
}

function createCard(page) {
  const template = document.getElementById("card-template");

  const content = template.content.cloneNode(true);
  const card = content.querySelector(".card");
  card.href = page.path;
  if (page.path.startsWith("private/")) card.classList.add("crypto");

  content.querySelector(".card-header").innerHTML = page.title;
  content.querySelector(".card-date").innerHTML = page.update;

  const tags = content.querySelector(".card-tags");
  tags.innerHTML = "";
  page.tags.forEach((tagName) => tags.appendChild(createTagElem(tagName)));

  return content;
}

function renderPageList() {
  const fragment = document.createDocumentFragment();

  METADATA
    .filter((page) => !page.is_sub)
    .forEach((page) => fragment.appendChild(createCard(page)));

  document.getElementById("contents-list").appendChild(fragment);
}

function renderTagSet() {
  const tagSet = new Set(
    Object.keys(METADATA)
      .map((key) => METADATA[key].tags)
      .flat(),
  );

  const fragment = document.createDocumentFragment();
  tagSet.forEach((tagName) => {
    fragment.appendChild(createTagElem(tagName));
    fragment.appendChild(document.createTextNode(" "));
  });

  document.getElementById("tags-list").appendChild(fragment);
}

function indexMain() {
  const params = new URLSearchParams(window.location.search);
  if (!params.has("tag")) {
    renderPageList();
    renderTagSet();
  } else {
    const tag = params.get("tag");
    const tagElem = createTagElem(tag);

    document.getElementById("tag-filter").appendChild(tagElem);

    const fragment = document.createDocumentFragment();
    METADATA.filter((page) => page.tags.includes(tag)).forEach((page) =>
      fragment.appendChild(createCard(page)),
    );

    document.getElementById("contents-list").appendChild(fragment);
  }
}

async function decryptPage() {
  const pwd = document.getElementById("decrypt-key").value;
  const key = await crypto.subtle.digest(
    "SHA-256",
    Uint8Array.from(pwd, (c) => c.charCodeAt(0)),
  );

  const ivCypher = document.body.dataset.cypher;
  const plain = await decrypt(ivCypher, key);
  document.getElementById("main-content").innerHTML = plain;
}

function cryptoMain() {
  document
    .getElementById("decrypt-key")
    .addEventListener("keydown", async (e) => {
      if (e.key === "Enter") await decryptPage();
    });

  document
    .getElementById("decrypt-btn")
    .addEventListener("click", async (e) => {
      await decryptPage();
    });
}

//-----------------------------------------------------
// Theme
//-----------------------------------------------------

function toggleTheme() {
  const currentTheme = document.documentElement.getAttribute("theme");
  setTheme(currentTheme === "dark" ? "" : "dark");
}

//-----------------------------------------------------
// Search
//-----------------------------------------------------

// (json, Set<string>) -> integer
function hitRate(bloom_filter, words) {
  if (words.size == 0) return 0;

  const filter = b64ToU8Arr(bloom_filter.filter);
  const num_hash = bloom_filter.num_hash;
  const num_bit = filter.byteLength * 8;

  let num_hit_word = 0;
  for (const word of words) {
    const hashes = fxhash32_multi(word, num_hash).map((h) => h % num_bit);
    const hit = hashes.every((h) => filter[(h / 8) | 0] & (1 << h % 8));
    if (hit) num_hit_word += 1;
  }

  return num_hit_word / words.size;
}

function search(query) {
  if (!query) return [];

  const words = new Set(
    segment(query).flatMap((w) => (w.trim() ? w.trim().toLowerCase() : [])),
  );

  return BLOOM_FILTER.flatMap((bf, i) => {
    const r = hitRate(bf, words);
    if (r == 0) return [];
    return {
      title: METADATA[i].title,
      path: METADATA[i].path,
      rate: r,
    };
  }).sort((a, b) => b.rate - a.rate);
}

function loadScriptLazily(script_path) {
  return new Promise((resolve, reject) => {
    const loaded =
      document.querySelector(`script[src="${script_path}"]`) !== null;
    if (loaded) {
      resolve();
      return;
    }

    const script = document.createElement("script");
    script.src = script_path;
    script.onload = resolve;
    script.onerror = () =>
      reject(new Error(`Failed to load script: ${script_path}`));

    document.head.appendChild(script);
  });
}

function loadScripts(scripts, callback) {
  // すべてのスクリプトが読み込まれた後にコールバックを呼び出す
  Promise.all(scripts.map(loadScriptLazily)).then(() => {
    callback();
  });
}

let debounceTimer;
function searchAndRender() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    const query = document.getElementById("search-input").value;

    const path_to_root =
      document.head.querySelector('meta[name="path_to_root"]').content ?? "";
    const segmenter_path = `${path_to_root}/segmenter.js`;
    const filter_path = `${path_to_root}/bloom_filter.js`;
    loadScripts([segmenter_path, filter_path], () => {
      debounceTimer = null;

      const result = search(query);
      const html = result
        .map((r) => {
          const path = r.path;
          const rate = r.rate.toFixed(2);
          return `<div class="search-hit"><a href="${path_to_root}/${path}">${r.title}</a><span class="search-result-meta">Match rate: ${rate}</span></div>`;
        })
        .join("");
      document.getElementById("search-result").innerHTML = html;
    });
  }, 300);
}

//-----------------------------------------------------
// Main
//-----------------------------------------------------

window.addEventListener("DOMContentLoaded", () => {
  switch (document.body.dataset.page) {
    case "index":
      indexMain();
      break;
    case "crypto":
      cryptoMain();
      break;
  }
});

//-----------------------------------------------------
// Hash
//-----------------------------------------------------

// string -> bigint (64-bit)
function fxhash64(str) {
  const SEED = 0x517cc1b727220a95n;
  let v = 0n;
  for (const c of new TextEncoder().encode(str)) {
    v = (v << 5n) | (v >> 59n); // rotate left 5
    v ^= BigInt(c);
    v *= SEED;
    v &= 0xffffffffffffffffn; // 64-bit に切り詰める
  }
  return v;
}

// str に対応する n 個のハッシュ値を返す
// (string, number) -> [number (32-bit); n]
function fxhash32_multi(str, n) {
  const hash64 = fxhash64(str);
  const h1 = Number(hash64 & 0xffffffffn);
  const h2 = Number(hash64 >> 32n);
  // INFO: 論理右シフトは符号なし 32 ビット整数を返す
  return Array.from({ length: n }, (_, i) => (h1 + h2 * i) >>> 0);
}

//-----------------------------------------------------
// Crypto
//-----------------------------------------------------

// (string, string) -> string
async function decrypt(ivCypher, key) {
  ivCypher = b64ToU8Arr(ivCypher);
  const iv = ivCypher.slice(0, 16);
  const cypher = ivCypher.slice(16);

  const aesKey = await crypto.subtle.importKey(
    "raw",
    key,
    { name: "AES-CBC" },
    false,
    ["decrypt"],
  );

  const plain = await crypto.subtle.decrypt(
    { name: "AES-CBC", iv: iv },
    aesKey,
    cypher,
  );

  return new TextDecoder().decode(plain);
}

//-----------------------------------------------------
// Misc
//-----------------------------------------------------

function b64ToU8Arr(b64) {
  return Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
}
