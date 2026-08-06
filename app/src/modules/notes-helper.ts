import { invoke } from "@tauri-apps/api/core";
import { AppIcons, setButtonIcon, icon } from "../icons";
import { confirmDialog } from "../overlays";

interface Note {
  id?: number;
  title: string;
  content: string;
  updated_at?: string;
}

let notesState: Note[] = [];
let activeNote: Note | null = null;
let saveTimeout: number | null = null;

// DOM Elements
let tabBtnNotes: HTMLButtonElement | null = null;
let panelNotes: HTMLElement | null = null;
let btnNewNote: HTMLButtonElement | null = null;
let notesList: HTMLElement | null = null;

let notesModal: HTMLElement | null = null;
let noteModalTitle: HTMLInputElement | null = null;
let noteModalBody: HTMLTextAreaElement | null = null;
let btnDeleteNote: HTMLButtonElement | null = null;
let btnCloseNotesModal: HTMLButtonElement | null = null;
let noteSaveStatus: HTMLElement | null = null;

export function initNotesTab(): void {
  // Query UI Elements
  tabBtnNotes = document.getElementById("tab-btn-notes") as HTMLButtonElement;
  panelNotes = document.getElementById("panel-notes");
  btnNewNote = document.getElementById("btn-new-note") as HTMLButtonElement;
  notesList = document.getElementById("notes-list");

  notesModal = document.getElementById("notes-modal");
  noteModalTitle = document.getElementById("note-modal-title") as HTMLInputElement;
  noteModalBody = document.getElementById("note-modal-body") as HTMLTextAreaElement;
  btnDeleteNote = document.getElementById("btn-delete-note") as HTMLButtonElement;
  btnCloseNotesModal = document.getElementById("btn-close-notes-modal") as HTMLButtonElement;
  noteSaveStatus = document.getElementById("note-save-status");

  // Setup Icons
  if (btnNewNote) {
    setButtonIcon(btnNewNote, AppIcons.plus);
  }
  if (btnDeleteNote) {
    setButtonIcon(btnDeleteNote, AppIcons.trash2);
  }
  if (btnCloseNotesModal) {
    setButtonIcon(btnCloseNotesModal, AppIcons.x);
  }

  // Bind Sidebar Tab Navigation
  tabBtnNotes?.addEventListener("click", () => {
    // Switch Tab Classes
    document.querySelectorAll(".sidebar-tab-btn").forEach(btn => btn.classList.remove("active"));
    document.querySelectorAll(".sidebar-panel").forEach(p => p.classList.remove("active"));

    tabBtnNotes?.classList.add("active");
    panelNotes?.classList.add("active");

    // Stop Monitor if active
    const stopMonitor = (window as any).stopMonitorInterval;
    if (typeof stopMonitor === "function") {
      stopMonitor();
    }

    void loadAndRenderNotes();
  });

  // Bind CRUD Events
  btnNewNote?.addEventListener("click", () => {
    void handleCreateNote();
  });

  // Bind Modal Events
  btnCloseNotesModal?.addEventListener("click", () => {
    closeNoteModal();
  });

  // Close modal when clicking background
  notesModal?.addEventListener("click", (e) => {
    if (e.target === notesModal) {
      closeNoteModal();
    }
  });

  // Auto-save on modal body input (with debounce)
  noteModalBody?.addEventListener("input", () => {
    triggerAutoSave();
  });

  noteModalBody?.addEventListener("blur", () => {
    void saveCurrentNoteImmediate();
  });

  // Auto-save and rename on title input / blur
  noteModalTitle?.addEventListener("input", () => {
    if (activeNote) {
      // Update UI title dynamically in the list sidebar without modifying activeNote yet
      const activeEl = notesList?.querySelector(`.profile-item[data-id="${activeNote.id}"] .note-item-title`);
      if (activeEl) {
        activeEl.textContent = noteModalTitle?.value || "Nota sin título";
      }
    }
    triggerAutoSave();
  });

  noteModalTitle?.addEventListener("blur", () => {
    void saveCurrentNoteImmediate();
  });

  // Delete note from modal
  btnDeleteNote?.addEventListener("click", () => {
    if (activeNote && typeof activeNote.id === "number") {
      void (async () => {
        const confirmDelete = await confirmDialog({
          title: "Eliminar Nota",
          message: "¿Estás seguro de que deseas eliminar esta nota permanentemente?",
          confirmLabel: "Eliminar"
        });
        if (confirmDelete && activeNote && typeof activeNote.id === "number") {
          void handleDeleteNote(activeNote.id);
        }
      })();
    }
  });
}

// Fetch notes from SQLite database and render
async function loadAndRenderNotes(): Promise<void> {
  try {
    const list = await invoke<Note[]>("get_notes_cmd");
    notesState = list;
    renderNotes();
  } catch (err) {
    console.error("Error al cargar notas:", err);
  }
}

