import { describe, expect, it } from "vitest";
import { renderSafeMarkdown } from "./markdown-lite";

describe("markdown-lite: renderizado seguro técnico", () => {
  it("escapa etiquetas HTML/XSS antes de formatear", () => {
    const html = renderSafeMarkdown(
      'Hola <script>alert(1)</script> y <img src=x onerror="evil()"> **ok**',
    );
    expect(html).not.toContain("<script>");
    expect(html).not.toContain("<img");
    expect(html).toContain("&lt;script&gt;");
    expect(html).toContain("<strong>ok</strong>");
  });

  it("renderiza tablas GFM y listas ordenadas/desordenadas", () => {
    const md = `### Estado del servidor

| Servicio | Estado | Puerto |
| :--- | :--- | :--- |
| nginx | active | 80, 443 |
| ssh | active | 22 |

- Disco raíz al 42%
- RAM libre **1.8 GiB**`;

    const html = renderSafeMarkdown(md);
    expect(html).toContain('<table class="copilot-md-table">');
    expect(html).toContain("<th>Servicio</th>");
    expect(html).toContain("<td>nginx</td>");
    expect(html).toContain('<ul class="copilot-md-list">');
    expect(html).toContain("<strong>1.8 GiB</strong>");
  });

  it("renderiza bloques de código con botón copiar y protege el interior", () => {
    const html = renderSafeMarkdown("```bash\ncat /etc/os-release 2>/dev/null\n```");
    expect(html).toContain("copilot-copy-code-btn");
    expect(html).toContain("cat /etc/os-release 2&gt;/dev/null");
  });
});
