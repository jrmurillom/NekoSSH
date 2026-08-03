import { describe, expect, it } from "vitest";
import { groupAndSortConnections, ConnectionFolder, ConnectionProfile } from "./connection-tree-helper";

describe("connection-tree-helper", () => {
  it("debe ordenar las carpetas según sort_order y agrupar sus perfiles correctamente", () => {
    const folders: ConnectionFolder[] = [
      { id: 2, name: "Producción", sort_order: 2 },
      { id: 1, name: "Desarrollo", sort_order: 1 },
    ];

    const profiles: ConnectionProfile[] = [
      { id: 101, folder_id: 1, name: "Dev Server 1", host: "1.1.1.1", port: 22, username: "root", auth_type: "password", keepalive: 30, tunnel_type: "none" },
      { id: 102, folder_id: 2, name: "Prod DB", host: "2.2.2.2", port: 22, username: "admin", auth_type: "key", keepalive: 30, tunnel_type: "none" },
    ];

    const result = groupAndSortConnections(folders, profiles);

    expect(result.length).toBe(2);
    expect(result[0].folder.name).toBe("Desarrollo");
    expect(result[0].profiles.length).toBe(1);
    expect(result[0].profiles[0].name).toBe("Dev Server 1");

    expect(result[1].folder.name).toBe("Producción");
    expect(result[1].profiles.length).toBe(1);
    expect(result[1].profiles[0].name).toBe("Prod DB");
  });

  it("debe asignar perfiles huérfanos a la primera carpeta disponible", () => {
    const folders: ConnectionFolder[] = [
      { id: 10, name: "General", sort_order: 0 },
    ];

    const profiles: ConnectionProfile[] = [
      { id: 201, folder_id: 999, name: "Servidor Huérfano", host: "3.3.3.3", port: 22, username: "user", auth_type: "password", keepalive: 30, tunnel_type: "none" },
    ];

    const result = groupAndSortConnections(folders, profiles);

    expect(result.length).toBe(1);
    expect(result[0].profiles.length).toBe(1);
    expect(result[0].profiles[0].name).toBe("Servidor Huérfano");
  });
});
