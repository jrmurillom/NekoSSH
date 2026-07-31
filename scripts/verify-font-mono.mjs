import fs from "fs";

const css = fs.readFileSync("app/src/styles.css", "utf8");
const ts = fs.readFileSync("app/src/main.ts", "utf8");

const match = css.match(/--font-mono:\s*([^;]+);/);
if (!match) {
  throw new Error("token --font-mono ausente en styles.css");
}

const token = match[1].trim();
if (!ts.includes("getComputedStyle(document.documentElement)")) {
  throw new Error("getComputedStyle ausente en main.ts");
}
if (!ts.includes('getPropertyValue("--font-mono")')) {
  throw new Error("lectura de --font-mono ausente en main.ts");
}
if (ts.includes('fontFamily: "var(--font-mono)"') || ts.includes("fontFamily: 'var(--font-mono)'")) {
  throw new Error("aún se pasa var(--font-mono) como string estático a Terminal");
}

console.log("PASS token:", token);
console.log("PASS getComputedStyle wiring");
console.log("PASS Terminal usa familia resuelta en runtime");
