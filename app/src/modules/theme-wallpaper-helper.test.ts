import { describe, expect, it } from "vitest";
import {
  clearThemeWallpaper,
  classifyWallpaperUrlForMigration,
  getThemeWallpaper,
  migrateLegacyWallpaper,
  parseThemeWallpaperMap,
  serializeThemeWallpaperMap,
  setThemeWallpaper,
} from "./theme-wallpaper-helper";

describe("theme-wallpaper-helper", () => {
  it("devuelve wallpaper vacío si el tema no tiene entry", () => {
    expect(getThemeWallpaper({}, "nekossh")).toEqual({
      url: "",
      label: "",
      opacity: 0.3,
    });
  });

  it("guarda y lee por tema sin tocar otros", () => {
    let map = setThemeWallpaper({}, "nekossh", {
      url: "data:a",
      label: "a.png",
      opacity: 0.5,
    });
    map = setThemeWallpaper(map, "hatsune-miku", {
      url: "data:b",
      label: "b.png",
      opacity: 0.8,
    });

    expect(getThemeWallpaper(map, "nekossh").url).toBe("data:a");
    expect(getThemeWallpaper(map, "hatsune-miku").label).toBe("b.png");
    expect(getThemeWallpaper(map, "persona5").url).toBe("");
  });

  it("clear solo limpia el tema indicado", () => {
    let map = setThemeWallpaper({}, "nekossh", { url: "data:a", label: "a", opacity: 0.4 });
    map = setThemeWallpaper(map, "persona5", { url: "data:p", label: "p", opacity: 0.9 });
    map = clearThemeWallpaper(map, "nekossh");

    expect(getThemeWallpaper(map, "nekossh")).toEqual({ url: "", label: "", opacity: 0.3 });
    expect(getThemeWallpaper(map, "persona5").url).toBe("data:p");
  });

  it("serializa y parsea el mapa", () => {
    const map = setThemeWallpaper({}, "rei-ayanami", {
      url: "https://x/y.png",
      label: "y.png",
      opacity: 0.25,
    });
    const roundtrip = parseThemeWallpaperMap(serializeThemeWallpaperMap(map));
    expect(getThemeWallpaper(roundtrip, "rei-ayanami")).toEqual({
      url: "https://x/y.png",
      label: "y.png",
      opacity: 0.25,
    });
  });

  it("migra legacy al tema destino si aún no tiene entry", () => {
    const result = migrateLegacyWallpaper(
      {},
      { url: "data:legacy", label: "old.png", opacity: "0.55" },
      "nekossh",
    );
    expect(result.didMigrate).toBe(true);
    expect(result.clearLegacy).toBe(true);
    expect(getThemeWallpaper(result.map, "nekossh")).toEqual({
      url: "data:legacy",
      label: "old.png",
      opacity: 0.55,
    });
  });

  it("no sobrescribe un tema que ya tiene entry; igual pide borrar legacy", () => {
    const existing = setThemeWallpaper({}, "nekossh", {
      url: "data:keep",
      label: "keep.png",
      opacity: 0.2,
    });
    const result = migrateLegacyWallpaper(
      existing,
      { url: "data:legacy", label: "old.png", opacity: "0.9" },
      "nekossh",
    );
    expect(result.didMigrate).toBe(false);
    expect(result.clearLegacy).toBe(true);
    expect(getThemeWallpaper(result.map, "nekossh").url).toBe("data:keep");
  });

  it("sin claves legacy no migra", () => {
    const result = migrateLegacyWallpaper(
      {},
      { url: null, label: null, opacity: null },
      "nekossh",
    );
    expect(result).toEqual({ map: {}, didMigrate: false, clearLegacy: false });
  });

  it("clasifica URLs para migración a BD/disco", () => {
    expect(classifyWallpaperUrlForMigration("")).toEqual({ kind: "opacity_only" });
    expect(classifyWallpaperUrlForMigration("data:image/png;base64,aa")).toEqual({
      kind: "data_url",
      dataUrl: "data:image/png;base64,aa",
    });
    expect(classifyWallpaperUrlForMigration("https://x/y.png")).toEqual({
      kind: "http_url",
      url: "https://x/y.png",
    });
    expect(classifyWallpaperUrlForMigration("C:\\fotos\\a.png")).toEqual({
      kind: "file_path",
      path: "C:\\fotos\\a.png",
    });
    expect(classifyWallpaperUrlForMigration("foto.png")).toEqual({ kind: "skip" });
  });
});
