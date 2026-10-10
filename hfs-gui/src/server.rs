//! 管理 hfs.exe 子程序，並把 API / SSE 的資料收進共用狀態給介面畫。
//!
//! 每次啟動/換埠都會遞增 `gen`，舊世代的背景執行緒看到 gen 變了就自己結束。

use crate::{api, embedded, updater};
use eframe::egui;
use serde_json::{json, Map, Value};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
pub const DEFAULT_PORT: u16 = 8080;
const MAX_LOG: usize = 3000;

#[derive(Clone, PartialEq)]
pub enum Status {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed(String),
}

impl Status {
    /// 伺服器正在跑或正在起來（關視窗時要詢問的狀態）
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Starting | Status::Running)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum LogKind {
    Info,
    Console,
    Request,
    Error,
}

pub struct LogLine {
    pub time: String,
    pub kind: LogKind,
    pub text: String,
    /// 訪客請求才有，介面用來分欄顯示
    pub req: Option<Req>,
    at: Instant,
}

pub struct Req {
    pub ip: String,
    pub method: String,
    pub path: String,
    pub status: u64,
    pub size: Option<f64>,
    pub user: Option<String>,
    /// 多通道下載：同一個檔案的分段請求（206）合併成一列，這裡記段數
    pub parts: u32,
}

#[derive(Default, Clone)]
pub struct Stats {
    pub out_kb: f64,
    pub in_kb: f64,
    pub sent: f64,
    pub got: f64,
}

pub struct Shared {
    pub port: u16,
    pub status: Status,
    /// true = 接管了不是我們開的 HFS
    pub attached: bool,
    pub urls: Vec<String>,
    pub vfs: Option<Value>,
    pub log: VecDeque<LogLine>,
    pub conns: Vec<Map<String, Value>>,
    pub stats: Stats,
    /// 最近 120 秒的 (送出, 接收) KB/s，畫流量圖用
    pub history: VecDeque<(f32, f32)>,
    /// get_accounts 的結果
    pub accounts: Vec<Value>,
    /// get_config 的結果
    pub config: Option<Value>,
    /// 正在跑的 HFS 版本（get_status）
    pub hfs_version: Option<String>,
    pub update: UpdateInfo,
    gen: u64,
}

#[derive(Default)]
pub struct UpdateInfo {
    pub checking: bool,
    /// 檢查結果，介面讀走後清掉
    pub result: Option<Result<updater::Release, String>>,
    pub phase: UpdatePhase,
    /// 正在進行的是第一次安裝（不是升級），介面用不同的文字
    pub first_install: bool,
}

#[derive(Default, Clone, PartialEq)]
pub enum UpdatePhase {
    #[default]
    Idle,
    Downloading(u64, u64),
    Installing,
    Restarting,
    Done(String),
    Failed(String),
}

impl Shared {
    pub fn push_log(&mut self, kind: LogKind, text: impl Into<String>) {
        self.push_log_at(now(), kind, text);
    }

    fn push_log_at(&mut self, time: String, kind: LogKind, text: impl Into<String>) {
        self.push_line(LogLine { time, kind, text: text.into(), req: None, at: Instant::now() });
    }

    fn push_line(&mut self, line: LogLine) {
        if self.log.len() >= MAX_LOG {
            self.log.pop_front();
        }
        self.log.push_back(line);
    }

    fn reset_live(&mut self) {
        self.conns.clear();
        self.stats = Stats::default();
        self.urls.clear();
        self.history.clear();
        self.hfs_version = None;
    }

    /// 帳號名稱清單（不含群組以外的資訊）
    pub fn usernames(&self) -> Vec<String> {
        self.accounts.iter().filter_map(|a| a["username"].as_str().map(str::to_owned)).collect()
    }
}

#[derive(Clone)]
pub struct Server {
    pub shared: Arc<Mutex<Shared>>,
    child: Arc<Mutex<Option<Child>>>,
    ctx: egui::Context,
    pub hfs_dir: PathBuf,
}

impl Server {
    pub fn new(ctx: egui::Context, hfs_dir: PathBuf) -> Self {
        let port = read_port(&hfs_dir.join("config.yaml")).unwrap_or(DEFAULT_PORT);
        Self {
            shared: Arc::new(Mutex::new(Shared {
                port,
                status: Status::Stopped,
                attached: false,
                urls: Vec::new(),
                vfs: None,
                log: VecDeque::new(),
                conns: Vec::new(),
                stats: Stats::default(),
                history: VecDeque::new(),
                accounts: Vec::new(),
                config: None,
                hfs_version: None,
                update: UpdateInfo::default(),
                gen: 0,
            })),
            child: Arc::new(Mutex::new(None)),
            ctx,
            hfs_dir,
        }
    }

