/**
 * Snippets manager modal (Fase 4a): flat list + chips + clipboard copy.
 * Scoped to #snippets-modal — footer gear/prefs live in main.ts.
 */
import { invoke } from "@tauri-apps/api/core";
import { AppIcons, icon, setButtonIcon } from "./icons";
import { alertDialog, confirmDialog, showContextMenu } from "./overlays";

type SnippetCategory = { id?: number; name: string; sort_order: number };
type Snippet = {
  id?: number;
  category_id: number;
  title: string;
  body: string;
  sort_order: number;
};

let categories: SnippetCategory[] = [];
let snippets: Snippet[] = [];
let activeCategoryId: number | null = null; // null = Todas
let searchQuery = "";
let editingId: number | null = null;

export function initSnippetsUi() {
  const btnOpen = document.getElementById("btn-open-snippets");
  const modal = document.getElementById("snippets-modal");
  const btnClose = document.getElementById("btn-close-snippets");
  const search = document.getElementById("snippets-search") as HTMLInputElement | null;
  const btnNewSnippet = document.getElementById("btn-snippet-new");
  const form = document.getElementById("snippet-form") as HTMLFormElement | null;
  const btnCancelForm = document.getElementById("btn-snippet-form-cancel");
  const catForm = document.getElementById("snippet-category-form") as HTMLFormElement | null;
  const btnCancelCat = document.getElementById("btn-snippet-category-cancel");

  if (btnOpen) {
    setButtonIcon(btnOpen, AppIcons.clipboardList, { size: 16, className: "icon--md" });
    const label = document.createElement("span");
    label.textContent = "Snippets";
    btnOpen.appendChild(label);
    btnOpen.addEventListener("click", () => void openModal());
  }

  if (btnClose) {
    setButtonIcon(btnClose, AppIcons.x, { size: 16, className: "icon--md" });
    btnClose.addEventListener("click", () => closeModal());
  }
  modal?.addEventListener("click", (e) => {
    if (e.target === modal) closeModal();
  });
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && modal?.classList.contains("active")) {
      closeModal();
    }
  });

  search?.addEventListener("input", () => {
    searchQuery = search.value;
    renderList();
  });

  btnNewSnippet?.addEventListener("click", () => showForm(null));
  btnCancelForm?.addEventListener("click", () => hideForm());
  form?.addEventListener("submit", (e) => {
    e.preventDefault();
    void saveForm();
  });
  btnCancelCat?.addEventListener("click", () => hideCategoryForm());
  catForm?.addEventListener("submit", (e) => {
    e.preventDefault();
    void saveCategoryForm();
  });
}

async function openModal() {
  const modal = document.getElementById("snippets-modal");
  if (!modal) return;
  modal.classList.add("active");
  hideForm();
  hideCategoryForm();
  await reload();
  queueMicrotask(() => document.getElementById("snippets-search")?.focus());
}

function closeModal() {
  document.getElementById("snippets-modal")?.classList.remove("active");
  hideForm();
  hideCategoryForm();
  editingId = null;
}

async function reload() {
  try {
    await invoke("ensure_snippet_seed_cmd");
    categories = await invoke<SnippetCategory[]>("list_snippet_categories");
    snippets = await invoke<Snippet[]>("list_snippets_cmd", {
      categoryId: null,
      query: null,
    });
    renderChips();
    renderList();
    fillCategorySelect();
  } catch (err) {
    await alertDialog({ title: "Snippets", message: String(err) });
  }
}

