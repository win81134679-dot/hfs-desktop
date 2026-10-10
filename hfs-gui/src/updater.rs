//! HFS 更新：查 GitHub 最新正式版、下載 Windows 版 zip、解壓縮。
//! 節流（一天最多查一次、失敗退避）由呼叫端搭配 prefs 落地處理，這裡只負責動作本身。

use serde_json::{json, Value};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

const LATEST_API: &str = "https://api.github.com/repos/rejetto/hfs/releases/latest";
const USER_AGENT: &str = "hfs-gui (https://github.com/rejetto/hfs)";

#[derive(Clone, PartialEq, Debug)]
pub struct Release {
    pub tag: String,
    pub published: String,
    pub notes: String,
    pub zip_url: String,
    pub zip_size: u64,
}

impl Release {
    pub fn version(&self) -> Option<(u32, u32, u32)> {
        parse_version(&self.tag)
    }

    pub fn to_json(&self) -> Value {
        json!({ "tag": self.tag, "published": self.published, "notes": self.notes, "zip_url": self.zip_url, "zip_size": self.zip_size })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            tag: v["tag"].as_str()?.to_string(),
            published: v["published"].as_str().unwrap_or_default().to_string(),
            notes: v["notes"].as_str().unwrap_or_default().to_string(),
            zip_url: v["zip_url"].as_str()?.to_string(),
            zip_size: v["zip_size"].as_u64().unwrap_or(0),
        })
    }
}

/// "v3.3.4" / "3.3.4" → (3, 3, 4)；預覽版（帶 -beta 之類）回傳 None
pub fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim().trim_start_matches(['v', 'V']);
    if s.contains('-') {
        return None;
    }
    let mut it = s.split('.').map(|p| p.parse::<u32>().ok());
    Some((it.next()??, it.next().flatten().unwrap_or(0), it.next().flatten().unwrap_or(0)))
}

pub fn is_newer(latest: &Release, current: &str) -> bool {
    match (latest.version(), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

/// 查最新正式版。只在呼叫端判斷「到期」時呼叫，不要放進迴圈。
pub fn fetch_latest() -> Result<Release, String> {
    let resp = ureq::get(LATEST_API)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(20))
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(403, _) | ureq::Error::Status(429, _) => "GitHub 暫時限制查詢次數，稍後再試".to_string(),
            ureq::Error::Status(code, _) => format!("GitHub 回應 HTTP {code}"),
            other => format!("無法連線到 GitHub：{other}"),
        })?;
    let text = resp.into_string().map_err(|e| format!("讀取 GitHub 回應失敗：{e}"))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("GitHub 回應格式不對：{e}"))?;
    parse_release(&v)
}

fn parse_release(v: &Value) -> Result<Release, String> {
    let tag = v["tag_name"].as_str().ok_or("缺少版本號")?.to_string();
    let asset = v["assets"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|a| a["name"].as_str().is_some_and(|n| n.starts_with("hfs-windows-x64") && n.ends_with(".zip")))
        .ok_or("這個版本沒有 Windows x64 的下載檔")?;
    Ok(Release {
        tag,
        published: v["published_at"].as_str().unwrap_or_default().to_string(),
        notes: v["body"].as_str().unwrap_or_default().to_string(),
        zip_url: asset["browser_download_url"].as_str().ok_or("缺少下載網址")?.to_string(),
        zip_size: asset["size"].as_u64().unwrap_or(0),
    })
}

/// 同時幾條連線下載
const THREADS: usize = 8;

/// 多通道下載到 dest；伺服器不支援分段時退回單線。progress(已下載, 總大小)
pub fn download(url: &str, dest: &Path, progress: impl FnMut(u64, u64)) -> Result<(), String> {
    // 先用 Range 0-0 探測：拿到轉址後的真正網址、檔案大小、是否支援分段
    let probe = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .set("Range", "bytes=0-0")
        .timeout(Duration::from_secs(30))
        .call()
        .map_err(|e| format!("下載失敗：{e}"))?;
    let final_url = probe.get_url().to_string();
    let total: u64 = probe
        .header("Content-Range")
        .and_then(|v| v.rsplit('/').next())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let ranged = probe.status() == 206 && total > 0;
    drop(probe);
    if ranged {
        download_parallel(&final_url, total, dest, progress)
    } else {
        download_single(url, dest, progress)
    }
}

