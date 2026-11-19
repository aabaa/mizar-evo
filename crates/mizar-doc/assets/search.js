"use strict";
(() => {
  const kinds = new Set([
    "predicate", "functor", "mode", "attribute", "structure", "selector",
    "registration", "theorem", "lemma", "algorithm", "scheme", "template",
    "synonym", "antonym", "redefinition", "builtin"
  ]);
  const fold = text => text.replace(/[A-Z]/g, letter => letter.toLowerCase());
  const encode = text => encodeURIComponent(text).replace(/[!'()*]/g,
    letter => "%" + letter.charCodeAt(0).toString(16).toUpperCase());
  const string = value => typeof value === "string";
  const fields = (value, names) => value !== null && !Array.isArray(value) &&
    typeof value === "object" && Object.keys(value).length === names.length &&
    names.every(name => Object.hasOwn(value, name));
  const refuse = () => { throw new Error("Incompatible search index"); };

  function parse(text) {
    const value = JSON.parse(text);
    // JSON.parse validates syntax; this separate lexical pass retains duplicate keys.
    const tokens = /"(?:[^"\\]|\\[\s\S])*"|[{}\[\]:]/g;
    const stack = [];
    for (const match of text.matchAll(tokens)) {
      const token = match[0];
      if (token === "{") stack.push(new Set());
      else if (token === "[") stack.push(null);
      else if (token === "}" || token === "]") stack.pop();
      else if (token.startsWith('"') &&
               /^[\x09\x0a\x0d\x20]*:/.test(text.slice(match.index + token.length))) {
        const key = JSON.parse(token);
        const keys = stack[stack.length - 1];
        if (keys.has(key)) refuse();
        keys.add(key);
      }
    }
    return value;
  }

  function validate(text) {
    const index = parse(text);
    if (!fields(index, ["format_version", "package_version", "items"]) ||
        index.format_version !== 1 || !string(index.package_version) ||
        !Array.isArray(index.items)) refuse();
    for (const item of index.items) {
      if (!fields(item, ["identity", "fqn", "kind", "summary", "signature", "href"]) ||
          !string(item.fqn) || !kinds.has(item.kind) ||
          !(item.summary === null || string(item.summary)) ||
          !string(item.signature) || !string(item.href)) refuse();
      const id = item.identity;
      if (!Array.isArray(id) || id.length !== 7 || !string(id[0]) ||
          !string(id[1]) || !(id[2] === null || string(id[2])) ||
          !Array.isArray(id[3]) || !id[3].every(string) ||
          !string(id[4]) || !string(id[5]) || !string(id[6]) ||
          !/^[0-9a-f]{64}$/.test(id[6])) refuse();
      const parts = item.href.split("#");
      if (parts.length !== 2 || parts[1] !== encode(item.fqn)) refuse();
      for (const part of parts[0].split("/")) {
        const decoded = decodeURIComponent(part);
        if (!decoded || decoded === "." || decoded === ".." ||
            /[/\\\x00-\x1f\x7f-\x9f]/.test(decoded) || encode(decoded) !== part) refuse();
      }
    }
    return index.items;
  }

  function relativeHref(href, indexURL) {
    const target = new URL(href, indexURL);
    const page = new URL(document.URL).pathname.split("/");
    page.pop();
    const path = target.pathname.split("/");
    let shared = 0;
    while (shared < page.length && page[shared] === path[shared]) shared++;
    return "../".repeat(page.length - shared) + path.slice(shared).join("/") +
      href.slice(href.indexOf("#"));
  }

  async function bind(root) {
    const own = selector => {
      const nodes = Array.from(root.querySelectorAll(selector))
        .filter(node => node.closest(".mizar-search") === root);
      return nodes.length === 1 ? nodes[0] : null;
    };
    const input = own(".mizar-search-query");
    const status = own(".mizar-search-status");
    const results = own(".mizar-search-results");
    const unavailable = () => {
      if (results) results.replaceChildren();
      if (status) status.textContent = "Search unavailable";
    };
    if (!(input instanceof HTMLInputElement) || input.type !== "search" ||
        !status || status.getAttribute("role") !== "status" ||
        !results || results.tagName !== "UL") {
      unavailable();
      return;
    }
    status.textContent = "Search loading";
    results.replaceChildren();
    try {
      const source = root.getAttribute("data-mizar-index");
      if (!source) refuse();
      for (const part of source.split("/")) {
        const decoded = decodeURIComponent(part);
        if (!decoded || /[/\\\x00-\x1f\x7f-\x9f]/.test(decoded) ||
            encode(decoded) !== part) refuse();
      }
      const indexURL = new URL(source, document.URL);
      if (indexURL.origin !== location.origin || !/^https?:$/.test(indexURL.protocol) ||
          indexURL.username || indexURL.password || indexURL.search || indexURL.hash) refuse();
      const response = await fetch(indexURL.href, {redirect: "error", credentials: "same-origin"});
      if (!response.ok || response.url !== indexURL.href) refuse();
      const text = new TextDecoder("utf-8", {fatal: true, ignoreBOM: true})
        .decode(await response.arrayBuffer());
      const items = validate(text);
      const display = () => {
        const terms = fold(input.value).split(/[\x09-\x0d\x20]+/).filter(Boolean);
        const matches = items.filter(item => {
          const values = [item.fqn, item.kind, item.summary ?? "", item.signature].map(fold);
          return terms.every(term => values.some(value => value.includes(term)));
        });
        const rows = document.createDocumentFragment();
        for (const item of matches) {
          const row = document.createElement("li");
          const link = document.createElement("a");
          link.setAttribute("href", relativeHref(item.href, indexURL));
          link.textContent = item.fqn;
          row.append(link);
          for (const field of ["kind", "summary", "signature"]) {
            const span = document.createElement("span");
            span.className = "mizar-" + field;
            span.textContent = item[field] ?? "";
            row.append(span);
          }
          rows.append(row);
        }
        results.replaceChildren(rows);
        status.textContent = items.length === 0 ? "Search index is empty" :
          matches.length === 0 ? "No results" :
          matches.length + (matches.length === 1 ? " result" : " results");
      };
      input.addEventListener("input", display);
      display();
    } catch {
      unavailable();
    }
  }
  for (const root of document.querySelectorAll(".mizar-search")) void bind(root);
})();
