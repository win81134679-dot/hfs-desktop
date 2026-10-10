//! 內嵌在 exe 裡的資源：分享網頁外掛（drive-ui）與預設設定。
//! 這樣只要一個 exe：第一次執行自動下載 HFS，再把這些寫進去。

use std::path::{Path, PathBuf};

const PLUGIN_FILES: &[(&str, &[u8])] = &[
    ("plugin.js", include_bytes!("../../hfs/plugins/drive-ui/plugin.js")),
    ("public/index.html", include_bytes!("../../hfs/plugins/drive-ui/public/index.html")),
    ("public/app.js", include_bytes!("../../hfs/plugins/drive-ui/public/app.js")),
    ("public/app.css", include_bytes!("../../hfs/plugins/drive-ui/public/app.css")),
    ("public/icon.svg", include_bytes!("../../hfs/plugins/drive-ui/public/icon.svg")),
];

/// 把內嵌的外掛寫進 hfs\plugins\drive-ui\（內容不同才寫，HFS 會自動重新載入）。
/// 回傳實際更新了幾個檔案。外掛自己的 storage 資料夾不動。
pub fn sync_plugin(hfs_dir: &Path) -> std::io::Result<usize> {
    let base = hfs_dir.join("plugins").join("drive-ui");
    let mut changed = 0;
    for (rel, bytes) in PLUGIN_FILES {
        let path = base.join(rel);
        if std::fs::read(&path).ok().as_deref() == Some(*bytes) {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, bytes)?;
        changed += 1;
    }
    Ok(changed)
}

/// 預設分享資料夾：%USERPROFILE%\Downloads\HFS分享（不存在就建立）
pub fn default_share_dir() -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE")?;
    let dir = PathBuf::from(home).join("Downloads").join("HFS分享");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// 第一次使用時寫入的 config.yaml
pub fn default_config(port: u16) -> String {
    let mut text = format!(
        "port: {port}\n\
         open_browser_at_start: false\n\
         enable_plugins:\n  - antibrute\n  - drive-ui\n"
    );
    if let Some(dir) = default_share_dir() {
        // 結尾加 \ 讓 HFS 知道是資料夾；單引號字串裡反斜線不需要跳脫
        let src = format!("{}\\", dir.display()).replace('\'', "''");
        text += &format!("vfs:\n  source: '{src}'\n  can_upload: true\n");
    }
    text
}

/// 沒找到現成的 HFS 時，安裝到這裡：%LOCALAPPDATA%\HFS Desktop
pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("HFS Desktop")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_sync_is_idempotent() {
        let dir = std::env::temp_dir().join(format!("hfs-gui-embed-{}", std::process::id()));
        assert_eq!(sync_plugin(&dir).unwrap(), PLUGIN_FILES.len());
        assert_eq!(sync_plugin(&dir).unwrap(), 0);
        std::fs::write(dir.join("plugins/drive-ui/public/app.js"), b"old").unwrap();
        assert_eq!(sync_plugin(&dir).unwrap(), 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn config_has_plugin_and_port() {
        let c = default_config(8080);
        assert!(c.starts_with("port: 8080\n"));
        assert!(c.contains("- drive-ui"));
    }
}