fn download_parallel(url: &str, total: u64, dest: &Path, mut progress: impl FnMut(u64, u64)) -> Result<(), String> {
    use std::os::windows::fs::FileExt;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    let tmp = dest.with_extension("part");
    let file = std::fs::File::create(&tmp).map_err(|e| format!("無法寫入 {}：{e}", tmp.display()))?;
    file.set_len(total).map_err(|e| format!("磁碟空間不足？{e}"))?;
    let file = Arc::new(file);
    let chunk = (total / (THREADS as u64 * 3)).clamp(1 << 20, 16 << 20);
    let mut ranges: Vec<(u64, u64)> = (0..total).step_by(chunk as usize).map(|s| (s, (s + chunk).min(total) - 1)).collect();
    ranges.reverse(); // pop() 從頭開始拿
    let queue = Arc::new(Mutex::new(ranges));
    let done = Arc::new(AtomicU64::new(0));
    let running = Arc::new(AtomicUsize::new(THREADS));
    let error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let mut handles = Vec::new();
    for _ in 0..THREADS {
        let (file, queue, done, running, error, url) = (file.clone(), queue.clone(), done.clone(), running.clone(), error.clone(), url.to_string());
        handles.push(std::thread::spawn(move || {
            'next: loop {
                if error.lock().unwrap().is_some() {
                    break;
                }
                let Some((a, b)) = queue.lock().unwrap().pop() else { break };
                for attempt in 1..=4u64 {
                    let mut written = 0u64;
                    let res = (|| -> Result<(), String> {
                        let resp = ureq::get(&url)
                            .set("User-Agent", USER_AGENT)
                            .set("Range", &format!("bytes={a}-{b}"))
                            .timeout(Duration::from_secs(120))
                            .call()
                            .map_err(|e| e.to_string())?;
                        if resp.status() != 206 {
                            return Err(format!("HTTP {}", resp.status()));
                        }
                        let mut reader = resp.into_reader();
                        let mut buf = vec![0u8; 128 * 1024];
                        loop {
                            let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
                            if n == 0 {
                                break;
                            }
                            file.seek_write(&buf[..n], a + written).map_err(|e| e.to_string())?;
                            written += n as u64;
                            done.fetch_add(n as u64, Ordering::Relaxed);
                        }
                        if written != b - a + 1 {
                            return Err("片段不完整".into());
                        }
                        Ok(())
                    })();
                    match res {
                        Ok(()) => continue 'next,
                        Err(e) => {
                            done.fetch_sub(written, Ordering::Relaxed);
                            if attempt == 4 {
                                *error.lock().unwrap() = Some(format!("下載失敗：{e}"));
                                break 'next;
                            }
                            std::thread::sleep(Duration::from_millis(800 * attempt)); // 退避後重試這一段
                        }
                    }
                }
            }
            running.fetch_sub(1, Ordering::Relaxed);
        }));
    }
    while running.load(Ordering::Relaxed) > 0 {
        std::thread::sleep(Duration::from_millis(200));
        progress(done.load(Ordering::Relaxed), total);
    }
    for h in handles {
        let _ = h.join();
    }
    drop(file);
    if let Some(e) = error.lock().unwrap().take() {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    let got = done.load(Ordering::Relaxed);
    if got != total {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("下載不完整（{got} / {total} 位元組）"));
    }
    progress(total, total);
    std::fs::rename(&tmp, dest).map_err(|e| format!("無法存檔：{e}"))
}

