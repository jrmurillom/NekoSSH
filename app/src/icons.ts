/**
 * Lucide outline icons for NekoSSH chrome UI.
 * Color comes from CSS `currentColor` / theme tokens — never hardcode stroke hex here.
 */
import {
  createElement,
  type IconNode,
  ArrowUp,
  ArrowRight,
  RefreshCw,
  ChevronRight,
  ChevronDown,
  Folder,
  FolderPlus,
  File,
  Pencil,
  Type,
  Trash2,
  Copy,
  X,
  Plus,
  ClipboardList,
  MoreVertical,
  Settings,
  Check,
  Terminal,
  Clipboard,
} from "lucide";

export type IconOptions = {
  size?: number;
  strokeWidth?: number;
  className?: string;
};

const DEFAULT_SIZE = 16;
const DEFAULT_STROKE = 2;

/** Create an outline Lucide SVG that inherits color from the parent. */
export function icon(node: IconNode, opts: IconOptions = {}): SVGElement {
  const size = opts.size ?? DEFAULT_SIZE;
  const strokeWidth = opts.strokeWidth ?? DEFAULT_STROKE;
  const className = ["icon", opts.className].filter(Boolean).join(" ");
  const el = createElement(node, {
    width: String(size),
    height: String(size),
    "stroke-width": String(strokeWidth),
    class: className,
    fill: "none",
    stroke: "currentColor",
    "aria-hidden": "true",
  });
  return el;
}

/** Replace button contents with a single outline icon. */
export function setButtonIcon(
  btn: HTMLElement,
  node: IconNode,
  opts?: IconOptions,
): void {
  btn.replaceChildren(icon(node, opts));
}

export const AppIcons = {
  arrowUp: ArrowUp,
  arrowRight: ArrowRight,
  refreshCw: RefreshCw,
  chevronRight: ChevronRight,
  chevronDown: ChevronDown,
  folder: Folder,
  folderPlus: FolderPlus,
  file: File,
  pencil: Pencil,
  type: Type,
  trash2: Trash2,
  copy: Copy,
  x: X,
  plus: Plus,
  clipboardList: ClipboardList,
  moreVertical: MoreVertical,
  settings: Settings,
  check: Check,
  terminal: Terminal,
  clipboard: Clipboard,
} as const;
