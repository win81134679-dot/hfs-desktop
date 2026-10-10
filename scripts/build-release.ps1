# 建置可公開發佈的版本：
#   1. 用 --remap-path-prefix 把本機路徑（使用者名稱、專案資料夾）從 exe 裡改寫掉
#   2. 建置後掃描 exe，確認沒有殘留本機路徑才繼續
#   3. 打包 dist\hfs-desktop-v<版本>-windows-x64.zip（zip 內路徑用 /）
# 用法：powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$gui = Join-Path $root 'hfs-gui'
$cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
$rustupHome = if ($env:RUSTUP_HOME) { $env:RUSTUP_HOME } else { Join-Path $env:USERPROFILE '.rustup' }

# 路徑在執行時才算出來，不寫死在任何會上傳的檔案裡。
# 路徑可能有空格，所以用 CARGO_ENCODED_RUSTFLAGS（以 0x1F 分隔），不用 RUSTFLAGS
Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
$env:CARGO_ENCODED_RUSTFLAGS = @(
    "--remap-path-prefix=$cargoHome=/cargo",
    "--remap-path-prefix=$rustupHome=/rustup",
    "--remap-path-prefix=$root=/src"
) -join [char]0x1f
# 讓 rustflags 只作用在這次建置，避免和平常的 debug 建置共用快取
$env:CARGO_TARGET_DIR = Join-Path $gui 'target\publish'

Push-Location $gui
try {
    cargo test --release
    if ($LASTEXITCODE) { throw '單元測試失敗' }
    cargo build --release
    if ($LASTEXITCODE) { throw '建置失敗' }
} finally { Pop-Location }

$exe = Join-Path $env:CARGO_TARGET_DIR 'release\hfs-gui.exe'

# 掃描：本機路徑（UTF-8 與 UTF-16）都不能出現
$latin1 = [Text.Encoding]::GetEncoding(28591) # 位元組和字元一對一，方便用字串搜尋
$hay = $latin1.GetString([IO.File]::ReadAllBytes($exe))
$needles = @($env:USERPROFILE, $root, $cargoHome) | Select-Object -Unique
$leaks = @()
foreach ($n in $needles) {
    foreach ($enc in [Text.Encoding]::UTF8, [Text.Encoding]::Unicode) {
        $needle = $latin1.GetString($enc.GetBytes($n))
        if ($hay.IndexOf($needle, [StringComparison]::Ordinal) -ge 0) { $leaks += "$($enc.WebName): $n" }
    }
}
if ($leaks) { throw "exe 裡還有本機路徑，停止發佈：`n$($leaks -join "`n")" }
Write-Host "掃描通過：exe 裡沒有本機路徑" -ForegroundColor Green

# 打包
$version = (Select-String -Path (Join-Path $gui 'Cargo.toml') -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
$dist = Join-Path $root 'dist'
$stage = Join-Path $dist 'stage'
Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force (Join-Path $stage 'plugins\drive-ui\public') | Out-Null
Copy-Item $exe (Join-Path $stage 'hfs-gui.exe')
foreach ($f in 'plugin.js', 'public\index.html', 'public\app.js', 'public\app.css', 'public\icon.svg') {
    Copy-Item (Join-Path $root "hfs\plugins\drive-ui\$f") (Join-Path $stage "plugins\drive-ui\$f")
}
Copy-Item (Join-Path $root 'hfs\config.example.yaml') $stage
Copy-Item (Join-Path $root 'LICENSE') $stage
Copy-Item (Join-Path $root 'scripts\README.txt') $stage

$zip = Join-Path $dist "hfs-desktop-v$version-windows-x64.zip"
Remove-Item $zip -Force -ErrorAction SilentlyContinue
Add-Type -AssemblyName System.IO.Compression, System.IO.Compression.FileSystem
$fs = [IO.File]::Open($zip, 'Create')
$archive = New-Object IO.Compression.ZipArchive($fs, [IO.Compression.ZipArchiveMode]::Create)
Get-ChildItem $stage -Recurse -File | ForEach-Object {
    $name = $_.FullName.Substring($stage.Length + 1).Replace('\', '/')
    [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $_.FullName, $name, [IO.Compression.CompressionLevel]::Optimal) | Out-Null
}
$archive.Dispose(); $fs.Dispose()
# 單一 exe 版本（自動下載 HFS）也另外放一份
Copy-Item $exe (Join-Path $dist "HFS-Desktop-v$version.exe") -Force

# 安裝程式（建桌面與開始功能表捷徑「HFS 檔案分享」）；沒裝 Inno Setup 6 就略過
$iscc = @("$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe", "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe") |
    Where-Object { Test-Path $_ } | Select-Object -First 1
if ($iscc) {
    & $iscc /Q "/DAppVersion=$version" "/DExeFile=$exe" (Join-Path $root 'scripts\installer.iss')
    if ($LASTEXITCODE) { throw "安裝程式編譯失敗（ISCC exit $LASTEXITCODE）" }
} else {
    Write-Host "找不到 Inno Setup 6，略過安裝程式" -ForegroundColor Yellow
}

Write-Host "完成：" -ForegroundColor Green
Get-ChildItem $dist -File | ForEach-Object { "  {0}  {1:N1} MB  SHA256 {2}" -f $_.Name, ($_.Length / 1MB), (Get-FileHash $_.FullName -Algorithm SHA256).Hash }
