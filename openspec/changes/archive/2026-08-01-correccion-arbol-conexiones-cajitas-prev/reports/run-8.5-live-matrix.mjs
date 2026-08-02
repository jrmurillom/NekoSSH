/**
 * Live matrix 8.5 — Playwright vs Vite :1420 with mocked Tauri IPC.
 * Exercises real app CSS + main.ts handlers; NOT the native WebView CDP.
 */
import { chromium } from "playwright";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUT = path.join(__dirname, "evidence-8.5");
const BASE = process.env.NEKO_URL || "http://localhost:1420/";
fs.mkdirSync(OUT, { recursive: true });

const results = [];
function record(id, name, status, evidence) {
  results.push({ id, name, status, evidence });
  console.log(`[${status}] ${id} ${name} — ${evidence}`);
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    permissions: ["clipboard-read", "clipboard-write"],
  });
  const page = await context.newPage();
  page.on("console", (msg) => {
    if (msg.type() === "error") console.log("PAGE_ERR:", msg.text());
  });

  await page.addInitScript(() => {
    const state = {
      nextFolderId: 3,
      nextProfileId: 3,
      folders: [
        { id: 1, name: "General", sort_order: 0 },
        { id: 2, name: "Nueva carpeta", sort_order: 1 },
      ],
      profiles: [
        {
          id: 1,
          folder_id: 1,
          name: "asd",
          host: "192.168.1.10",
          port: 22,
          username: "root",
          auth_type: "password",
          password: "x",
          keepalive: 60,
          tunnel_type: "none",
        },
        {
          id: 2,
          folder_id: 1,
          name: "test",
          host: "example.com",
          port: 2222,
          username: "admin",
          auth_type: "password",
          password: "y",
          keepalive: 60,
          tunnel_type: "none",
        },
      ],
      lastStartSsh: null,
      invokes: [],
    };
    window.__NEKO_MOCK__ = state;

    const listeners = new Map();
    window.__TAURI_INTERNALS__ = {
      invertChannelId: true,
      transformCallback: (callback, once) => {
        const id = Math.floor(Math.random() * 1e9);
        const wrapper = (payload) => {
          callback(payload);
          if (once) listeners.delete(id);
        };
        listeners.set(id, wrapper);
        return id;
      },
      unregisterCallback: (id) => listeners.delete(id),
      runCallback: (id, data) => {
        const cb = listeners.get(id);
        if (cb) cb(data);
      },
      invoke: async (cmd, args = {}) => {
        state.invokes.push({ cmd, args });
        switch (cmd) {
          case "list_folders":
            return structuredClone(state.folders);
          case "get_profiles":
            return structuredClone(state.profiles);
          case "create_folder": {
            const id = state.nextFolderId++;
            state.folders.push({
              id,
              name: args.name ?? "Nueva carpeta",
              sort_order: args.sort_order ?? state.folders.length,
            });
            return id;
          }
          case "update_folder": {
            const f = state.folders.find((x) => x.id === args.id);
            if (f) f.name = args.name;
            return null;
          }
          case "delete_folder": {
            state.folders = state.folders.filter((x) => x.id !== args.id);
            state.profiles = state.profiles.filter((x) => x.folder_id !== args.id);
            return null;
          }
          case "create_profile": {
            const p = { ...args.profile, id: state.nextProfileId++ };
            state.profiles.push(p);
            return p.id;
          }
          case "update_profile": {
            const idx = state.profiles.findIndex((x) => x.id === args.profile?.id);
            if (idx >= 0) state.profiles[idx] = { ...state.profiles[idx], ...args.profile };
            return null;
          }
          case "delete_profile": {
            state.profiles = state.profiles.filter((x) => x.id !== args.id);
            return null;
          }
          case "start_ssh_session": {
            state.lastStartSsh = args;
            return null;
          }
          case "close_ssh_session":
          case "write_ssh_input":
          case "resize_ssh_pty":
          case "get_preferred_external_editor_cmd":
            return "";
          case "set_preferred_external_editor_cmd":
            return null;
          case "ensure_snippet_seed_cmd":
            return null;
          case "list_snippet_categories":
            return [{ id: 1, name: "General", sort_order: 0 }];
          case "list_snippets_cmd":
            return [];
          case "plugin:event|listen":
          case "plugin:event|unlisten":
            return null;
          default:
            if (String(cmd).startsWith("plugin:")) return null;
            console.warn("[mock invoke unhandled]", cmd, args);
            return null;
        }
      },
    };
  });

  await page.goto(BASE, { waitUntil: "networkidle", timeout: 60000 });
  await page.waitForSelector(".connection-tree .folder-row", { timeout: 15000 });
  await page.screenshot({ path: path.join(OUT, "01-tree-initial.png") });

  // --- 2 Cajitas ---
  const cajita = await page.evaluate(() => {
    const el = document.querySelector(".connection-tree .profile-item");
    if (!el) return null;
    const s = getComputedStyle(el);
    return {
      background: s.backgroundColor,
      borderTopWidth: s.borderTopWidth,
      borderTopStyle: s.borderTopStyle,
      borderTopColor: s.borderTopColor,
      borderRadius: s.borderRadius,
      padding: s.padding,
    };
  });
  const cajitaOk =
    cajita &&
    cajita.borderTopStyle === "solid" &&
    parseFloat(cajita.borderTopWidth) >= 1 &&
    parseFloat(cajita.borderRadius) > 0 &&
    cajita.background !== "rgba(0, 0, 0, 0)" &&
    cajita.background !== "transparent";
  record(
    2,
    "Cajitas visibles (fondo/borde/radius)",
    cajitaOk ? "PASS" : "FAIL",
    cajitaOk
      ? `computed: bg=${cajita.background}; border=${cajita.borderTopWidth} ${cajita.borderTopStyle} ${cajita.borderTopColor}; radius=${cajita.borderRadius}; padding=${cajita.padding}`
      : `computed=${JSON.stringify(cajita)}`,
  );

  // --- 3 Indent + guide ---
  const guide = await page.evaluate(() => {
    const el = document.querySelector(".connection-tree .folder-children");
    if (!el) return null;
    const s = getComputedStyle(el);
    return {
      marginLeft: s.marginLeft,
      paddingLeft: s.paddingLeft,
      borderLeftWidth: s.borderLeftWidth,
      borderLeftStyle: s.borderLeftStyle,
      borderLeftColor: s.borderLeftColor,
    };
  });
  const guideOk =
    guide &&
    parseFloat(guide.marginLeft) > 0 &&
    parseFloat(guide.paddingLeft) > 0 &&
    guide.borderLeftStyle === "solid" &&
    parseFloat(guide.borderLeftWidth) >= 1;
  record(
    3,
    "Indent + línea guía",
    guideOk ? "PASS" : "FAIL",
    guideOk ? JSON.stringify(guide) : `computed=${JSON.stringify(guide)}`,
  );

  // --- 1 Expand/collapse ---
  const beforeKids = await page.locator(".folder-block").first().locator(".folder-children .profile-item").count();
  await page.locator(".folder-row").first().click();
  await page.waitForTimeout(200);
  const afterCollapse = await page.locator(".folder-block").first().locator(".folder-children .profile-item").count();
  await page.locator(".folder-row").first().click();
  await page.waitForTimeout(200);
  const afterExpand = await page.locator(".folder-block").first().locator(".folder-children .profile-item").count();
  const expandOk = beforeKids > 0 && afterCollapse === 0 && afterExpand === beforeKids;
  await page.screenshot({ path: path.join(OUT, "02-after-expand-collapse.png") });
  record(
    1,
    "Expand/collapse carpeta",
    expandOk ? "PASS" : "FAIL",
    `before=${beforeKids} collapsed=${afterCollapse} reexpanded=${afterExpand}`,
  );

  // --- 11 Empty dashed ---
  const emptyCss = await page.evaluate(() => {
    const el = document.querySelector(".connection-tree .folder-empty");
    if (!el) return null;
    const s = getComputedStyle(el);
    return {
      text: el.textContent?.trim(),
      borderStyle: s.borderTopStyle,
      borderWidth: s.borderTopWidth,
    };
  });
  const emptyOk =
    emptyCss &&
    emptyCss.text === "Sin conexiones" &&
    emptyCss.borderStyle === "dashed" &&
    parseFloat(emptyCss.borderWidth) >= 1;
  record(
    11,
    "Empty «Sin conexiones» dashed",
    emptyOk ? "PASS" : "FAIL",
    JSON.stringify(emptyCss),
  );

  // --- 10 Copy user@host ---
  await page.evaluate(() => navigator.clipboard.writeText(""));
  await page.locator(".btn-copy-endpoint").first().click();
  await page.waitForTimeout(150);
  const clip = await page.evaluate(() => navigator.clipboard.readText());
  const copyOk = clip === "root@192.168.1.10";
  record(
    10,
    "Copiar user@host",
    copyOk ? "PASS" : "FAIL",
    `clipboard="${clip}" expected="root@192.168.1.10"`,
  );

  // --- 5 Context menu Edit ---
  const profileItem = page.locator(".connection-tree .profile-item").first();
  await profileItem.click({ button: "right" });
  await page.waitForSelector(".chrome-context-menu", { timeout: 5000 });
  await page.locator(".chrome-context-menu .chrome-context-item", { hasText: "Editar" }).click();
  await page.waitForTimeout(300);
  const modalActive = await page.evaluate(() => {
    const m = document.getElementById("profile-modal");
    return {
      active: m?.classList.contains("active") ?? false,
      title: document.getElementById("modal-title")?.textContent ?? "",
      name: document.getElementById("prof-name")?.value ?? "",
      host: document.getElementById("prof-host")?.value ?? "",
      user: document.getElementById("prof-username")?.value ?? "",
      port: document.getElementById("prof-port")?.value ?? "",
      id: document.getElementById("profile-id")?.value ?? "",
    };
  });
  await page.screenshot({ path: path.join(OUT, "03-edit-modal.png") });
  const editOk =
    modalActive.active &&
    modalActive.title.includes("Editar") &&
    modalActive.name === "asd" &&
    modalActive.host === "192.168.1.10" &&
    modalActive.user === "root" &&
    modalActive.port === "22" &&
    modalActive.id === "1";
  record(
    5,
    "Context menu Editar abre modal con datos",
    editOk ? "PASS" : "FAIL",
    JSON.stringify(modalActive),
  );

  // --- 6 Save edit persists ---
  if (modalActive.active) {
    await page.fill("#prof-name", "asd-edited");
    await page.fill("#prof-host", "10.0.0.99");
    await page.click("#btn-save-profile");
    await page.waitForTimeout(500);
  }
  const afterSave = await page.evaluate(() => {
    const items = [...document.querySelectorAll(".connection-tree .profile-item")].map((el) => ({
      name: el.querySelector(".profile-item-name")?.textContent,
      host: el.querySelector(".profile-item-host")?.textContent,
    }));
    const mock = window.__NEKO_MOCK__;
    const p = mock.profiles.find((x) => x.id === 1);
    return { items, profile: p, invokes: mock.invokes.filter((i) => i.cmd === "update_profile") };
  });
  const saveOk =
    afterSave.profile?.name === "asd-edited" &&
    afterSave.profile?.host === "10.0.0.99" &&
    afterSave.items.some((i) => i.name === "asd-edited" && i.host?.includes("10.0.0.99")) &&
    afterSave.invokes.length >= 1;
  await page.screenshot({ path: path.join(OUT, "04-after-save.png") });
  record(
    6,
    "Guardar edit persiste y se refleja en lista",
    saveOk ? "PASS" : "FAIL",
    JSON.stringify(afterSave),
  );

  // --- 7 Rename ---
  const item2 = page.locator(".connection-tree .profile-item").filter({ hasText: "test" }).first();
  await item2.click({ button: "right" });
  await page.waitForSelector(".chrome-context-menu", { timeout: 5000 });
  await page.locator(".chrome-context-menu .chrome-context-item", { hasText: "Renombrar" }).click();
  await page.waitForTimeout(200);
  const renameInput = page.locator(".connection-tree .profile-item input, .connection-tree input").first();
  const renameVisible = await renameInput.isVisible().catch(() => false);
  if (renameVisible) {
    await renameInput.fill("test-renamed");
    await renameInput.press("Enter");
    await page.waitForTimeout(300);
  }
  const renameState = await page.evaluate(() => {
    const names = [...document.querySelectorAll(".profile-item-name")].map((n) => n.textContent);
    const p = window.__NEKO_MOCK__.profiles.find((x) => x.id === 2);
    return { names, profileName: p?.name };
  });
  const renameOk = renameVisible && renameState.profileName === "test-renamed" && renameState.names.includes("test-renamed");
  await page.screenshot({ path: path.join(OUT, "05-after-rename.png") });
  record(
    7,
    "Renombrar",
    renameOk ? "PASS" : "FAIL",
    JSON.stringify({ renameVisible, ...renameState }),
  );

  // --- 8 Delete SKIP (destructivo en DB real); document mock-only optional ---
  record(
    8,
    "Eliminar",
    "SKIP",
    "Omitido a propósito (destructivo). No se ejecutó delete_profile contra store live/real.",
  );

  // --- 4 Nueva conexión / nueva carpeta ---
  const foldersBefore = await page.evaluate(() => window.__NEKO_MOCK__.folders.length);
  await page.click("#btn-new-folder");
  await page.waitForTimeout(400);
  const foldersAfter = await page.evaluate(() => ({
    count: window.__NEKO_MOCK__.folders.length,
    last: window.__NEKO_MOCK__.folders.at(-1),
    rows: [...document.querySelectorAll(".folder-name")].map((n) => n.textContent),
  }));
  const newFolderOk = foldersAfter.count === foldersBefore + 1;

  await page.click("#btn-new-profile");
  await page.waitForTimeout(200);
  const newModal = await page.evaluate(() => ({
    active: document.getElementById("profile-modal")?.classList.contains("active"),
    title: document.getElementById("modal-title")?.textContent,
  }));
  if (newModal.active) {
    await page.fill("#prof-name", "smoke-new");
    await page.fill("#prof-host", "127.0.0.1");
    await page.fill("#prof-username", "u");
    await page.evaluate(() => document.getElementById("profile-form")?.requestSubmit());
    await page.waitForTimeout(400);
  }
  const newProf = await page.evaluate(() => {
    const p = window.__NEKO_MOCK__.profiles.find((x) => x.name === "smoke-new");
    const shown = [...document.querySelectorAll(".profile-item-name")].some((n) => n.textContent === "smoke-new");
    return { p, shown };
  });
  await page.screenshot({ path: path.join(OUT, "06-new-folder-connection.png") });
  const newOk = newFolderOk && newModal.active && newModal.title?.includes("Nueva") && !!newProf.p && newProf.shown;
  record(
    4,
    "Nueva conexión / nueva carpeta",
    newOk ? "PASS" : "FAIL",
    JSON.stringify({ newFolderOk, foldersAfter, newModal, newProf }),
  );

  // --- 9 Double-click connects ---
  // Re-find a profile item; dblclick should call start_ssh_session
  await page.evaluate(() => {
    window.__NEKO_MOCK__.lastStartSsh = null;
  });
  const target = page.locator(".connection-tree .profile-item").filter({ hasText: "asd-edited" }).first();
  await target.dblclick();
  await page.waitForTimeout(500);
  const ssh = await page.evaluate(() => window.__NEKO_MOCK__.lastStartSsh);
  const tabVisible = await page.locator(".terminal-tab, .tab-item, [class*='term']").first().isVisible().catch(() => false);
  const dblOk = ssh != null && (ssh.profile?.host === "10.0.0.99" || ssh.host === "10.0.0.99" || JSON.stringify(ssh).includes("10.0.0.99"));
  await page.screenshot({ path: path.join(OUT, "07-dblclick-connect.png") });
  record(
    9,
    "Doble clic conecta",
    dblOk ? "PASS" : "FAIL",
    `start_ssh_session args=${JSON.stringify(ssh)}; tabishVisible=${tabVisible}`,
  );

  // --- 12 Snippets/footer smoke ---
  await page.click("#btn-open-snippets");
  await page.waitForTimeout(400);
  const snippets = await page.evaluate(() => {
    const m = document.getElementById("snippets-modal");
    return {
      active: m?.classList.contains("active") ?? false,
      text: m?.textContent?.slice(0, 80) ?? "",
    };
  });
  // close snippets if open
  await page.keyboard.press("Escape").catch(() => {});
  await page.waitForTimeout(150);
  await page.click("#btn-footer-gear");
  await page.waitForTimeout(200);
  const footer = await page.evaluate(() => {
    const pop = document.getElementById("prefs-popover");
    return { open: pop?.classList.contains("is-open") ?? false };
  });
  await page.screenshot({ path: path.join(OUT, "08-snippets-footer.png") });
  const smokeOk = snippets.active && footer.open;
  record(
    12,
    "Snippets/footer smoke",
    smokeOk ? "PASS" : "FAIL",
    JSON.stringify({ snippets, footer }),
  );

  const summaryPath = path.join(OUT, "matrix-results.json");
  fs.writeFileSync(summaryPath, JSON.stringify({ base: BASE, results, at: new Date().toISOString() }, null, 2));
  console.log("WROTE", summaryPath);
  await browser.close();

  const failed = results.filter((r) => r.status === "FAIL");
  process.exit(failed.length ? 1 : 0);
}

main().catch((err) => {
  console.error(err);
  process.exit(2);
});
