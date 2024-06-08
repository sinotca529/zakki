function createTagElem(tagName) {
  const a = document.createElement("a");
  a.className = "tag";
  a.href = `tag.html?tag=${tagName}`;
  a.innerHTML = tagName;
  return a;
}

function createCard(page) {
  const cardLink = document.createElement("a");
  cardLink.className = "card-link";
  cardLink.href = page.path;

  const card = document.createElement("div");
  card.className = "card";

  const header = document.createElement("div");
  header.className = "card-header";
  header.innerHTML = page.title;

  const meta = document.createElement("div");
  meta.className = "card-meta";

  const date = document.createElement("div");
  date.className = "card-date";
  date.innerHTML = page.date;

  const tags = document.createElement("div");
  tags.className = "card-tags";
  page.tags.forEach((tagName) => {
    const tag = createTagElem(tagName);
    tags.appendChild(tag);
  });

  meta.appendChild(date);
  meta.appendChild(tags);

  card.appendChild(header);
  card.appendChild(meta);

  cardLink.appendChild(card);

  return cardLink;
}
function tagMain() {
  const params = new URLSearchParams(window.location.search);
  if (!params.has("tag")) return;
  const tag = params.get("tag");

  document.getElementById("h1").innerHTML = `タグ : ${tag} のついたページ`;
  document.getElementsByTagName("title")[0].innerHTML = `タグ: ${tag}`;

  const fragment = document.createDocumentFragment();
  METADATA.filter((page) => page.tags.includes(tag))
    .sort((a, b) => b.date.localeCompare(a.date))
    .forEach((page) => fragment.appendChild(createCard(page)));

  document.getElementById("contents-list").appendChild(fragment);
}

function renderPageList(metadata) {
  const fragment = document.createDocumentFragment();

  metadata
    .sort((a, b) => b.date.localeCompare(a.date))
    .forEach((page) => fragment.appendChild(createCard(page)));

  document.getElementById("contents-list").appendChild(fragment);
}

function renderTagSet(metadata) {
  const tagSet = new Set(
    Object.keys(metadata)
      .map((key) => metadata[key].tags)
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
  renderPageList(METADATA);
  renderTagSet(METADATA);
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
  const cypherData = base64ToUint8Array(document.body.dataset.cypher);
  const iv = cypherData.slice(0, 16);
  const encryptedData = cypherData.slice(16);
  const keyObj = await getAesKey();

  try {
    const decryptedData = await decryptAes256Cbc(encryptedData, iv, keyObj);
    const decryptedText = new TextDecoder().decode(decryptedData);
    document.documentElement.innerHTML = decryptedText;
  } catch (error) {
    console.error("Decryption failed:", error);
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
