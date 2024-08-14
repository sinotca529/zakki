function createTagElem(tagName) {
  const a = document.createElement("a");
  a.className = "tag";
  a.href = `tag.html?tag=${tagName}`;
  a.innerHTML = tagName;
  return a;
}

function toggleSearchInput() {
  document
    .getElementById('searchbar')
    .classList
    .toggle('hidden');
}

function createCard(page) {
  const header = document.createElement("div");
  header.className = "card-header";
  header.innerHTML = page.title;

  const date = document.createElement("div");
  date.className = "card-date";
  date.innerHTML = page.update;

  const tags = document.createElement("div");
  tags.className = "card-tags";
  page.tags.forEach((tagName) => {
    const tag = createTagElem(tagName);
    tags.appendChild(tag);
  });

  const meta = document.createElement("div");
  meta.className = "card-meta";
  meta.appendChild(date);
  meta.appendChild(tags);

  const card = document.createElement("a");
  card.className = "card";
  card.href = page.path;
  if (page.flags.includes("crypto")) card.classList.add("crypto");
  card.appendChild(header);
  card.appendChild(meta);

  return card;
}

function tagMain() {
  const params = new URLSearchParams(window.location.search);
  if (!params.has("tag")) return;
  const tag = params.get("tag");

  document.getElementById("title").innerHTML = `タグ : ${tag} のついたページ`;
  document.getElementsByTagName("title")[0].innerHTML = `タグ: ${tag}`;

  const fragment = document.createDocumentFragment();
  METADATA.filter((page) => page.tags.includes(tag))
    .sort((a, b) => b.update.localeCompare(a.update))
    .forEach((page) => fragment.appendChild(createCard(page)));

  document.getElementById("contents-list").appendChild(fragment);
}

function renderPageList() {
  const fragment = document.createDocumentFragment();

  METADATA.sort((a, b) => b.update.localeCompare(a.update)).forEach((page) =>
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
  tagSet.forEach((tagName) => {
    const tag = createTagElem(tagName);
    fragment.appendChild(tag);
    fragment.appendChild(document.createTextNode(" "));
  });

  document.getElementById("tags-list").appendChild(fragment);
}

function indexMain() {
  renderPageList();
  renderTagSet();
}

async function decryptAes256Cbc(data, iv, key) {
  const aesKey = await crypto.subtle.importKey(
    "raw",
    key,
    { name: "AES-CBC" },
    false,
    ["decrypt"],
  );
  return crypto.subtle.decrypt({ name: "AES-CBC", iv: iv }, aesKey, data);
}

async function getAesKey() {
  const key = document.getElementById("keyInput").value;
  const keyData = new TextEncoder().encode(key);
  return await crypto.subtle.digest("SHA-256", keyData);
}

function base64ToUint8Array(base64Str) {
  const raw = atob(base64Str);
  return Uint8Array.from(
    Array.prototype.map.call(raw, (x) => {
      return x.charCodeAt(0);
    }),
  );
}

async function decodeCypher() {
  try {
    const ivCypher = base64ToUint8Array(document.body.dataset.cypher);
    const iv = ivCypher.slice(0, 16);
    const cypher = ivCypher.slice(16);

    let plain = await decryptAes256Cbc(cypher, iv,  await getAesKey());
    plain = new TextDecoder().decode(plain);

    document.documentElement.innerHTML = plain;
  } catch {
    alert("Failed to decrypto");
  }
}

function cryptoMain() {
  document.getElementById("keyInput").onkeydown = (e) => {
    if (e.key === "Enter") decodeCypher();
  };
}

// string -> [string]
function tokenize(text) {
  if (!('Segmenter' in Intl)) {
    alert("このブラウザはSegmenterをサポートしていません。");
    return null;
  }

  const segmenter = new Intl.Segmenter("ja", { granularity: "word" });
  const segments = segmenter.segment(text)[Symbol.iterator]();
  return Array.from(segments.map(s => s.segment));
}

// string -> bigint
function fxhash64(str) {
  const SEED = 0x517cc1b727220a95n;
  let v = 0n;
  for (const c of new TextEncoder().encode(str)) {
    v = (v << 5n) | (v >> 59n); // rotate left 5
    v ^= BigInt(c);
    v *= SEED;
    v &= 0xffffffffffffffffn; // 64 bit に切り詰める
  }
  return v;
}

// str に対応する n 個のハッシュ値を返す
// (string, number) -> [number; n]
function fxhash32_multi(str, n) {
  const hash64 = fxhash64(str);
  const hash1 = Number(hash64 & 0xFFFFFFFFn);
  const hash2 = Number(hash64 >> 32n);

  // INFO: 論理右シフトは符号なし 32 ビット整数を返す
  // see: https://developer.mozilla.org/ja/docs/Web/JavaScript/Reference/Operators/Unsigned_right_shift
  return Array.from({ length: n }, (_, i) => (hash1 + (hash2 * i)) >>> 0);
}

function search(query) {
  if (!query) return [];

  const words = new Set(
    tokenize(query)
      .filter(w => w.trim() !== "")
      .map(w => w.toLowerCase())
  );

  return METADATA
    .map(m => {
      const filter = base64ToUint8Array(m.bloom_filter);
      const num_hash = m.bloom_num_hash;
      const num_bit = filter.byteLength * 8;

      let num_hit_word = 0;
      for (const word of words) {
        const hashes = fxhash32_multi(word, num_hash).map(h => h % num_bit);
        const hit = hashes.every(h => (filter[(h / 8) | 0] & (1 << (h % 8))) != 0);
        if (hit) num_hit_word += 1;
      }

      return {
        title: m.title,
        path: m.path,
        rate: num_hit_word / words.size
      };
    })
    .filter(d => d.rate !== 0)
    .sort((a, b) => b.rate - a.rate);
}

let debounceTimer;
function searchAndRender() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    const query = document
      .getElementById('search_input')
      .value;

    const result = search(query);
    const html = result
      .map(r => `<div><a href="${r.path}">${r.title}</a><span style="color:gray;margin-left:1em;">MatchRate:${r.rate}</span></div>`)
      .join('');

    document.getElementById('search_result').innerHTML = html;
  }, 300);
}

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
