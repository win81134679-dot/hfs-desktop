# 第三方元件與致謝

本專案以 [GPL-3.0](LICENSE) 授權釋出。下列第三方元件依其各自的授權使用：

| 元件 | 用途 | 授權 |
|---|---|---|
| [HFS – HTTP File Server](https://github.com/rejetto/hfs)（rejetto） | 實際提供檔案分享的伺服器；本專案透過其管理 API 與外掛機制運作，**不包含** HFS 本體 | GPL-3.0 |
| [Material Icons](https://github.com/google/material-design-icons)（Google） | 桌面程式與分享網頁的圖示路徑 | Apache-2.0 |
| [egui / eframe / egui_extras](https://github.com/emilk/egui) | 桌面介面 | MIT / Apache-2.0 |
| [tray-icon](https://github.com/tauri-apps/tray-icon) | 系統匣圖示 | MIT / Apache-2.0 |
| [ureq](https://github.com/algesten/ureq) | HTTP 用戶端 | MIT / Apache-2.0 |
| [zip](https://github.com/zip-rs/zip2) | 解壓縮更新檔 | MIT |
| [qrcode](https://github.com/kennytm/qrcode-rust) | QR Code | MIT / Apache-2.0 |
| [rfd](https://github.com/PolyMeilex/rfd) | 檔案選擇對話框 | MIT |
| [serde_json](https://github.com/serde-rs/json)、[chrono](https://github.com/chronotope/chrono)、[windows-sys](https://github.com/microsoft/windows-rs)、[winresource](https://github.com/BenjaminRi/winresource) | 其他 | MIT / Apache-2.0 |
| [Inno Setup](https://jrsoftware.org/isinfo.php)（Jordan Russell、Martijn Laan） | 安裝程式；`scripts/ChineseTraditional.isl` 為其非官方繁體中文翻譯 | Inno Setup License |

介面字型使用 Windows 內建的 Segoe UI 與微軟正黑體，不隨本專案散布。

本專案為個人製作的非官方工具，與 HFS 作者無關。