// Render notes list
function renderNotes(): void {
  if (!notesList) return;
  notesList.innerHTML = "";

  if (notesState.length === 0) {
    const empty = document.createElement("div");
    empty.className = "profile-list-empty";
    empty.textContent = "No tienes notas creadas.";
    notesList.appendChild(empty);
    return;
  }

  notesState.forEach(note => {
    const item = document.createElement("div");
    item.className = "profile-item";
    item.dataset.id = String(note.id);
    if (activeNote && activeNote.id === note.id) {
      item.classList.add("active");
    }

    // Prepend FileText icon to the list item
    const leftContainer = document.createElement("div");
    leftContainer.style.display = "flex";
    leftContainer.style.alignItems = "center";
    leftContainer.style.gap = "10px";

    const noteIconEl = icon(AppIcons.fileText, { size: 14 });
    noteIconEl.style.color = "var(--color-accent-primary)";

    const textContainer = document.createElement("div");
    textContainer.style.display = "flex";
    textContainer.style.flexDirection = "column";

    const titleEl = document.createElement("span");
    titleEl.className = "note-item-title";
    titleEl.textContent = note.title || "Nota sin título";

    const dateEl = document.createElement("span");
    dateEl.className = "note-item-date";
    dateEl.textContent = note.updated_at ? note.updated_at.split(".")[0] : "";

    textContainer.appendChild(titleEl);
    textContainer.appendChild(dateEl);

    leftContainer.appendChild(noteIconEl);
    leftContainer.appendChild(textContainer);

    item.appendChild(leftContainer);

    // Click to open modal
    item.addEventListener("click", () => {
      openNoteModal(note);
    });

    notesList?.appendChild(item);
  });
}

// Create new empty note
async function handleCreateNote(): Promise<void> {
  try {
    const newNote = await invoke<Note>("create_note_cmd", {
      title: "Nueva Nota",
      content: ""
    });
    notesState.unshift(newNote);
    renderNotes();
    openNoteModal(newNote);
  } catch (err) {
    console.error("Error al crear nota:", err);
  }
}

// Delete Note
async function handleDeleteNote(id: number): Promise<void> {
  try {
    await invoke("delete_note_cmd", { id });
    notesState = notesState.filter(n => n.id !== id);
    activeNote = null;
    closeNoteModal();
    renderNotes();
  } catch (err) {
    console.error("Error al eliminar nota:", err);
  }
}

// Open modal
function openNoteModal(note: Note): void {
  activeNote = { ...note }; // local clone

  if (noteModalTitle) noteModalTitle.value = activeNote.title;
  if (noteModalBody) noteModalBody.value = activeNote.content;
  if (noteSaveStatus) {
    noteSaveStatus.textContent = "Cambios guardados";
    noteSaveStatus.style.opacity = "0.7";
  }

  // Highlight active item in sidebar list
  notesList?.querySelectorAll(".profile-item").forEach(el => el.classList.remove("active"));
  const activeEl = notesList?.querySelector(`.profile-item[data-id="${note.id}"]`);
  activeEl?.classList.add("active");

  // Show Modal
  if (notesModal) {
    notesModal.classList.add("active");
    notesModal.setAttribute("aria-hidden", "false");
  }
  noteModalBody?.focus();
}

// Close modal & ensure saving
function closeNoteModal(): void {
  if (saveTimeout) {
    clearTimeout(saveTimeout);
    saveTimeout = null;
  }
  void saveCurrentNoteImmediate();

  if (notesModal) {
    notesModal.classList.remove("active");
    notesModal.setAttribute("aria-hidden", "true");
  }
  activeNote = null;
  // Reload and render notes in sidebar
  void loadAndRenderNotes();
}

// Debounced auto-save
function triggerAutoSave(): void {
  if (noteSaveStatus) {
    noteSaveStatus.textContent = "Guardando...";
    noteSaveStatus.style.opacity = "1";
  }

  if (saveTimeout) {
    clearTimeout(saveTimeout);
  }

  saveTimeout = window.setTimeout(() => {
    void saveCurrentNoteImmediate();
  }, 1000);
}

// Immediate database save
async function saveCurrentNoteImmediate(): Promise<void> {
  if (!activeNote || typeof activeNote.id !== "number") return;

  const currentTitle = noteModalTitle?.value || "Nota sin título";
  const currentContent = noteModalBody?.value || "";

  // Only save if dirty
  const isDirty = activeNote.title !== currentTitle || activeNote.content !== currentContent;
  if (!isDirty) {
    if (noteSaveStatus) {
      noteSaveStatus.textContent = "Cambios guardados";
      noteSaveStatus.style.opacity = "0.7";
    }
    return;
  }

  activeNote.title = currentTitle;
  activeNote.content = currentContent;

  try {
    await invoke("update_note_cmd", {
      id: activeNote.id,
      title: activeNote.title,
      content: activeNote.content
    });

    if (noteSaveStatus) {
      noteSaveStatus.textContent = "Cambios guardados";
      noteSaveStatus.style.opacity = "0.7";
    }
  } catch (err) {
    console.error("Error al auto-guardar nota:", err);
    if (noteSaveStatus) {
      noteSaveStatus.textContent = "Error al guardar";
      noteSaveStatus.style.opacity = "1";
    }
  }
}
