function createTagElem(tagName) {
  const a = document.createElement("a");
  a.className = "tag";
  a.href = `index.html?tag=${tagName}`;
  a.innerHTML = tagName;
  return a;
}

function createCard(page) {
  const template = document.getElementById("card-template");

  const content = template.content.cloneNode(true);
  const card = content.querySelector(".card");
  card.href = page.path;
  if (page.flags.includes("crypto")) card.classList.add("crypto");

  content.querySelector(".card-title").innerHTML = page.title;
  content.querySelector(".card-date").innerHTML = page.update;

  const tags = content.querySelector(".card-tags");
  tags.innerHTML = "";
  page.tags.forEach((tagName) => tags.appendChild(createTagElem(tagName)));

  return content;
}

function renderPageList() {
  const fragment = document.createDocumentFragment();

  METADATA.forEach((page) => {
    const li = document.createElement("li");
    li.appendChild(createCard(page));
    fragment.appendChild(li);
  });

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

window.addEventListener("DOMContentLoaded", () => {
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
});
