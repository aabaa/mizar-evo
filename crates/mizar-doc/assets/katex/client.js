/* Bounded opaque math client; all executable and style bytes are locally pinned. */
(() => {
  "use strict";
  const scriptUrl = document.currentScript && document.currentScript.src;
  const configDigest = "5dd8852eca56609dc4e235ede907ba109f874db4a2b3d3de3da8c060c16e9b9c";
  const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });

  function fail(entry) {
    const status = document.createElement("span");
    status.setAttribute("role", "status");
    status.textContent = "Mathematics unavailable";
    const code = document.createElement("code");
    code.textContent = entry.payload;
    entry.node.replaceChildren(status, code);
    entry.node.dataset.state = "error";
  }

  async function digest(bytes) {
    const hash = await crypto.subtle.digest("SHA-256", bytes);
    return Array.from(new Uint8Array(hash), byte => byte.toString(16).padStart(2, "0")).join("");
  }

  async function start() {
    const entries = Array.from(document.querySelectorAll(".mizar-math[data-mizar-tex]"), node => {
      const entry = { node, payload: node.querySelector("code").textContent, length: 0, valid: false };
      try {
        const hex = node.dataset.mizarTex;
        if (!/^(?:[0-9a-f]{2})*$/.test(hex)) throw new Error("transport");
        const bytes = Uint8Array.from(hex.match(/../g) || [], pair => parseInt(pair, 16));
        entry.payload = decoder.decode(bytes);
        entry.length = bytes.length;
        entry.valid = true;
      } catch (_) {
        fail(entry);
      }
      return entry;
    });
    if (entries.length === 0) return;
    try {
      const base = new URL(".", scriptUrl);
      if (!/^https?:$/.test(base.protocol) || base.origin !== location.origin) throw new Error("origin");
      async function verified(path, expected) {
        const url = new URL(path, base);
        if (url.origin !== base.origin || !url.href.startsWith(base.href)) throw new Error("path");
        const response = await fetch(url, { redirect: "error", credentials: "same-origin", cache: "no-store" });
        if (!response.ok || response.redirected || response.url !== url.href) throw new Error("local asset");
        const bytes = await response.arrayBuffer();
        if (await digest(bytes) !== expected) throw new Error("asset integrity");
        return bytes;
      }
      const config = JSON.parse(decoder.decode(await verified("config.json", configDigest)));
      const assets = new Map(await Promise.all(config.assets.map(async asset => [
        asset.path, await verified(asset.path, asset.sha256)
      ])));
      const fontUrls = new Map();
      for (const [path, bytes] of assets) {
        if (path.startsWith("fonts/")) fontUrls.set(path, URL.createObjectURL(new Blob([bytes])));
      }
      const css = decoder.decode(assets.get("katex.min.css")).replace(/url\((fonts\/[^)]+)\)/g, (_, path) => {
        if (!fontUrls.has(path)) throw new Error("font inventory");
        return `url("${fontUrls.get(path)}")`;
      });
      const style = document.createElement("link");
      style.rel = "stylesheet";
      style.href = URL.createObjectURL(new Blob([css], { type: "text/css" }));
      await new Promise((resolve, reject) => {
        style.onload = resolve;
        style.onerror = reject;
        document.head.appendChild(style);
      });
      const runtime = document.createElement("script");
      runtime.src = URL.createObjectURL(new Blob([assets.get("katex.min.js")], { type: "text/javascript" }));
      try {
        await new Promise((resolve, reject) => {
          runtime.onload = resolve;
          runtime.onerror = reject;
          document.head.appendChild(runtime);
        });
      } finally {
        runtime.remove();
        URL.revokeObjectURL(runtime.src);
      }
      const engine = window.katex;
      if (!engine || engine.version !== config.release) throw new Error("runtime release");
      for (const entry of entries) {
        try {
          if (!entry.valid || entry.length > config.payload_limit) throw new Error("payload limit");
          if (!["true", "false"].includes(entry.node.dataset.mizarDisplay)) throw new Error("display mode");
          const output = document.createElement("span");
          engine.render(entry.payload, output, {
            ...config.options,
            displayMode: entry.node.dataset.mizarDisplay === "true",
            macros: Object.create(null)
          });
          entry.node.replaceChildren(...output.childNodes);
          entry.node.dataset.state = "rendered";
        } catch (_) {
          fail(entry);
        }
      }
    } catch (_) {
      entries.forEach(fail);
    }
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start, { once: true });
  } else {
    void start();
  }
})();