function renderChips() {
  const row = document.getElementById("snippets-chips");
  if (!row) return;
  row.replaceChildren();

  const addBtn = document.createElement("button");
  addBtn.type = "button";
  addBtn.className = "snippets-chip snippets-chip-add";
  addBtn.title = "Nueva categoría";
  addBtn.setAttribute("aria-label", "Nueva categoría");
  addBtn.appendChild(icon(AppIcons.plus, { size: 14, className: "icon--sm" }));
  addBtn.addEventListener("click", () => showCategoryForm());
  row.appendChild(addBtn);

  const all = document.createElement("button");
  all.type = "button";
  all.className = "snippets-chip" + (activeCategoryId === null ? " is-active" : "");
  all.textContent = "Todas";
  all.addEventListener("click", () => {
    activeCategoryId = null;
    renderChips();
    renderList();
  });
  row.appendChild(all);

  for (const cat of categories) {
    if (cat.id === undefined) continue;
    const cid = cat.id;
    const chip = document.createElement("button");
    chip.type = "button";
    chip.className = "snippets-chip" + (activeCategoryId === cid ? " is-active" : "");
    const name = document.createElement("span");
    name.className = "snippets-chip-name";
    name.textContent = cat.name;
    const del = document.createElement("span");
    del.className = "snippets-chip-x";
    del.title = "Eliminar categoría";
    del.appendChild(icon(AppIcons.x, { size: 12, className: "icon--sm" }));
    del.addEventListener("click", (e) => {
      e.stopPropagation();
      void removeCategory(cid, cat.name);
    });
    chip.append(name, del);
    chip.addEventListener("click", () => {
      activeCategoryId = cid;
      renderChips();
      renderList();
    });
    row.appendChild(chip);
  }
}

function filteredSnippets(): Snippet[] {
  const q = searchQuery.trim().toLowerCase();
  return snippets.filter((s) => {
    if (activeCategoryId !== null && s.category_id !== activeCategoryId) return false;
    if (!q) return true;
    return s.title.toLowerCase().includes(q) || s.body.toLowerCase().includes(q);
  });
}

function renderList() {
  const list = document.getElementById("snippets-list");
  if (!list) return;
  list.replaceChildren();
  const items = filteredSnippets();
  if (items.length === 0) {
    const empty = document.createElement("div");
    empty.className = "snippets-empty";
    empty.textContent = "Sin snippets";
    list.appendChild(empty);
    return;
  }
  for (const sn of items) {
    if (sn.id === undefined) continue;
    const sid = sn.id;
    const row = document.createElement("div");
    row.className = "snippets-row";

    const text = document.createElement("div");
    text.className = "snippets-row-text";
    const title = document.createElement("div");
    title.className = "snippets-row-title";
    title.textContent = sn.title;
    const meta = document.createElement("div");
    meta.className = "snippets-row-cmd";
    meta.textContent = sn.body;
    text.append(title, meta);

    const actions = document.createElement("div");
    actions.className = "snippets-row-actions";

    const copyBtn = document.createElement("button");
    copyBtn.type = "button";
    copyBtn.className = "btn-icon";
    copyBtn.title = "Copiar";
    setButtonIcon(copyBtn, AppIcons.copy, { size: 14, className: "icon--sm" });
    copyBtn.addEventListener("click", () => void copyBody(sn.body));

    const moreBtn = document.createElement("button");
    moreBtn.type = "button";
    moreBtn.className = "btn-icon";
    moreBtn.title = "Más";
    setButtonIcon(moreBtn, AppIcons.moreVertical, { size: 14, className: "icon--sm" });
    moreBtn.addEventListener("click", async (e) => {
      e.stopPropagation();
      const rect = moreBtn.getBoundingClientRect();
      const choice = await showContextMenu(rect.left, rect.bottom + 4, [
        { id: "edit", label: "Editar", icon: AppIcons.pencil },
        { id: "delete", label: "Eliminar", icon: AppIcons.trash2, danger: true, separatorBefore: true },
      ]);
      if (choice === "edit") showForm(sn);
      if (choice === "delete") void removeSnippet(sid, sn.title);
    });

    actions.append(copyBtn, moreBtn);
    row.append(text, actions);
    list.appendChild(row);
  }
}

async function copyBody(body: string) {
  try {
    await navigator.clipboard.writeText(body);
  } catch (err) {
    await alertDialog({
      title: "No se pudo copiar",
      message: String(err),
    });
  }
}

function showCategoryForm() {
  hideForm();
  const panel = document.getElementById("snippet-category-panel");
  const input = document.getElementById("snippet-category-name") as HTMLInputElement | null;
  if (input) input.value = "";
  panel?.classList.add("is-open");
  queueMicrotask(() => input?.focus());
}

