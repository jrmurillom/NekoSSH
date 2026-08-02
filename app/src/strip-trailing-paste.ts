/**
 * Removes trailing newlines/whitespace from pasted terminal text.
 * Internal line breaks are preserved.
 */
export function stripTrailingPasteNoise(text: string): string {
  return text.replace(/\s+$/u, "");
}
