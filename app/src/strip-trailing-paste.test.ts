import { describe, expect, it } from "vitest";
import { stripTrailingPasteNoise } from "./strip-trailing-paste";

describe("stripTrailingPasteNoise", () => {
  it("quita un salto de línea final", () => {
    expect(stripTrailingPasteNoise("ls -la\n")).toBe("ls -la");
  });

  it("quita \\r\\n final", () => {
    expect(stripTrailingPasteNoise("pwd\r\n")).toBe("pwd");
  });

  it("quita varios saltos y espacios solo al final", () => {
    expect(stripTrailingPasteNoise("echo hi\n\n  \t")).toBe("echo hi");
  });

  it("conserva Enter entre líneas", () => {
    expect(stripTrailingPasteNoise("line1\nline2\n")).toBe("line1\nline2");
  });

  it("no altera texto sin trailing", () => {
    expect(stripTrailingPasteNoise("hello")).toBe("hello");
  });

  it("texto vacío queda vacío", () => {
    expect(stripTrailingPasteNoise("")).toBe("");
    expect(stripTrailingPasteNoise("\n\n")).toBe("");
  });
});
