function b64ToU8Arr(b64) {
  return Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
}

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

window.addEventListener("DOMContentLoaded", () => {
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
});
