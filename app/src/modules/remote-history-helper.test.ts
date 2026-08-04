import { describe, it, expect, vi, beforeEach } from "vitest";
import { parseRemoteHistoryLines, copyCommandToClipboard } from "./remote-history-helper";

describe("remote-history-helper", () => {
  beforeEach(() => {
    // Mock de navigator.clipboard nativo
    Object.defineProperty(navigator, "clipboard", {
      value: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
      writable: true,
      configurable: true,
    });
  });

  it("should parse normal lines without timestamps as N/D", () => {
    const raw = ["ls -la", "cd /var/www", "npm run dev"];
    const parsed = parseRemoteHistoryLines(raw);
    expect(parsed).toEqual([
      { date: "N/D", command: "ls -la" },
      { date: "N/D", command: "cd /var/www" },
      { date: "N/D", command: "npm run dev" },
    ]);
  });

  it("should parse Zsh extended history lines with timestamps", () => {
    const raw = [
      ": 1627999999:0;git status",
      ": 1628000000:0;node -v"
    ];
    const parsed = parseRemoteHistoryLines(raw);
    
    expect(parsed).toHaveLength(2);
    expect(parsed[0].command).toBe("git status");
    expect(parsed[0].date).not.toBe("N/D");
    expect(parsed[1].command).toBe("node -v");
    expect(parsed[1].date).not.toBe("N/D");
  });

  it("should parse Bash extended history lines with multi-line timestamps", () => {
    const raw = [
      "#1627999999",
      "docker ps",
      "#1628000000",
      "docker-compose up -d"
    ];
    const parsed = parseRemoteHistoryLines(raw);
    
    expect(parsed).toHaveLength(2);
    expect(parsed[0].command).toBe("docker ps");
    expect(parsed[0].date).not.toBe("N/D");
    expect(parsed[1].command).toBe("docker-compose up -d");
    expect(parsed[1].date).not.toBe("N/D");
  });

  describe("copyCommandToClipboard", () => {
    it("should call writeText with the command", async () => {
      await copyCommandToClipboard("my command to copy");
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith("my command to copy");
    });
  });
});
