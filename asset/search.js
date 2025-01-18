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

// (json, Set<string>) -> integer
function hitRate(bloom_filter, words) {
  if (words.size == 0) return 0;

  // base64 -> Uint8Array
  const filter = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));

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
    segment(query).flatMap((w) => (w.trim() ? w.toLowerCase() : [])),
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

function searchAndRenderCore(query) {
  const path_to_root = document.head.querySelector('meta[name="path_to_root"]').content ?? "";
  const html = search(query)
    .map((r) => {
      return `<div><a href="${path_to_root}/${r.path}">${r.title}</a><span style="color:gray;margin-left:1em;">MatchRate:${r.rate}</span></div>`;
    })
    .join("");
  document.getElementById("search-result").innerHTML = html;
}
