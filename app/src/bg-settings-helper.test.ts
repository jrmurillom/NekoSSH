import { describe, expect, it } from "vitest";
import { clampAndFormatOpacity, resolveBackgroundUrl, applyBackgroundStyle, calculateTerminalOverlayOpacity } from "./bg-settings-helper";

describe("bg-settings-helper", () => {
  describe("resolveBackgroundUrl", () => {
    it("debe retornar string vacío si la ruta está vacía o son solo espacios", () => {
      expect(resolveBackgroundUrl("")).toBe("");
      expect(resolveBackgroundUrl("   ")).toBe("");
    });

    it("debe preservar URLs remotas HTTP/HTTPS intactas", () => {
      const httpUrl = "http://example.com/wallpaper.png";
      const httpsUrl = "https://images.unsplash.com/photo-123.jpg";
      expect(resolveBackgroundUrl(httpUrl)).toBe(httpUrl);
      expect(resolveBackgroundUrl(httpsUrl)).toBe(httpsUrl);
    });

    it("debe preservar Data URIs intactas", () => {
      const dataUri = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
      expect(resolveBackgroundUrl(dataUri)).toBe(dataUri);
    });

    it("debe aplicar convertFn a rutas de archivos locales en Windows o Unix", () => {
      const mockConvert = (src: string) => `asset://localhost/${src.replace(/\\/g, "/")}`;
      const winPath = "C:\\Users\\Roberto\\Pictures\\wallpaper.jpg";
      const unixPath = "/home/user/pictures/wallpaper.png";

      expect(resolveBackgroundUrl(winPath, mockConvert)).toBe("asset://localhost/C:/Users/Roberto/Pictures/wallpaper.jpg");
      expect(resolveBackgroundUrl(unixPath, mockConvert)).toBe("asset://localhost//home/user/pictures/wallpaper.png");
    });
  });

  describe("clampAndFormatOpacity", () => {
    it("debe formatear correctamente valores normales de opacidad", () => {
      expect(clampAndFormatOpacity(0.3)).toEqual({ numeric: 0.3, formatted: "0.30" });
      expect(clampAndFormatOpacity(0.75)).toEqual({ numeric: 0.75, formatted: "0.75" });
    });

    it("debe acotar (clamp) valores menores a 0 o mayores a 1", () => {
      expect(clampAndFormatOpacity(-0.5)).toEqual({ numeric: 0, formatted: "0.00" });
      expect(clampAndFormatOpacity(1.5)).toEqual({ numeric: 1, formatted: "1.00" });
    });

    it("debe retornar valor por defecto 0.30 ante NaN", () => {
      expect(clampAndFormatOpacity(NaN)).toEqual({ numeric: 0.3, formatted: "0.30" });
    });
  });

  describe("applyBackgroundStyle", () => {
    it("debe aplicar la propiedad backgroundImage u opacidad al objeto de elemento", () => {
      const mockElement = { style: { backgroundImage: "", opacity: "" } };
      applyBackgroundStyle(mockElement, "https://example.com/bg.jpg", 0.75);

      expect(mockElement.style.backgroundImage).toBe('url("https://example.com/bg.jpg")');
      expect(mockElement.style.opacity).toBe("0.75");
    });

    it("debe resetear la propiedad si la URL es vacia", () => {
      const mockElement = { style: { backgroundImage: 'url("https://example.com/bg.jpg")', opacity: "0.75" } };
      applyBackgroundStyle(mockElement, "", 0.75);

      expect(mockElement.style.backgroundImage).toBe("");
      expect(mockElement.style.opacity).toBe("0");
    });
  });

  describe("calculateTerminalOverlayOpacity", () => {
    it("debe calcular la tinte de sobreposicion de la terminal segun la opacidad", () => {
      expect(calculateTerminalOverlayOpacity(1.0)).toBe(0.15);
      expect(calculateTerminalOverlayOpacity(0.0)).toBe(0.95);
      expect(calculateTerminalOverlayOpacity(0.3)).toBe(0.71);
    });
  });
});
