/**
 * Helper puro de manipulación y filtrado del árbol de conexiones y carpetas.
 */

export interface ConnectionFolder {
  id?: number;
  name: string;
  sort_order: number;
}

export interface ConnectionProfile {
  id?: number;
  folder_id?: number;
  name: string;
  host: string;
  port: number;
  username: string;
  auth_type: "password" | "key";
  password?: string;
  private_key?: string;
  passphrase?: string;
  keepalive: number;
  tunnel_type: "none" | "local" | "dynamic";
  tunnel_local_port?: number;
  tunnel_dest?: string;
}

export interface GroupedTree {
  folder: ConnectionFolder;
  profiles: ConnectionProfile[];
}

/**
 * Agrupa y ordena las carpetas y perfiles para su renderizado en la interfaz.
 */
export function groupAndSortConnections(
  folders: ConnectionFolder[],
  profiles: ConnectionProfile[]
): GroupedTree[] {
  // Ordenar carpetas por sort_order
  const sortedFolders = [...folders].sort((a, b) => a.sort_order - b.sort_order);

  const folderMap = new Map<number, GroupedTree>();
  sortedFolders.forEach((folder) => {
    if (folder.id !== undefined) {
      folderMap.set(folder.id, { folder, profiles: [] });
    }
  });

  // Si no hay carpetas definidas, crear una carpeta por defecto 'General'
  if (sortedFolders.length === 0) {
    const defaultFolder: ConnectionFolder = { id: 1, name: "General", sort_order: 0 };
    folderMap.set(1, { folder: defaultFolder, profiles: [] });
  }

  const defaultFolderId = sortedFolders[0]?.id || 1;

  // Asignar perfiles a sus carpetas correspondientes (o a la carpeta por defecto si son huérfanos)
  profiles.forEach((profile) => {
    const targetId = profile.folder_id && folderMap.has(profile.folder_id) ? profile.folder_id : defaultFolderId;
    const group = folderMap.get(targetId);
    if (group) {
      group.profiles.push(profile);
    }
  });

  return Array.from(folderMap.values());
}