    fn s(&self) -> MutexGuard<'_, Shared> {
        self.shared.lock().unwrap()
    }

    pub fn log(&self, kind: LogKind, text: impl Into<String>) {
        self.s().push_log(kind, text);
        self.ctx.request_repaint();
    }

    pub fn port(&self) -> u16 {
        self.s().port
    }

    fn current(&self, gen: u64) -> bool {
        self.s().gen == gen
    }

    fn alive(&self, gen: u64) -> bool {
        let s = self.s();
        s.gen == gen && s.status == Status::Running
    }

    fn fail(&self, gen: u64, msg: String) {
        {
            let mut s = self.s();
            if s.gen == gen {
                s.status = Status::Failed(msg.clone());
                s.gen += 1;
                s.reset_live();
            }
            s.push_log(LogKind::Error, msg);
        }
        self.ctx.request_repaint();
    }

    // ---------- 啟動 / 停止 ----------

    pub fn start(&self) {
        let gen = {
            let mut s = self.s();
            if matches!(s.status, Status::Starting | Status::Running | Status::Stopping) {
                return;
            }
            s.gen += 1;
            s.status = Status::Starting;
            s.push_log(LogKind::Info, "正在啟動伺服器…");
            s.gen
        };
        self.ctx.request_repaint();
        let me = self.clone();
        thread::spawn(move || me.start_thread(gen));
    }

    fn start_thread(&self, gen: u64) {
        let port = self.port();
        if api::get_status(port).is_ok() {
            self.s().attached = true;
            self.log(LogKind::Info, format!("埠 {port} 已經有 HFS 在運行，直接接管"));
        } else {
            if let Err(e) = self.spawn_child() {
                self.fail(gen, e);
                return;
            }
            self.s().attached = false;
            if !self.current(gen) {
                // 啟動途中被按了停止
                self.kill_child();
                return;
            }
        }
        self.wait_ready(gen, port, Duration::from_secs(30));
    }

    fn spawn_child(&self) -> Result<(), String> {
        let exe = self.hfs_dir.join("hfs.exe");
        if !exe.is_file() {
            self.install_hfs()?;
        }
        // 內嵌的分享網頁同步到 HFS（換成新版 exe 時網頁也跟著更新）
        match embedded::sync_plugin(&self.hfs_dir) {
            Ok(0) => {}
            Ok(n) => self.log(LogKind::Info, format!("已更新雲端硬碟風格分享網頁（{n} 個檔案）")),
            Err(e) => self.log(LogKind::Error, format!("無法寫入分享網頁外掛：{e}")),
        }
        // 第一次使用：建立預設設定（8080 埠、分享網頁、下載\HFS分享、區網可上傳）
        let config = self.hfs_dir.join("config.yaml");
        if !config.exists() {
            match std::fs::write(&config, embedded::default_config(DEFAULT_PORT)) {
                Ok(()) => self.log(LogKind::Info, "已建立預設設定 config.yaml（分享資料夾：下載\\HFS分享）"),
                Err(e) => self.log(LogKind::Error, format!("無法建立 config.yaml：{e}")),
            }
        }
        let mut child = Command::new(&exe)
            .current_dir(&self.hfs_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("無法啟動 hfs.exe：{e}"))?;
        // 一定要持續讀，不然管線塞滿 hfs.exe 會卡住
        if let Some(out) = child.stdout.take() {
            self.pipe(out, LogKind::Console);
        }
        if let Some(err) = child.stderr.take() {
            self.pipe(err, LogKind::Error);
        }
        *self.child.lock().unwrap() = Some(child);
        Ok(())
    }

    fn pipe<R: Read + Send + 'static>(&self, reader: R, kind: LogKind) {
        let me = self.clone();
        thread::spawn(move || {
            for line in BufReader::new(reader).split(b'\n').map_while(Result::ok) {
                let line = strip_ansi(String::from_utf8_lossy(&line).trim_end());
                if line.trim().is_empty() {
                    continue;
                }
                // hfs 的輸出自帶 "HH:MM:SS " 開頭
                let b = line.as_bytes();
                let (time, text) = if b.len() > 9 && b[2] == b':' && b[5] == b':' && b[8] == b' ' {
                    (line[..8].to_string(), line[9..].to_string())
                } else {
                    (now(), line)
                };
                // 分享清單被別處改過（網頁管理介面、config.yaml）時 hfs 會印這行
                let vfs_changed = text == "VFS ready";
                let running = {
                    let mut s = me.s();
                    s.push_log_at(time, kind, text);
                    s.status == Status::Running
                };
                if vfs_changed && running {
                    me.refresh_vfs();
                }
                me.ctx.request_repaint();
            }
        });
    }

    /// 等 API 有回應就進入運行狀態，並開始收即時資料
    fn wait_ready(&self, gen: u64, port: u16, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        loop {
            if !self.current(gen) {
                return;
            }
            if let Some(code) = self.child_exited() {
                self.fail(gen, format!("HFS 啟動失敗（結束代碼 {code}），請看上面的記錄"));
                return;
            }
            if let Ok(st) = api::get_status(port) {
                self.apply_status(&st);
                {
                    let mut s = self.s();
                    if s.gen != gen {
                        return;
                    }
                    s.status = Status::Running;
                    s.push_log(LogKind::Info, format!("伺服器已啟動，埠 {port}"));
                }
                self.refresh_vfs();
                self.refresh_accounts();
                self.refresh_config();
                self.spawn_streams(gen);
                self.spawn_supervisor(gen);
                self.ctx.request_repaint();
                return;
            }
            if Instant::now() > deadline {
                self.fail(gen, format!("HFS 沒有回應，埠 {port} 可能被其他程式佔用"));
                self.kill_child();
                return;
            }
            thread::sleep(Duration::from_millis(300));
        }
    }

    pub fn stop(&self, on_done: Option<Box<dyn FnOnce() + Send>>) {
        let me = self.clone();
        thread::spawn(move || {
            me.stop_blocking();
            if let Some(f) = on_done {
                f();
            }
        });
    }

    /// 請 HFS 自己結束（讓它把設定寫回檔案），等不到才強制結束
    pub fn stop_blocking(&self) {
        {
            let mut s = self.s();
            s.gen += 1;
            s.status = Status::Stopping;
            s.push_log(LogKind::Info, "正在停止伺服器…");
        }
        self.ctx.request_repaint();
        let _ = api::call_timeout(self.port(), "quit", &json!({}), Duration::from_secs(3));
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let done = match self.child.lock().unwrap().as_mut() {
                None => true,
                Some(c) => matches!(c.try_wait(), Ok(Some(_))) || Instant::now() > deadline,
            };
            if done {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
        self.kill_child();
        // 接管模式沒有子程序可等，確認 API 不再回應
        for _ in 0..30 {
            if api::get_status(self.port()).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(200));
        }
        {
            let mut s = self.s();
            s.status = Status::Stopped;
            s.attached = false;
            s.reset_live();
            s.push_log(LogKind::Info, "伺服器已停止");
        }
        self.ctx.request_repaint();
    }

    /// 單一 exe 模式：還沒有 HFS 時，從 GitHub 下載最新正式版安裝到 hfs_dir
    fn install_hfs(&self) -> Result<(), String> {
        let set = |p: UpdatePhase| {
            self.s().update.phase = p;
            self.ctx.request_repaint();
        };
        {
            let mut s = self.s();
            s.update.first_install = true;
            s.update.phase = UpdatePhase::Downloading(0, 0);
            s.push_log(LogKind::Info, "第一次使用：正在從 GitHub 下載最新版 HFS…");
        }
        self.ctx.request_repaint();
        let result = (|| -> Result<updater::Release, String> {
            let rel = updater::fetch_latest()?;
            let zip = self.hfs_dir.join("hfs-update.zip");
            updater::download(&rel.zip_url, &zip, |d, t| set(UpdatePhase::Downloading(d, if t > 0 { t } else { rel.zip_size })))?;
            updater::verify(&zip)?;
            set(UpdatePhase::Installing);
            updater::extract(&zip, &self.hfs_dir)?;
            let _ = std::fs::remove_file(&zip);
            Ok(rel)
        })();
        match result {
            Ok(rel) => {
                let mut s = self.s();
                s.update.first_install = false;
                s.update.phase = UpdatePhase::Idle;
                // 交給介面記下查到的版本，免得一啟動又去查 GitHub
                s.update.result = Some(Ok(rel.clone()));
                s.push_log(LogKind::Info, format!("已安裝 HFS {}（{}）", rel.tag, self.hfs_dir.display()));
                Ok(())
            }
            Err(e) => {
                let msg = format!("下載 HFS 失敗：{e}");
                set(UpdatePhase::Failed(msg.clone()));
                Err(msg)
            }
        }
    }

    // ---------- 更新 ----------

    /// 查 GitHub 最新版（呼叫端負責節流），結果放在 shared.update.result
    pub fn check_update(&self) {
        {
            let mut s = self.s();
            if s.update.checking {
                return;
            }
            s.update.checking = true;
        }
        let me = self.clone();
        thread::spawn(move || {
            let result = updater::fetch_latest();
            let mut s = me.s();
            s.update.checking = false;
            s.update.result = Some(result);
            drop(s);
            me.ctx.request_repaint();
        });
    }

    pub fn clear_update_phase(&self) {
        let mut s = self.s();
        s.update.phase = UpdatePhase::Idle;
        s.update.first_install = false;
    }

    /// 下載 → 停止 HFS → 備份舊版 → 解壓縮新版 → 重新啟動並確認版本；任一步失敗都還原舊版
    pub fn upgrade(&self, rel: updater::Release) {
        {
            let mut s = self.s();
            if !matches!(s.update.phase, UpdatePhase::Idle | UpdatePhase::Done(_) | UpdatePhase::Failed(_)) {
                return;
            }
            s.update.phase = UpdatePhase::Downloading(0, rel.zip_size);
            s.push_log(LogKind::Info, format!("開始下載 HFS {}", rel.tag));
        }
        self.ctx.request_repaint();
        let me = self.clone();
        thread::spawn(move || {
            let set = |p: UpdatePhase| {
                me.s().update.phase = p;
                me.ctx.request_repaint();
            };
            let fail = |msg: String| {
                me.log(LogKind::Error, format!("升級失敗：{msg}"));
                set(UpdatePhase::Failed(msg));
            };
            let dir = me.hfs_dir.clone();
            let zip = dir.join("hfs-update.zip");
            let exe = dir.join("hfs.exe");
            let old = dir.join("hfs.exe.old");

            if let Err(e) = updater::download(&rel.zip_url, &zip, |d, t| set(UpdatePhase::Downloading(d, if t > 0 { t } else { rel.zip_size }))) {
                return fail(e);
            }
            if let Err(e) = updater::verify(&zip) {
                let _ = std::fs::remove_file(&zip);
                return fail(e);
            }
            set(UpdatePhase::Installing);
            me.stop_blocking();
            let _ = std::fs::remove_file(&old);
            if let Err(e) = std::fs::rename(&exe, &old) {
                me.start();
                return fail(format!("無法備份舊版 hfs.exe：{e}"));
            }
            if let Err(e) = updater::extract(&zip, &dir) {
                let _ = std::fs::remove_file(&exe);
                let _ = std::fs::rename(&old, &exe);
                me.start();
                return fail(format!("{e}（已還原舊版）"));
            }
            let _ = std::fs::remove_file(&zip);

            set(UpdatePhase::Restarting);
            me.start();
            let deadline = Instant::now() + Duration::from_secs(45);
            let ok = loop {
                {
                    let s = me.s();
                    if s.status == Status::Running {
                        break s.hfs_version.as_deref().and_then(updater::parse_version) == rel.version();
                    }
                    if matches!(s.status, Status::Failed(_)) {
                        break false;
                    }
                }
                if Instant::now() > deadline {
                    break false;
                }
                thread::sleep(Duration::from_millis(300));
            };
            if ok {
                me.log(LogKind::Info, format!("已升級到 HFS {}（舊版備份在 hfs.exe.old）", rel.tag));
                set(UpdatePhase::Done(format!("已升級到 HFS {}", rel.tag)));
            } else {
                me.stop_blocking();
                let _ = std::fs::remove_file(&exe);
                let _ = std::fs::rename(&old, &exe);
                me.start();
                fail("新版沒有正常啟動，已還原成舊版".into());
            }
        });
    }

    pub fn kill_child(&self) {
        if let Some(mut c) = self.child.lock().unwrap().take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }

    /// 子程序結束了就回傳結束代碼（並放掉 handle）
    fn child_exited(&self) -> Option<i32> {
        let mut guard = self.child.lock().unwrap();
        let code = guard.as_mut()?.try_wait().ok().flatten()?.code().unwrap_or(-1);
        *guard = None;
        Some(code)
    }

    /// 監看子程序有沒有掛掉，順便定期更新網址
    fn spawn_supervisor(&self, gen: u64) {
        let me = self.clone();
        thread::spawn(move || {
            let mut tick = 0u32;
            loop {
                thread::sleep(Duration::from_secs(1));
                if !me.current(gen) {
                    return;
                }
                if let Some(code) = me.child_exited() {
                    me.fail(gen, format!("HFS 意外結束（結束代碼 {code}）"));
                    return;
                }
                tick += 1;
                if tick % 20 == 0 {
                    match api::get_status(me.port()) {
                        Ok(st) => {
                            me.apply_status(&st);
                            me.refresh_vfs();
                        }
                        Err(_) if me.s().attached => {
                            me.fail(gen, "和 HFS 的連線中斷了".into());
                            return;
                        }
                        Err(_) => {}
                    }
                }
            }
        });
    }

    fn apply_status(&self, st: &Value) {
        let mut urls: Vec<String> = ["http", "https"]
            .iter()
            .filter_map(|k| st["urls"][k].as_array())
            .flatten()
            .filter_map(|u| u.as_str().map(str::to_owned))
            .collect();
        // IPv4 排前面，比較好念給別人聽
        urls.sort_by_key(|u| u.contains('['));
        let mut s = self.s();
        s.urls = urls;
        if let Some(v) = st["version"].as_str() {
            s.hfs_version = Some(v.to_string());
        }
        drop(s);
        self.ctx.request_repaint();
    }

    // ---------- 即時資料 ----------

    fn spawn_streams(&self, gen: u64) {
        self.stream(gen, "get_connections", |me| me.s().conns.clear(), Server::on_conns);
        self.stream(gen, "get_connection_stats", |_| {}, Server::on_stats);
        // 4xx/5xx 的請求 HFS 會改寫到 error_log，兩個都要收
        self.stream(gen, "get_log?file=log%7Cerror_log", |_| {}, Server::on_log);
    }

    fn stream(&self, gen: u64, path: &'static str, on_connect: fn(&Server), on_data: fn(&Server, Value)) {
        let me = self.clone();
        thread::spawn(move || {
            while me.alive(gen) {
                on_connect(&me);
                let _ = api::sse(me.port(), path, |v| {
                    if !me.alive(gen) {
                        return false;
                    }
                    on_data(&me, v);
                    me.ctx.request_repaint();
                    true
                });
                // 斷線一秒後重連
                for _ in 0..10 {
                    if !me.alive(gen) {
                        return;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });
    }

    /// list 協定：[[op, a, b], ...]，op 是 + - = props ready e
    fn on_conns(&self, v: Value) {
        let mut s = self.s();
        for (op, a, b) in list_ops(&v) {
            match op {
                "+" => s.conns.extend(list_items(a).cloned()),
                "-" => {
                    if let Some(Value::Object(q)) = a {
                        s.conns.retain(|c| !matches_all(c, q));
                    }
                }
                "=" => {
                    if let (Some(Value::Object(q)), Some(Value::Object(change))) = (a, b) {
                        if let Some(c) = s.conns.iter_mut().find(|c| matches_all(c, q)) {
                            for (k, val) in change {
                                if val.is_null() {
                                    c.remove(k);
                                } else {
                                    c.insert(k.clone(), val.clone());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn on_stats(&self, v: Value) {
        let mut s = self.s();
        s.stats = Stats {
            out_kb: v["outSpeedKb"].as_f64().unwrap_or(0.0),
            in_kb: v["inSpeedKb"].as_f64().unwrap_or(0.0),
            sent: v["sent_got"][0].as_f64().unwrap_or(0.0),
            got: v["sent_got"][1].as_f64().unwrap_or(0.0),
        };
        let point = (s.stats.out_kb as f32, s.stats.in_kb as f32);
        if s.history.len() >= 120 {
            s.history.pop_front();
        }
        s.history.push_back(point);
    }

    fn on_log(&self, v: Value) {
        let mut s = self.s();
        for (op, a, _) in list_ops(&v) {
            if op != "+" {
                continue;
            }
            for rec in list_items(a) {
                let uri = rec.get("uri").and_then(Value::as_str).unwrap_or("");
                // /~/ 開頭是網頁介面自己的 API 和資源，不是使用者的動作
                if uri.starts_with("/~/") {
                    continue;
                }
                let ip = rec.get("ip").and_then(Value::as_str).unwrap_or("");
                let method = rec.get("method").and_then(Value::as_str).unwrap_or("");
                let status = rec.get("status").and_then(Value::as_u64).unwrap_or(0);
                let path = percent_decode(uri.split('?').next().unwrap_or(uri));
                let size = rec.get("length").and_then(Value::as_f64);
                let user = rec.get("user").and_then(Value::as_str).filter(|u| !u.is_empty()).map(str::to_owned);
                let mut text = format!("{ip}  {method} {path}  {status}");
                if let Some(len) = size {
                    text += &format!("  {}", fmt_bytes(len));
                }
                if let Some(user) = &user {
                    text += &format!("  （{user}）");
                }
                let time = rec
                    .get("ts")
                    .and_then(Value::as_str)
                    .and_then(|ts| chrono::DateTime::parse_from_rfc3339(ts).ok())
                    .map(|t| t.with_timezone(&chrono::Local).format("%H:%M:%S").to_string())
                    .unwrap_or_else(now);
                let kind = if status >= 400 { LogKind::Error } else { LogKind::Request };
                // 分段下載（206）：一分鐘內同一個 IP 抓同一個檔案的片段，併進同一列
                if status == 206 {
                    let merged = s.log.iter_mut().rev().take(300).find(|l| {
                        l.at.elapsed() < Duration::from_secs(60)
                            && l.req.as_ref().is_some_and(|r| r.status == 206 && r.ip == ip && r.path == path)
                    });
                    if let Some(line) = merged {
                        let r = line.req.as_mut().unwrap();
                        r.parts += 1;
                        r.size = Some(r.size.unwrap_or(0.0) + size.unwrap_or(0.0));
                        line.at = Instant::now();
                        line.text = format!("{ip}  {method} {path}  206  {}  （{} 段）", fmt_bytes(r.size.unwrap_or(0.0)), r.parts);
                        continue;
                    }
                }
                let req = Req { ip: ip.to_owned(), method: method.to_owned(), path, status, size, user, parts: 1 };
                s.push_line(LogLine { time, kind, text, req: Some(req), at: Instant::now() });
            }
        }
    }

    // ---------- 虛擬檔案系統 ----------

    pub fn refresh_vfs(&self) {
        match api::call(self.port(), "get_vfs", &json!({})) {
            Ok(v) => self.s().vfs = v.get("root").cloned(),
            Err(e) => self.log(LogKind::Error, format!("讀取分享清單失敗：{e}")),
        }
        self.ctx.request_repaint();
    }

    /// 在背景呼叫 API，失敗寫進記錄，最後重新讀分享清單
    fn vfs_op(&self, f: impl FnOnce(&Server, u16) -> Result<(), String> + Send + 'static) {
        let me = self.clone();
        thread::spawn(move || {
            if let Err(e) = f(&me, me.port()) {
                me.log(LogKind::Error, e);
            }
            me.refresh_vfs();
        });
    }

    pub fn add_paths(&self, parent: Option<String>, paths: Vec<PathBuf>) {
        self.vfs_op(move |me, port| {
            for p in paths {
                let mut body = json!({ "source": p.to_string_lossy() });
                if let Some(parent) = &parent {
                    body["parent"] = json!(parent);
                }
                let res = api::call(port, "add_vfs", &body).map_err(|e| format!("加入 {} 失敗：{e}", p.display()))?;
                let name = res["name"].as_str().unwrap_or_default();
                me.log(LogKind::Info, format!("已加入分享：{name}"));
            }
            Ok(())
        });
    }

    pub fn add_virtual(&self, parent: Option<String>, name: String) {
        self.vfs_op(move |me, port| {
            let mut body = json!({ "name": name });
            if let Some(parent) = &parent {
                body["parent"] = json!(parent);
            }
            api::call(port, "add_vfs", &body).map_err(|e| format!("新增虛擬資料夾失敗：{e}"))?;
            me.log(LogKind::Info, format!("已新增虛擬資料夾：{name}"));
            Ok(())
        });
    }

    pub fn remove(&self, uri: String) {
        self.vfs_op(move |me, port| {
            let res = api::call(port, "del_vfs", &json!({ "uris": [uri] })).map_err(|e| format!("移除失敗：{e}"))?;
            match res["errors"][0].as_i64() {
                Some(0) => {
                    me.log(LogKind::Info, format!("已從分享移除：{}", percent_decode(&uri)));
                    Ok(())
                }
                code => Err(format!("移除失敗（代碼 {code:?}）")),
            }
        });
    }

    /// 開 = 任何人可上傳；關 = 回到 HFS 預設的只有管理員
    pub fn set_upload(&self, uri: String, enabled: bool) {
        self.vfs_op(move |me, port| {
            let who = if enabled { json!(true) } else { json!("admin") };
            api::call(port, "set_vfs", &json!({ "uri": uri, "props": { "can_upload": who } }))
                .map_err(|e| format!("設定上傳權限失敗：{e}"))?;
            let state = if enabled { "允許任何人上傳" } else { "關閉上傳" };
            me.log(LogKind::Info, format!("{}：{state}", percent_decode(&uri)));
            Ok(())
        });
    }

    /// 把分享項目搬到另一個資料夾底下（只改分享結構，不動電腦上的檔案）
    pub fn move_vfs(&self, from: String, parent: String) {
        self.vfs_op(move |me, port| {
            api::call(port, "move_vfs", &json!({ "from": from, "parent": parent })).map_err(|e| {
                if e.starts_with("HTTP 409") {
                    "移動失敗：目的地已經有同名的項目".to_string()
                } else {
                    format!("移動失敗：{e}")
                }
            })?;
            me.log(LogKind::Info, format!("已把「{}」移到「{}」", percent_decode(&from), percent_decode(&parent)));
            Ok(())
        });
    }

    /// 一般化的 set_vfs：值為 null 代表恢復繼承
    pub fn set_vfs_props(&self, uri: String, props: Value, done_msg: String) {
        self.vfs_op(move |me, port| {
            api::call(port, "set_vfs", &json!({ "uri": uri, "props": props })).map_err(|e| format!("儲存屬性失敗：{e}"))?;
            me.log(LogKind::Info, done_msg);
            Ok(())
        });
    }

    // ---------- 帳號 ----------

    pub fn refresh_accounts(&self) {
        match api::call(self.port(), "get_accounts", &json!({})) {
            Ok(v) => {
                let mut list = v["list"].as_array().cloned().unwrap_or_default();
                list.sort_by_key(|a| a["username"].as_str().unwrap_or_default().to_lowercase());
                self.s().accounts = list;
            }
            Err(e) => self.log(LogKind::Error, format!("讀取帳號失敗：{e}")),
        }
        self.ctx.request_repaint();
    }

    /// 新增（is_new）或修改帳號；changes 裡的 password 是明文，HFS 會自己轉成 SRP
    pub fn save_account(&self, username: String, changes: Value, is_new: bool) {
        let me = self.clone();
        thread::spawn(move || {
            let port = me.port();
            let res = if is_new {
                let mut body = changes.clone();
                body["username"] = json!(username);
                api::call(port, "add_account", &body)
            } else {
                api::call(port, "set_account", &json!({ "username": username, "changes": changes }))
            };
            match res {
                Ok(_) => me.log(LogKind::Info, format!("{}帳號：{username}", if is_new { "已新增" } else { "已更新" })),
                Err(e) if e.starts_with("HTTP 409") => me.log(LogKind::Error, format!("帳號 {username} 已經存在")),
                Err(e) => me.log(LogKind::Error, format!("儲存帳號失敗：{e}")),
            }
            me.refresh_accounts();
        });
    }

    pub fn delete_account(&self, username: String) {
        let me = self.clone();
        thread::spawn(move || {
            match api::call(me.port(), "del_account", &json!({ "username": username })) {
                Ok(_) => me.log(LogKind::Info, format!("已刪除帳號：{username}")),
                Err(e) => me.log(LogKind::Error, format!("刪除帳號失敗：{e}")),
            }
            me.refresh_accounts();
        });
    }

    // ---------- 設定 ----------

    pub fn refresh_config(&self) {
        match api::call(self.port(), "get_config", &json!({})) {
            Ok(v) => self.s().config = Some(v),
            Err(e) => self.log(LogKind::Error, format!("讀取設定失敗：{e}")),
        }
        self.ctx.request_repaint();
    }

    /// 寫入 HFS 設定；值為 null 代表恢復預設
    pub fn save_config(&self, values: Value, done_msg: &'static str) {
        let me = self.clone();
        thread::spawn(move || {
            match api::call(me.port(), "set_config", &json!({ "values": values })) {
                Ok(_) => me.log(LogKind::Info, done_msg),
                Err(e) => me.log(LogKind::Error, format!("儲存設定失敗：{e}")),
            }
            me.refresh_config();
        });
    }

    pub fn block_ip(&self, ip: String) {
        let me = self.clone();
        thread::spawn(move || {
            let port = me.port();
            match api::call(port, "add_block", &json!({ "ip": ip, "comment": "從 HFS 介面封鎖" })) {
                Ok(_) => {
                    let _ = api::call(port, "disconnect", &json!({ "ip": ip }));
                    me.log(LogKind::Info, format!("已封鎖 {ip}"));
                }
                Err(e) => me.log(LogKind::Error, format!("封鎖失敗：{e}")),
            }
            me.refresh_config();
        });
    }

    pub fn unblock_ip(&self, ip: String) {
        let rules: Vec<Value> = self
            .s()
            .config
            .as_ref()
            .and_then(|c| c["block"].as_array().cloned())
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r["ip"].as_str() != Some(ip.as_str()))
            .collect();
        self.save_config(json!({ "block": rules }), "已解除封鎖");
    }

    pub fn disconnect_all(&self) {
        let me = self.clone();
        thread::spawn(move || match api::call(me.port(), "disconnect", &json!({ "allButLocalhost": true })) {
            Ok(v) => me.log(LogKind::Info, format!("已中斷 {} 個連線", v["result"].as_u64().unwrap_or(0))),
            Err(e) => me.log(LogKind::Error, format!("中斷連線失敗：{e}")),
        });
    }

    pub fn disconnect(&self, ip: String, port: u16) {
        let me = self.clone();
        thread::spawn(move || {
            match api::call(me.port(), "disconnect", &json!({ "ip": ip, "port": port })) {
                Ok(_) => me.log(LogKind::Info, format!("已中斷 {ip}:{port}")),
                Err(e) => me.log(LogKind::Error, format!("中斷連線失敗：{e}")),
            }
        });
    }

    pub fn set_port(&self, new: u16) {
        let (status, old) = {
            let s = self.s();
            (s.status.clone(), s.port)
        };
        if new == old {
            return;
        }
        match status {
            Status::Running => {
                let me = self.clone();
                thread::spawn(move || match api::call(old, "set_config", &json!({ "values": { "port": new } })) {
                    Ok(_) => {
                        let gen = {
                            let mut s = me.s();
                            s.port = new;
                            s.gen += 1;
                            s.status = Status::Starting;
                            s.reset_live();
                            s.push_log(LogKind::Info, format!("埠號改為 {new}"));
                            s.gen
                        };
                        me.wait_ready(gen, new, Duration::from_secs(10));
                    }
                    Err(e) => me.log(LogKind::Error, format!("改埠號失敗：{e}")),
                });
            }
            Status::Stopped | Status::Failed(_) => match write_port(&self.hfs_dir.join("config.yaml"), new) {
                Ok(()) => {
                    self.s().port = new;
                    self.log(LogKind::Info, format!("埠號改為 {new}，下次啟動生效"));
                }
                Err(e) => self.log(LogKind::Error, format!("寫入 config.yaml 失敗：{e}")),
            },
            _ => self.log(LogKind::Error, "伺服器啟動或停止中，請稍後再改埠號"),
        }
    }
}

fn list_ops(v: &Value) -> impl Iterator<Item = (&str, Option<&Value>, Option<&Value>)> {
    v.as_array().into_iter().flatten().filter_map(|op| {
        let op = op.as_array()?;
        Some((op.first()?.as_str()?, op.get(1), op.get(2)))
    })
}

/// "+" 的參數可能是單筆物件或陣列
fn list_items(a: Option<&Value>) -> impl Iterator<Item = &Map<String, Value>> {
    let items: Vec<&Map<String, Value>> = match a {
        Some(Value::Array(arr)) => arr.iter().filter_map(Value::as_object).collect(),
        Some(Value::Object(m)) => vec![m],
        _ => vec![],
    };
    items.into_iter()
}

fn matches_all(c: &Map<String, Value>, q: &Map<String, Value>) -> bool {
    q.iter().all(|(k, v)| c.get(k) == Some(v))
}

fn read_port(config: &Path) -> Option<u16> {
    let text = std::fs::read_to_string(config).ok()?;
    text.lines().find_map(|l| l.strip_prefix("port:").and_then(|v| v.trim().parse().ok()))
}

fn write_port(config: &Path, port: u16) -> std::io::Result<()> {
    let text = std::fs::read_to_string(config).unwrap_or_default();
    let mut found = false;
    let mut lines: Vec<String> = text
        .lines()
        .map(|l| {
            if l.starts_with("port:") {
                found = true;
                format!("port: {port}")
            } else {
                l.to_string()
            }
        })
        .collect();
    if !found {
        lines.insert(0, format!("port: {port}"));
    }
    std::fs::write(config, lines.join("\n") + "\n")
}

pub fn now() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

pub fn fmt_bytes(n: f64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = n.max(0.0);
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{v:.0} {}", UNITS[i])
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}

pub fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    let hex = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push(h << 4 | l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// 同 JavaScript 的 encodeURIComponent
pub fn encode_uri_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &c in s.as_bytes() {
        if c.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&c) {
            out.push(c as char);
        } else {
            out += &format!("%{c:02X}");
        }
    }
    out
}

pub fn is_loopback(ip: &str) -> bool {
    matches!(ip, "127.0.0.1" | "::1" | "::ffff:127.0.0.1")
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_and_decode_round_trip() {
        let name = "HFS分享 (1)#?.txt";
        let enc = encode_uri_component(name);
        assert_eq!(enc, "HFS%E5%88%86%E4%BA%AB%20(1)%23%3F.txt");
        assert_eq!(percent_decode(&enc), name);
    }

    #[test]
    fn decode_leaves_bad_escapes() {
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz%4"), "%zz%4");
        assert_eq!(percent_decode("%中文"), "%中文");
    }

    #[test]
    fn port_rewrite() {
        let dir = std::env::temp_dir().join(format!("hfs-gui-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = dir.join("config.yaml");
        std::fs::write(&cfg, "vfs:\n  source: C:\\x\nhttps_port: 443\n").unwrap();
        assert_eq!(read_port(&cfg), None);
        write_port(&cfg, 9000).unwrap();
        assert_eq!(read_port(&cfg), Some(9000));
        write_port(&cfg, 8080).unwrap();
        let text = std::fs::read_to_string(&cfg).unwrap();
        assert_eq!(text.matches("port: ").count(), 1 + 1); // port 和 https_port
        assert_eq!(read_port(&cfg), Some(8080));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn list_protocol() {
        let v: Value = serde_json::from_str(
            r#"[["+",[{"ip":"1.2.3.4","port":5},{"ip":"1.2.3.4","port":6}]],["=",{"ip":"1.2.3.4","port":5},{"op":"download","path":null}],["-",{"ip":"1.2.3.4","port":6}],["ready"]]"#,
        )
        .unwrap();
        let mut conns: Vec<Map<String, Value>> = Vec::new();
        for (op, a, b) in list_ops(&v) {
            match op {
                "+" => conns.extend(list_items(a).cloned()),
                "-" => conns.retain(|c| !matches_all(c, a.unwrap().as_object().unwrap())),
                "=" => {
                    let c = conns.iter_mut().find(|c| matches_all(c, a.unwrap().as_object().unwrap())).unwrap();
                    for (k, val) in b.unwrap().as_object().unwrap() {
                        if val.is_null() {
                            c.remove(k);
                        } else {
                            c.insert(k.clone(), val.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        assert_eq!(conns.len(), 1);
        assert_eq!(conns[0]["op"], "download");
    }
}
