function createTagElem(tagName) {
  const a = document.createElement("a");
  a.className = "tag";
  a.href = `tag.html?tag=${tagName}`;
  a.innerHTML = tagName;
  return a;
}

function toggleSearchInput() {
  document.getElementById("searchbar").classList.toggle("hidden");
}

function createCard(page) {
  const template = document.getElementById('card-template');

  const content = template.content.cloneNode(true);
  const card = content.querySelector('.card');
  card.href = page.path;
  if (page.flags.includes('crypto')) card.classList.add('crypto');

  content.querySelector('.card-header').innerHTML = page.title;
  content.querySelector('.card-date').innerHTML = page.update;

  const tags = content.querySelector('.card-tags');
  tags.innerHTML = '';
  page.tags.forEach(tagName => tags.appendChild(createTagElem(tagName)));

  return content;
}

function tagMain() {
  const params = new URLSearchParams(window.location.search);
  if (!params.has("tag")) return;
  const tag = params.get("tag");
  const tagElem = createTagElem(tag);

  document.getElementById("tag-title").appendChild(tagElem);
  document.getElementsByTagName("title")[0].innerHTML = `Filter: ${tag}`;

  const fragment = document.createDocumentFragment();
  METADATA.filter((page) => page.tags.includes(tag))
    .forEach((page) => fragment.appendChild(createCard(page)));

  document.getElementById("contents-list").appendChild(fragment);
}

function renderPageList() {
  const fragment = document.createDocumentFragment();

  METADATA.forEach((page) =>
    fragment.appendChild(createCard(page)),
  );

  document.getElementById("contents-list").appendChild(fragment);
}

function renderTagSet() {
  const tagSet = new Set(
    Object.keys(METADATA)
      .map((key) => METADATA[key].tags)
      .flat(),
  );

  const fragment = document.createDocumentFragment();
  tagSet.forEach(tagName => {
    fragment.appendChild(createTagElem(tagName));
    fragment.appendChild(document.createTextNode(" "));
  });

  document.getElementById("tags-list").appendChild(fragment);
}

function indexMain() {
  renderPageList();
  renderTagSet();
}

async function decryptPage() {
  const pwd = document.getElementById("decrypt-key").value;
  const key = await crypto.subtle.digest('SHA-256', Uint8Array.from(pwd, c => c.charCodeAt(0)));

  const ivCypher = document.body.dataset.cypher;
  const plain = await decrypt(ivCypher, key);
  document.getElementById('main-content').innerHTML = plain;
}

function cryptoMain() {
  document.getElementById("decrypt-key")
    .addEventListener("keydown", async (e) => {
      if (e.key === "Enter") await decryptPage();
    });

  document.getElementById("decrypt-btn")
    .addEventListener("click", async (e) => {
      await decryptPage();
    });
}

// (string) -> [string]
function tokenize(text) {
  if (!("Segmenter" in Intl)) {
    alert("このブラウザはSegmenterをサポートしていません。");
    return null;
  }

  const segmenter = new Intl.Segmenter("ja", { granularity: "word" });
  const segments = segmenter.segment(text)[Symbol.iterator]();
  return Array.from(segments.map(s => s.segment));
}

//-----------------------------------------------------
// Theme
//-----------------------------------------------------

function toggleTheme() {
  const currentTheme = document.documentElement.getAttribute('theme');
  setTheme(currentTheme === 'dark' ? '' : 'dark');
}

//-----------------------------------------------------
// Search
//-----------------------------------------------------

// (json, Set<string>) -> json
function searchPage(meta, words) {
  const filter = b64ToU8Arr(meta.bloom_filter);
  const num_hash = meta.bloom_num_hash;
  const num_bit = filter.byteLength * 8;

  let num_hit_word = 0;
  for (const word of words) {
    const hashes = fxhash32_multi(word, num_hash).map((h) => h % num_bit);
    const hit = hashes.every(h => filter[(h / 8) | 0] & (1 << h % 8));
    if (hit) num_hit_word += 1;
  }

  return {
    title: meta.title,
    path: meta.path,
    rate: words.size === 0 ? 0 : num_hit_word / words.size,
  };
}

function search(query) {
  if (!query) return [];

  const words = new Set(tokenize(query).flatMap(w => w.trim() ? w.toLowerCase() : []));

  return METADATA
    .flatMap((m) => {
      const r = searchPage(m, words);
      return r.rate ? r : [];
    })
    .sort((a, b) => b.rate - a.rate);
}

let debounceTimer;
function searchAndRender() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    const query = document.getElementById("search-input").value;

    const result = search(query);
    const path_to_root = document.head.querySelector('meta[name="path_to_root"]').content ?? "";
    const html = result
      .map((r) => {
        const path = r.path;
        return `<div><a href="${path_to_root}/${path}">${r.title}</a><span style="color:gray;margin-left:1em;">MatchRate:${r.rate}</span></div>`;
      })
      .join("");

    document.getElementById("search-result").innerHTML = html;
  }, 300);
}

//-----------------------------------------------------
// Main
//-----------------------------------------------------

window.addEventListener("DOMContentLoaded", () => {
  switch (document.body.dataset.page) {
    case "tag":
      tagMain();
      break;
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
      'raw',
      key,
      { name: 'AES-CBC' },
      false,
      ['decrypt'],
    );

  const plain = await crypto
    .subtle
    .decrypt({ name: 'AES-CBC', iv: iv}, aesKey, cypher);

  return new TextDecoder().decode(plain);
}

//-----------------------------------------------------
// Misc
//-----------------------------------------------------

function b64ToU8Arr(b64) {
  return Uint8Array.from(atob(b64), c => c.charCodeAt(0));
}
