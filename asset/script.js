function createTagElem(tagName) {
  const a = document.createElement("a");
  a.className = "tag";
  a.href = `tag.html?tag=${tagName}`;
  a.innerHTML = tagName;
  return a;
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