function hideCategoryForm() {
  document.getElementById("snippet-category-panel")?.classList.remove("is-open");
}

async function saveCategoryForm() {
  const input = document.getElementById("snippet-category-name") as HTMLInputElement | null;
  const trimmed = (input?.value ?? "").trim();
  if (!trimmed) {
    await alertDialog({ title: "Snippets", message: "El nombre no puede estar vacío." });
    return;
  }
  try {
    await invoke("create_snippet_category", { name: trimmed });
    hideCategoryForm();
    await reload();
  } catch (err) {
    await alertDialog({ title: "Error", message: String(err) });
  }
}

async function removeCategory(id: number, name: string) {
  const ok = await confirmDialog({
    title: "Eliminar categoría",
    message: `¿Eliminar la categoría "${name}"?`,
    impact: "También se eliminarán todos los snippets de esta categoría.",
    confirmLabel: "Eliminar",
    danger: true,
  });
  if (!ok) return;
  try {
    await invoke("delete_snippet_category", { id });
    if (activeCategoryId === id) activeCategoryId = null;
    await reload();
  } catch (err) {
    await alertDialog({ title: "Error", message: String(err) });
  }
}

async function removeSnippet(id: number, title: string) {
  const ok = await confirmDialog({
    title: "Eliminar snippet",
    message: `¿Eliminar "${title}"?`,
    confirmLabel: "Eliminar",
    danger: true,
  });
  if (!ok) return;
  try {
    await invoke("delete_snippet_cmd", { id });
    await reload();
  } catch (err) {
    await alertDialog({ title: "Error", message: String(err) });
  }
}

function fillCategorySelect() {
  const sel = document.getElementById("snippet-form-category") as HTMLSelectElement | null;
  if (!sel) return;
  const prev = sel.value;
  sel.replaceChildren();
  for (const c of categories) {
    if (c.id === undefined) continue;
    const opt = document.createElement("option");
    opt.value = String(c.id);
    opt.textContent = c.name;
    sel.appendChild(opt);
  }
  if (prev && [...sel.options].some((o) => o.value === prev)) sel.value = prev;
  else if (activeCategoryId !== null) sel.value = String(activeCategoryId);
}

function showForm(sn: Snippet | null) {
  hideCategoryForm();
  const panel = document.getElementById("snippet-form-panel");
  const titleInput = document.getElementById("snippet-form-title") as HTMLInputElement | null;
  const bodyInput = document.getElementById("snippet-form-body") as HTMLTextAreaElement | null;
  const heading = document.getElementById("snippet-form-heading");
  fillCategorySelect();
  editingId = sn?.id ?? null;
  if (heading) heading.textContent = sn ? "Editar snippet" : "Nuevo snippet";
  if (titleInput) titleInput.value = sn?.title ?? "";
  if (bodyInput) bodyInput.value = sn?.body ?? "";
  const sel = document.getElementById("snippet-form-category") as HTMLSelectElement | null;
  if (sel && sn) sel.value = String(sn.category_id);
  panel?.classList.add("is-open");
  titleInput?.focus();
}

function hideForm() {
  document.getElementById("snippet-form-panel")?.classList.remove("is-open");
  editingId = null;
}

async function saveForm() {
  const titleInput = document.getElementById("snippet-form-title") as HTMLInputElement | null;
  const bodyInput = document.getElementById("snippet-form-body") as HTMLTextAreaElement | null;
  const sel = document.getElementById("snippet-form-category") as HTMLSelectElement | null;
  const title = titleInput?.value ?? "";
  const body = bodyInput?.value ?? "";
  const categoryId = Number(sel?.value);
  if (!categoryId) {
    await alertDialog({ title: "Snippets", message: "Selecciona una categoría." });
    return;
  }
  try {
    if (editingId !== null) {
      await invoke("update_snippet_cmd", { id: editingId, categoryId, title, body });
    } else {
      await invoke("create_snippet_cmd", { categoryId, title, body });
    }
    hideForm();
    await reload();
  } catch (err) {
    await alertDialog({ title: "Error", message: String(err) });
  }
}
