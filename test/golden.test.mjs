// testdata の表を読み、クライアント側の実装が同じ結果を返すことを確かめます。
// Rust 側にも同じ表を読むテストがあり、両者が同じ表を通ることで規則の一致を保ちます。
//
//   node --test test/

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const readJson = (name) =>
  JSON.parse(readFileSync(join(root, "testdata", name), "utf8"));

// script.js はブラウザ向けに書かれていて、モジュールとして読み込めません。
// 関数の本体として実行し、必要な関数だけを取り出します。
// window と document は最小限のものを引数で渡します。
//
// node:vm を使わないのは、別の realm で作られた配列が deepStrictEqual を
// 通らないためです。Array.prototype が違うものになります。
const load = () => {
  const src = readFileSync(join(root, "asset/script.js"), "utf8");
  const body = `${src}\nreturn { tokenize, fxhash64, encodeQueryValue };`;
  const noop = () => {};
  return new Function("window", "document", body)(
    { addEventListener: noop },
    { addEventListener: noop, getElementById: () => null, body: { dataset: {} } },
  );
};

const { tokenize, fxhash64, encodeQueryValue } = load();

test("tokenize が表と一致する", () => {
  const cases = readJson("tokenize.json");
  assert.ok(cases.length > 0);
  for (const c of cases) {
    assert.deepEqual(tokenize(c.in), c.out, `入力: ${JSON.stringify(c.in)}`);
  }
});

test("fxhash64 が表と一致する", () => {
  const cases = readJson("fxhash64.json");
  assert.ok(cases.length > 0);
  for (const c of cases) {
    const got = fxhash64(c.in).toString(16).padStart(16, "0");
    assert.equal(got, c.hash, `入力: ${JSON.stringify(c.in)}`);
  }
});

test("encodeQueryValue が表と一致する", () => {
  const cases = readJson("encode_query_value.json");
  assert.ok(cases.length > 0);
  for (const c of cases) {
    assert.equal(encodeQueryValue(c.in), c.out, `入力: ${JSON.stringify(c.in)}`);
  }
});
