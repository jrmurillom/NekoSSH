use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_RECURSION_DEPTH: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UploadItem {
    pub local_path: String,
    pub relative_path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalUploadPlan {
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_bytes: u64,
    pub items: Vec<UploadItem>,
}

/// Normaliza una ruta a formato POSIX con '/' como separador.
pub fn to_posix_path(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Escanea rutas locales (archivos y/o carpetas) generando un plan ordenado para la subida SFTP.
/// Las carpetas padre siempre preceden a sus archivos y subcarpetas hijas en `items`.
pub fn scan_local_upload_items(paths: &[String]) -> Result<LocalUploadPlan, String> {
    let mut items = Vec::new();
    let mut total_bytes = 0u64;
    let mut total_files = 0usize;
    let mut total_dirs = 0usize;

    for raw_path in paths {
        let p = Path::new(raw_path);
        if !p.exists() {
            return Err(format!("La ruta local '{}' no existe", raw_path));
        }

        let meta = fs::metadata(p).map_err(|e| format!("Error al leer metadatos de '{}': {}", raw_path, e))?;
        let base_name = p
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unnamed".to_string());

        if meta.is_file() {
            let size = meta.len();
            total_files += 1;
            total_bytes += size;
            items.push(UploadItem {
                local_path: p.to_string_lossy().into_owned(),
                relative_path: base_name,
                is_dir: false,
                size,
            });
        } else if meta.is_dir() {
            total_dirs += 1;
            // Registrar carpeta raíz soltada
            items.push(UploadItem {
                local_path: p.to_string_lossy().into_owned(),
                relative_path: base_name.clone(),
                is_dir: true,
                size: 0,
            });

            // Recorrer árbol recursivamente
            let rel_base = PathBuf::from(&base_name);
            scan_dir_recursive(p, &rel_base, 1, &mut items, &mut total_files, &mut total_dirs, &mut total_bytes)?;
        }
    }

    Ok(LocalUploadPlan {
        total_files,
        total_dirs,
        total_bytes,
        items,
    })
}

fn scan_dir_recursive(
    dir_path: &Path,
    rel_prefix: &Path,
    depth: usize,
    items: &mut Vec<UploadItem>,
    total_files: &mut usize,
    total_dirs: &mut usize,
    total_bytes: &mut u64,
) -> Result<(), String> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(format!(
            "Se excedió la profundidad máxima de carpetas ({}) en '{}'",
            MAX_RECURSION_DEPTH,
            dir_path.display()
        ));
    }

    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("Error al leer directorio '{}': {}", dir_path.display(), e))?;

    // Ordenar entradas por nombre para predictibilidad y determinismo
    let mut sorted_entries = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Error leyendo entrada en '{}': {}", dir_path.display(), e))?;
        sorted_entries.push(entry);
    }
    sorted_entries.sort_by_key(|a| a.file_name());

    for entry in sorted_entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let rel_path = rel_prefix.join(&file_name);
        let rel_posix = to_posix_path(&rel_path);

        let meta = fs::metadata(&path)
            .map_err(|e| format!("Error leyendo metadatos de '{}': {}", path.display(), e))?;

        if meta.is_dir() {
            *total_dirs += 1;
            items.push(UploadItem {
                local_path: path.to_string_lossy().into_owned(),
                relative_path: rel_posix,
                is_dir: true,
                size: 0,
            });

            scan_dir_recursive(
                &path,
                &rel_path,
                depth + 1,
                items,
                total_files,
                total_dirs,
                total_bytes,
            )?;
        } else if meta.is_file() {
            let size = meta.len();
            *total_files += 1;
            *total_bytes += size;
            items.push(UploadItem {
                local_path: path.to_string_lossy().into_owned(),
                relative_path: rel_posix,
                is_dir: false,
                size,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    struct TestDir(PathBuf);
    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("nekossh-test-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&path).unwrap();
            TestDir(path)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn scan_single_file() {
        let dir = TestDir::new();
        let file_path = dir.path().join("index.html");
        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(b"<h1>Hola NekoSSH</h1>").unwrap();
        }

        let plan = scan_local_upload_items(&[file_path.to_str().unwrap().to_string()]).unwrap();
        assert_eq!(plan.total_files, 1);
        assert_eq!(plan.total_dirs, 0);
        assert_eq!(plan.total_bytes, 21);
        assert_eq!(plan.items.len(), 1);
        assert_eq!(plan.items[0].relative_path, "index.html");
        assert!(!plan.items[0].is_dir);
    }

    #[test]
    fn scan_nested_directory_tree() {
        let dir = TestDir::new();
        let root = dir.path().join("techpeople");
        fs::create_dir_all(root.join("assets").join("css")).unwrap();
        fs::create_dir_all(root.join("_next")).unwrap();

        File::create(root.join("index.html"))
            .unwrap()
            .write_all(b"html")
            .unwrap();
        File::create(root.join("assets").join("css").join("style.css"))
            .unwrap()
            .write_all(b"body {}")
            .unwrap();
        File::create(root.join("_next").join("app.js"))
            .unwrap()
            .write_all(b"console.log(1)")
            .unwrap();

        let plan = scan_local_upload_items(&[root.to_str().unwrap().to_string()]).unwrap();

        assert_eq!(plan.total_files, 3);
        // techpeople, techpeople/assets, techpeople/assets/css, techpeople/_next
        assert_eq!(plan.total_dirs, 4);
        assert_eq!(plan.total_bytes, 4 + 7 + 14);

        // Verificar que las rutas relativas siempre usan '/' y preservan jerarquía
        let rel_paths: Vec<_> = plan.items.iter().map(|i| i.relative_path.as_str()).collect();
        assert!(rel_paths.contains(&"techpeople"));
        assert!(rel_paths.contains(&"techpeople/assets"));
        assert!(rel_paths.contains(&"techpeople/assets/css"));
        assert!(rel_paths.contains(&"techpeople/assets/css/style.css"));
        assert!(rel_paths.contains(&"techpeople/_next"));
        assert!(rel_paths.contains(&"techpeople/_next/app.js"));
        assert!(rel_paths.contains(&"techpeople/index.html"));

        // Verificar que ningún relative_path contenga backslashes
        for item in &plan.items {
            assert!(!item.relative_path.contains('\\'), "Ruta con backslash: {}", item.relative_path);
        }
    }

    #[test]
    fn scan_mixed_selection() {
        let dir = TestDir::new();
        let file_path = dir.path().join("standalone.txt");
        File::create(&file_path).unwrap().write_all(b"txt").unwrap();

        let folder = dir.path().join("subfolder");
        fs::create_dir_all(&folder).unwrap();
        File::create(folder.join("nested.txt")).unwrap().write_all(b"nested").unwrap();

        let plan = scan_local_upload_items(&[
            file_path.to_str().unwrap().to_string(),
            folder.to_str().unwrap().to_string(),
        ]).unwrap();

        assert_eq!(plan.total_files, 2);
        assert_eq!(plan.total_dirs, 1);
        assert_eq!(plan.total_bytes, 3 + 6);
    }

    #[test]
    fn scan_complex_web_bundle_structure() {
        let dir = TestDir::new();
        let root = dir.path().join("techpeople_site");
        fs::create_dir_all(root.join("_next").join("static").join("chunks")).unwrap();
        fs::create_dir_all(root.join("assets").join("img").join("icons")).unwrap();
        fs::create_dir_all(root.join("locales").join("es")).unwrap();

        File::create(root.join("index.html")).unwrap().write_all(b"<!DOCTYPE html>").unwrap();
        File::create(root.join("robots.txt")).unwrap().write_all(b"User-agent: *").unwrap();
        File::create(root.join("_next").join("static").join("chunks").join("main.js"))
            .unwrap()
            .write_all(b"console.log('chunk')")
            .unwrap();
        File::create(root.join("assets").join("img").join("icons").join("favicon.ico"))
            .unwrap()
            .write_all(b"ICON")
            .unwrap();
        File::create(root.join("locales").join("es").join("common.json"))
            .unwrap()
            .write_all(b"{\"lang\":\"es\"}")
            .unwrap();
        File::create(root.join("locales").join("en.json"))
            .unwrap()
            .write_all(b"{\"lang\":\"en\"}")
            .unwrap();

        let plan = scan_local_upload_items(&[root.to_str().unwrap().to_string()]).unwrap();
        assert_eq!(plan.total_files, 6);
        assert_eq!(plan.total_bytes, 15 + 13 + 20 + 4 + 13 + 13);
        // techpeople_site, _next, static, chunks, assets, img, icons, locales, es -> 9 dirs
        assert_eq!(plan.total_dirs, 9);

        let chunk_file_pos = plan
            .items
            .iter()
            .position(|i| i.relative_path == "techpeople_site/_next/static/chunks/main.js")
            .unwrap();
        let chunk_dir_pos = plan
            .items
            .iter()
            .position(|i| i.relative_path == "techpeople_site/_next/static/chunks")
            .unwrap();
        assert!(chunk_dir_pos < chunk_file_pos, "Directorio padre debe preceder a su archivo hijo");
    }

    #[test]
    fn scan_empty_directory_tree() {
        let dir = TestDir::new();
        let root = dir.path().join("empty_folder");
        fs::create_dir_all(root.join("sub_a").join("sub_b")).unwrap();

        let plan = scan_local_upload_items(&[root.to_str().unwrap().to_string()]).unwrap();
        assert_eq!(plan.total_files, 0);
        assert_eq!(plan.total_bytes, 0);
        assert_eq!(plan.total_dirs, 3);
        assert_eq!(plan.items.len(), 3);
        assert!(plan.items.iter().all(|i| i.is_dir));
    }

    #[test]
    fn scan_zero_byte_files() {
        let dir = TestDir::new();
        let root = dir.path().join("zero_bytes");
        fs::create_dir_all(&root).unwrap();
        File::create(root.join(".gitkeep")).unwrap();
        File::create(root.join("empty.txt")).unwrap();

        let plan = scan_local_upload_items(&[root.to_str().unwrap().to_string()]).unwrap();
        assert_eq!(plan.total_files, 2);
        assert_eq!(plan.total_bytes, 0);
        assert_eq!(plan.total_dirs, 1);
        assert_eq!(plan.items.len(), 3);
    }

    #[test]
    fn scan_utf8_spaces_and_special_chars() {
        let dir = TestDir::new();
        let root = dir.path().join("carpeta con espacios y tildes (2026)");
        fs::create_dir_all(root.join("imágenes")).unwrap();
        File::create(root.join("imágenes").join("diseño #1 & copia.png"))
            .unwrap()
            .write_all(b"png")
            .unwrap();

        let plan = scan_local_upload_items(&[root.to_str().unwrap().to_string()]).unwrap();
        assert_eq!(plan.total_files, 1);
        assert_eq!(plan.total_bytes, 3);
        assert_eq!(plan.total_dirs, 2);
        assert!(plan.items.iter().any(|i| i.relative_path == "carpeta con espacios y tildes (2026)/imágenes/diseño #1 & copia.png"));
    }

    #[test]
    fn scan_non_existent_path_returns_err() {
        let res = scan_local_upload_items(&["C:\\ruta\\inexistente\\fantasma_12345.xyz".to_string()]);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("no existe"), "Error esperado debe indicar no existencia: {}", err);
    }

    #[test]
    fn scan_depth_limit_protection() {
        let dir = TestDir::new();
        let mut deep = dir.path().join("root_deep");
        for i in 1..=35 {
            deep = deep.join(format!("level_{}", i));
        }
        fs::create_dir_all(&deep).unwrap();

        let root = dir.path().join("root_deep");
        let res = scan_local_upload_items(&[root.to_str().unwrap().to_string()]);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("profundidad máxima"), "Debe reportar límite de profundidad excedido: {}", err);
    }

    #[test]
    fn scan_stress_techpeople_exact_structure_131_files() {
        let dir = TestDir::new();
        let root = dir.path().join("techpeople");
        
        let chunks_dir = root.join("_next").join("static").join("chunks");
        let css_next_dir = root.join("_next").join("static").join("css");
        let media_dir = root.join("_next").join("static").join("media");
        let team_dir = root.join("assets").join("img").join("team");
        let icons_dir = root.join("assets").join("img").join("icons");
        let css_assets_dir = root.join("assets").join("css");
        let locales_es = root.join("locales").join("es");
        let locales_en = root.join("locales").join("en");

        fs::create_dir_all(&chunks_dir).unwrap();
        fs::create_dir_all(&css_next_dir).unwrap();
        fs::create_dir_all(&media_dir).unwrap();
        fs::create_dir_all(&team_dir).unwrap();
        fs::create_dir_all(&icons_dir).unwrap();
        fs::create_dir_all(&css_assets_dir).unwrap();
        fs::create_dir_all(&locales_es).unwrap();
        fs::create_dir_all(&locales_en).unwrap();

        // 5 root files
        File::create(root.join("index.html")).unwrap().write_all(b"<html>techpeople</html>").unwrap();
        File::create(root.join("favicon.ico")).unwrap().write_all(b"icon").unwrap();
        File::create(root.join("manifest.json")).unwrap().write_all(b"{}").unwrap();
        File::create(root.join("robots.txt")).unwrap().write_all(b"User-agent: *").unwrap();
        File::create(root.join("sitemap.xml")).unwrap().write_all(b"<xml></xml>").unwrap();

        // 40 JS chunks
        for i in 1..=40 {
            File::create(chunks_dir.join(format!("chunk_{}.js", i)))
                .unwrap()
                .write_all(b"console.log('chunk')")
                .unwrap();
        }

        // 10 Next CSS files
        for i in 1..=10 {
            File::create(css_next_dir.join(format!("style_{}.css", i)))
                .unwrap()
                .write_all(b"body { margin: 0; }")
                .unwrap();
        }

        // 20 Media files
        for i in 1..=20 {
            File::create(media_dir.join(format!("font_{}.woff2", i)))
                .unwrap()
                .write_all(b"woff2_binary_data")
                .unwrap();
        }

        // 30 Team images
        for i in 1..=30 {
            File::create(team_dir.join(format!("member_{}.jpg", i)))
                .unwrap()
                .write_all(b"jpg_binary_header")
                .unwrap();
        }

        // 15 Icons
        for i in 1..=15 {
            File::create(icons_dir.join(format!("icon_{}.svg", i)))
                .unwrap()
                .write_all(b"<svg></svg>")
                .unwrap();
        }

        // 5 Asset CSS files
        for i in 1..=5 {
            File::create(css_assets_dir.join(format!("theme_{}.css", i)))
                .unwrap()
                .write_all(b":root {}")
                .unwrap();
        }

        // 3 Locales ES + 3 Locales EN
        for i in 1..=3 {
            File::create(locales_es.join(format!("msg_{}.json", i)))
                .unwrap()
                .write_all(b"{}")
                .unwrap();
            File::create(locales_en.join(format!("msg_{}.json", i)))
                .unwrap()
                .write_all(b"{}")
                .unwrap();
        }

        let plan = scan_local_upload_items(&[root.to_str().unwrap().to_string()]).unwrap();
        assert_eq!(plan.total_files, 131, "Total de archivos descubiertos debe ser exactamente 131");
        assert_eq!(plan.total_dirs, 14, "Total de directorios descubiertos debe ser exactamente 14");
        assert!(plan.total_bytes > 0, "Total de bytes debe ser superior a 0");

        // Certificar que NINGUNA ruta relativa contiene backslashes '\'
        for item in &plan.items {
            assert!(!item.relative_path.contains('\\'), "Ruta relativa contiene backslash: {}", item.relative_path);
        }

        // Certificar que los archivos son abribles sin 'os error 5'
        for item in &plan.items {
            if !item.is_dir {
                let open_res = File::open(&item.local_path);
                assert!(open_res.is_ok(), "Fallo al abrir archivo local: {}", item.local_path);
            }
        }
    }
}
