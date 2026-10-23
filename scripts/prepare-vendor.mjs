// Copies the exact PDF libraries the UI was written against into src/vendor so the app
// has no network dependency. Fails loudly (and therefore fails the build) if anything is
// missing or the wrong version, so an online-only APK can never be produced by accident.
import { copyFileSync, existsSync, mkdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const out = join(root, "src", "vendor");
mkdirSync(out, { recursive: true });

const WANT = [
  { pkg: "pdf-lib", version: "1.17.1", from: "dist/pdf-lib.min.js", to: "pdf-lib.min.js" },
  { pkg: "pdfjs-dist", version: "3.11.174", from: "build/pdf.min.js", to: "pdf.min.js" },
  { pkg: "pdfjs-dist", version: "3.11.174", from: "build/pdf.worker.min.js", to: "pdf.worker.min.js" },
];

let bad = false;
for (const w of WANT) {
  const dir = join(root, "node_modules", w.pkg);
  const dest = join(out, w.to);
  if (existsSync(join(dir, "package.json"))) {
    const v = JSON.parse(readFileSync(join(dir, "package.json"), "utf8")).version;
    if (v !== w.version) { console.error(`✗ ${w.pkg} is ${v}, need exactly ${w.version}`); bad = true; continue; }
    copyFileSync(join(dir, w.from), dest);
  }
  if (!existsSync(dest) || statSync(dest).size < 50_000) {
    console.error(`✗ missing ${dest} — run "npm install" first`); bad = true; continue;
  }
  console.log(`✓ ${w.to} (${w.pkg}@${w.version}, ${statSync(dest).size} bytes)`);
}
if (bad) process.exit(1);
