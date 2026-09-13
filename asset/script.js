function indexMain() {
  const params = new URLSearchParams(window.location.search);
  if (!params.has("tag")) return;

  const tag = params.get("tag");

  // タグフィルタ表示
  const a = document.createElement("a");
  a.className = "tag";
  a.href = `index.html?tag=${encodeQueryValue(tag)}`;
  a.textContent = tag;
  document.getElementById("tag-filter").appendChild(a);

  // タグに一致しないカードを非表示
  document.querySelectorAll("#contents-list .card").forEach((card) => {
    const tags = [...card.querySelectorAll(".card-tags .tag")].map((a) => a.textContent);
    if (!tags.includes(tag)) card.hidden = true;
  });

  // タグセクションを非表示
  document.getElementById("tags-section").hidden = true;
}

async function decryptPage() {
  try {
    const pwd = document.getElementById("decrypt-key").value;
    const plain = await decrypt(document.body.dataset.cypher, pwd);
    document.getElementById("article").innerHTML = plain;
  } catch (e) {
    const err = document.getElementById("decrypt-error");
    if (e.name === "OperationError") {
      err.textContent = "パスワードが違います。";
    } else {
      err.textContent = "復号できませんでした。";
    }
  }
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
// Search
//-----------------------------------------------------

// 検索インデックス用にテキストをトークンへ分割する。
//
// - ASCII 英数字の連続は、そのまま 1 つのトークンにする (例: rust → rust)
// - それ以外の文字の連続は、文字バイグラムにする (例: 検索語 → 検索, 索語)
//
function tokenize(text) {
  // 0: 区切り文字, 1: ASCII 英数字, 2: それ以外の文字 (日本語など)
  const classOf = (c) =>
    !/[\p{L}\p{N}]/u.test(c) ? 0 : c.charCodeAt(0) < 128 ? 1 : 2;

  const tokens = [];
  let run = [];
  let cur = 0;

  const flush = () => {
    if (run.length === 0) return;
    if (cur === 1 || run.length === 1) {
      // 1 文字しかない run はバイグラムを作れないので、その文字自体をトークンにする
      tokens.push(run.join("").toLowerCase());
    } else {
      for (let i = 0; i + 2 <= run.length; i++) {
        tokens.push(run.slice(i, i + 2).join("").toLowerCase());
      }
    }
    run = [];
  };

  // 末尾の区切り文字は、最後の run を掃き出すための番兵
  for (const c of text + " ") {
    const cls = classOf(c);
    if (cls !== cur) {
      flush();
      cur = cls;
    }
    if (cls !== 0) run.push(c);
  }

  return tokens;
}

// Bloom filter が word を含むかを調べる (偽陽性あり)
function contains(filter, word) {
  const num_bit = filter.bits.byteLength * 8;

  // ハッシュ関数が 0 個の場合は未ヒットとみなす
  if (filter.num_hash === 0) return false;

  return fxhash32_multi(word, filter.num_hash)
    .map((h) => h % num_bit)
    .every((h) => filter.bits[(h / 8) | 0] & (1 << h % 8));
}

function search(query) {
  const terms = [...new Set(tokenize(query))];
  if (terms.length === 0) return [];

  // BLOOM_FILTER と METADATA は同じ順序で並んでおり、添字が同じ要素が
  // 同じ記事を指す。生成側が 1 つの配列から順に書き出すことが根拠。
  const filters = BLOOM_FILTER.map((bf) => ({
    bits: b64ToU8Arr(bf.filter),
    num_hash: bf.num_hash,
  }));

  // 語ごとに、どのページにヒットするかを調べる
  const hits = terms.map((t) => filters.map((f) => contains(f, t)));

  // ヒットしたページ数から IDF を求める。
  // 多くのページに出る語 (「の」など) ほど重みが小さくなる。
  const num_page = filters.length;
  const idf = hits.map((h) => {
    const df = h.filter(Boolean).length;
    return Math.log((num_page - df + 0.5) / (df + 0.5) + 1);
  });

  const total_idf = idf.reduce((a, b) => a + b, 0);
  if (total_idf === 0) return [];

  return filters
    .flatMap((_, i) => {
      const score = idf.reduce((acc, w, t) => (hits[t][i] ? acc + w : acc), 0);
      if (score === 0) return [];
      return {
        title: METADATA[i].title,
        path: METADATA[i].path,
        rate: score / total_idf,
      };
    })
    .sort((a, b) => b.rate - a.rate);
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
  Promise.all(scripts.map(loadScriptLazily)).then(callback);
}

let debounceTimer;
function searchAndRender() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    const query = document.getElementById("search-input").value;

    const path_to_root =
      document.head.querySelector('meta[name="path_to_root"]').content ?? "";
    const metadata_path = `${path_to_root}/metadata.js`;
    const filter_path = `${path_to_root}/bloom_filter.js`;
    loadScripts([metadata_path, filter_path], () => {
      debounceTimer = null;

      // 記事のタイトルとパスは利用者が書いた文字列なので、innerHTML には
      // 渡さない。textContent と href への代入はブラウザ側が扱う。
      const hits = search(query).map((r) => {
        const link = document.createElement("a");
        link.href = `${path_to_root}/${r.path}`;
        link.textContent = r.title;

        const meta = document.createElement("span");
        meta.className = "search-result-meta";
        meta.textContent = `Match rate: ${r.rate.toFixed(2)}`;

        const hit = document.createElement("div");
        hit.className = "search-hit";
        hit.append(link, meta);
        return hit;
      });

      // 結果が無いときは中身が空になり、#search-result:empty が隠す。
      document.getElementById("search-result").replaceChildren(...hits);
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
// 出力の並び: salt(16) || 反復回数 (4, ビッグエンディアン) || nonce(12) || 暗号文 + タグ
async function decrypt(blobB64, pwd) {
  const blob = b64ToU8Arr(blobB64);
  const salt = blob.slice(0, 16);
  const iterations = new DataView(blob.buffer).getUint32(16, false);
  const nonce = blob.slice(20, 32);
  const cypher = blob.slice(32);

  const material = await crypto.subtle.importKey(
    "raw", new TextEncoder().encode(pwd), "PBKDF2", false, ["deriveKey"],
  );
  const aesKey = await crypto.subtle.deriveKey(
    { name: "PBKDF2", salt, iterations, hash: "SHA-256" },
    material,
    { name: "AES-GCM", length: 256 },
    false,
    ["decrypt"],
  );
  const plain = await crypto.subtle.decrypt({ name: "AES-GCM", iv: nonce }, aesKey, cypher);
  return new TextDecoder().decode(plain);
}

//-----------------------------------------------------
// Misc
//-----------------------------------------------------

function b64ToU8Arr(b64) {
  return Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
}

// タグ名をクエリ文字列の値に変換する。
//
// クエリの区切りに使われる文字と制御文字だけを UTF-8 のバイト列にして
// %XX に直し、それ以外はそのまま残す。日本語が読める形で URL に出る。
//
// 注意: Rust 側の encode_query_value() と同じ規則である必要がある。
// 片方だけを変更するとタグの絞り込みが一致しなくなる。
function encodeQueryValue(str) {
  const metaChars = '&#+%= "<>`';
  let out = "";
  for (const c of str) {
    const code = c.codePointAt(0);
    const isControl = code <= 0x1f || (code >= 0x7f && code <= 0x9f);
    if (metaChars.includes(c) || isControl) {
      for (const b of new TextEncoder().encode(c)) {
        out += `%${b.toString(16).toUpperCase().padStart(2, "0")}`;
      }
    } else {
      out += c;
    }
  }
  return out;
}
