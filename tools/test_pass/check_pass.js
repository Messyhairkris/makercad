// Runs the pass's own script against a stub DOM: syntax, render() and the export text.
const fs = require("fs");
const file = process.argv[2];
const html = fs.readFileSync(file, "utf8");
const scripts = [...html.matchAll(/<script(?: id="seed" type="application\/json")?>([\s\S]*?)<\/script>/g)].map(m => m[1]);
const code = scripts.find(s => s.includes("const ITEMS"));
for (const s of scripts) { try { new Function(s); } catch (e) { if (s !== scripts[0]) { console.log("SYNTAX ERROR in a script:", e.message); process.exit(1); } } }
const els = {};
const mk = id => (els[id] = els[id] || { id, innerHTML: "", textContent: "", value: "", style: {}, select() {} });
global.document = {
  getElementById: id => id === "seed" ? { textContent: "{}" } : mk(id),
  querySelectorAll: () => [],
  addEventListener: () => {},
  execCommand: () => true,
};
global.window = { addEventListener: () => {} };
global.localStorage = { getItem: () => null, setItem: () => {} };
global.navigator = {};
eval(code);
const body = els["body"].innerHTML;
console.log("rendered body:", body.length, "chars");
const open = (body.match(/<div/g) || []).length, close = (body.match(/<\/div>/g) || []).length;
console.log("div open/close:", open, close);
console.log("sections:", (body.match(/<section>/g) || []).length, "items:", (body.match(/class="item"/g) || []).length);
console.log("imgs:", (body.match(/<img /g) || []).length, "tables:", (body.match(/<table>/g) || []).length);
console.log("literal &amp; in expectations:", (body.match(/class="expect">[^<]*&amp;amp;/g) || []).length);
exportAll();
const out = els["outText"].value;
console.log("export head:", JSON.stringify(out.split("\n").slice(0, 3)));
