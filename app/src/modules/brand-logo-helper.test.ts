import { describe, expect, it } from "vitest";
import {
  BRAND_LOGO_THEME_IDS,
  resolveBrandLogoThemeId,
  resolveBrandLogoUrl,
} from "./brand-logo-helper";

describe("resolveBrandLogoThemeId", () => {
  it("conserva ids del catálogo de 8 temas", () => {
    for (const id of BRAND_LOGO_THEME_IDS) {
      expect(resolveBrandLogoThemeId(id)).toBe(id);
    }
  });

  it("hace fallback a nekossh si el id es desconocido", () => {
    expect(resolveBrandLogoThemeId("")).toBe("nekossh");
    expect(resolveBrandLogoThemeId("desconocido")).toBe("nekossh");
    expect(resolveBrandLogoThemeId("sailor_moon")).toBe("nekossh");
  });
});

describe("resolveBrandLogoUrl", () => {
  const logos: Record<string, string> = Object.fromEntries(
    BRAND_LOGO_THEME_IDS.map((id) => [id, `url:${id}.png`]),
  );

  it("mapea cada tema a su URL", () => {
    expect(resolveBrandLogoUrl("hatsune-miku", logos)).toBe("url:hatsune-miku.png");
    expect(resolveBrandLogoUrl("persona5", logos)).toBe("url:persona5.png");
  });

  it("usa el logo nekossh cuando el tema no existe", () => {
    expect(resolveBrandLogoUrl("no-existe", logos)).toBe("url:nekossh.png");
  });

  it("usa nekossh si falta el PNG del tema pero existe el fallback", () => {
    const partial = { nekossh: "url:nekossh.png" };
    expect(resolveBrandLogoUrl("hatsune-miku", partial)).toBe("url:nekossh.png");
  });
});