fn download_single(url: &str, dest: &Path, mut progress: impl FnMut(u64, u64)) -> Result<(), String> {
    let resp = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .timeout(Duration::from_secs(600))
        .call()
        .map_err(|e| format!("下載失敗：{e}"))?;
    let total = resp.header("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut reader = resp.into_reader();
    let tmp = dest.with_extension("part");
    let mut file = std::fs::File::create(&tmp).map_err(|e| format!("無法寫入 {}：{e}", tmp.display()))?;
    let mut buf = vec![0u8; 256 * 1024];
    let mut done = 0u64;
    loop {
        let n = reader.read(&mut buf).map_err(|e| format!("下載中斷：{e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| format!("寫入失敗：{e}"))?;
        done += n as u64;
        progress(done, total);
    }
    drop(file);
    if total > 0 && done != total {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("下載不完整（{done} / {total} 位元組）"));
    }
    std::fs::rename(&tmp, dest).map_err(|e| format!("無法存檔：{e}"))
}

/// 檢查 zip 裡有沒有 hfs.exe
pub fn verify(zip_path: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("壓縮檔損壞：{e}"))?;
    archive.by_name("hfs.exe").map(|_| ()).map_err(|_| "壓縮檔裡沒有 hfs.exe".to_string())
}

/// 解壓縮到 dir（覆蓋同名檔；config.yaml 等使用者檔案不在壓縮檔裡，不會被動到）
pub fn extract(zip_path: &Path, dir: &Path) -> Result<Vec<PathBuf>, String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("壓縮檔損壞：{e}"))?;
    let mut written = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let Some(rel) = entry.enclosed_name() else { continue }; // 擋掉 ../ 之類的路徑
        let out = dir.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut f = std::fs::File::create(&out).map_err(|e| format!("無法寫入 {}：{e}", out.display()))?;
        std::io::copy(&mut entry, &mut f).map_err(|e| format!("解壓縮失敗：{e}"))?;
        written.push(out);
    }
    Ok(written)
}

/// 發布說明只取前面幾行，去掉 markdown 標記
pub fn short_notes(notes: &str, max_lines: usize) -> String {
    notes
        .lines()
        .map(|l| l.trim().trim_start_matches('#').trim().trim_start_matches(['-', '*']).trim())
        .filter(|l| !l.is_empty())
        .take(max_lines)
        .map(|l| format!("• {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert_eq!(parse_version("v3.3.4"), Some((3, 3, 4)));
        assert_eq!(parse_version("3.10"), Some((3, 10, 0)));
        assert_eq!(parse_version("v3.4.0-beta2"), None);
        let r = Release { tag: "v3.4.0".into(), published: String::new(), notes: String::new(), zip_url: "x".into(), zip_size: 0 };
        assert!(is_newer(&r, "3.3.4"));
        assert!(!is_newer(&r, "3.4.0"));
        assert!(!is_newer(&r, "3.10.1"));
    }

    /// 真的連 GitHub 下載（約 43 MB）。只手動跑：cargo test -- --ignored real_download --nocapture
    #[test]
    #[ignore]
    fn real_download() {
        let url = "https://github.com/rejetto/hfs/releases/download/v3.3.4/hfs-windows-x64-3.3.4.zip";
        let dest = std::env::temp_dir().join(format!("hfs-gui-dl-test-{}.zip", std::process::id()));
        let t = std::time::Instant::now();
        let mut last = 0;
        download(url, &dest, |d, total| {
            if d - last > 8 << 20 || d == total {
                println!("{:>5.1}s  {:.1} / {:.1} MB", t.elapsed().as_secs_f32(), d as f64 / 1048576.0, total as f64 / 1048576.0);
                last = d;
            }
        })
        .unwrap();
        let secs = t.elapsed().as_secs_f64();
        let size = std::fs::metadata(&dest).unwrap().len();
        println!("{size} bytes in {secs:.1}s = {:.2} MB/s", size as f64 / 1048576.0 / secs);
        assert_eq!(size, 43293600);
        verify(&dest).unwrap();
        std::fs::remove_file(&dest).unwrap();
    }

    #[test]
    fn release_json() {
        let v = json!({
            "tag_name": "v3.3.4", "published_at": "2026-09-30T13:48:20Z", "body": "## Fixes\n- a\n- b",
            "assets": [
                { "name": "hfs-linux-x64-3.3.4.zip", "browser_download_url": "L", "size": 1 },
                { "name": "hfs-windows-x64-3.3.4.zip", "browser_download_url": "W", "size": 43293600 }
            ]
        });
        let r = parse_release(&v).unwrap();
        assert_eq!(r.zip_url, "W");
        assert_eq!(r.zip_size, 43293600);
        assert_eq!(Release::from_json(&r.to_json()), Some(r.clone()));
        assert_eq!(short_notes(&r.notes, 5), "• Fixes\n• a\n• b");
    }
}
