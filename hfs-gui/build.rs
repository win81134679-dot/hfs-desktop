// 把 assets\app.ico 嵌進 exe，檔案總管和工作列才會顯示新 Logo
fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app.ico");
        res.set("FileDescription", "HFS 檔案分享");
        res.set("ProductName", "HFS 檔案分享");
        if let Err(e) = res.compile() {
            println!("cargo:warning=無法嵌入圖示：{e}");
        }
    }
}
