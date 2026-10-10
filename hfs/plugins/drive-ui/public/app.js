// Google 雲端硬碟風格的 HFS 分享網頁。純 JS、零建置。
(() => {
'use strict'

// ---------- 圖示（Material Icons 路徑，內嵌以便離線使用） ----------
const P = {
    menu: 'M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z',
    search: 'M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z',
    close: 'M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z',
    back: 'M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z',
    chevron: 'M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z',
    chevronLeft: 'M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z',
    folder: 'M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z',
    file: 'M6 2c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6H6zm7 7V3.5L18.5 9H13z',
    image: 'M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z',
    movie: 'M18 4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4h-4z',
    audio: 'M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z',
    doc: 'M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z',
    sheet: 'M20 2H4c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zM8 20H4v-4h4v4zm0-6H4v-4h4v4zm0-6H4V4h4v4zm6 12h-4v-4h4v4zm0-6h-4v-4h4v4zm0-6h-4V4h4v4zm6 12h-4v-4h4v4zm0-6h-4v-4h4v4zm0-6h-4V4h4v4z',
    slide: 'M10 8v8l5-4-5-4zm9-5H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14z',
    archive: 'M20.54 5.23l-1.39-1.68C18.88 3.21 18.47 3 18 3H6c-.47 0-.88.21-1.16.55L3.46 5.23C3.17 5.57 3 6.02 3 6.5V19c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V6.5c0-.48-.17-.93-.46-1.27zM12 17.5L6.5 12H10v-2h4v2h3.5L12 17.5zM5.12 5l.81-1h12l.94 1H5.12z',
    text: 'M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z',
    code: 'M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z',
    download: 'M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z',
    delete: 'M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z',
    edit: 'M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34a.9959.9959 0 0 0-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z',
    link: 'M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z',
    more: 'M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z',
    add: 'M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z',
    upload: 'M9 16h6v-6h4l-7-7-7 7h4zm-4 2h14v2H5z',
    newFolder: 'M20 6h-8l-2-2H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2zm-1 8h-3v3h-2v-3h-3v-2h3V9h2v3h3v2z',
    folderUpload: 'M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm0 12H4V8h16v10zM8 13.01l1.41 1.41L11 12.84V17h2v-4.16l1.59 1.59L16 13.01 12.01 9 8 13.01z',
    home: 'M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z',
    account: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z',
    logout: 'M17 7l-1.41 1.41L18.17 11H8v2h10.17l-2.58 2.58L17 17l5-5zM4 5h8V3H4c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h8v-2H4V5z',
    checkCircle: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z',
    error: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z',
    open: 'M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z',
    expandMore: 'M16.59 8.59L12 13.17 7.41 8.59 6 10l6 6 6-6z',
    expandLess: 'M12 8l-6 6 1.41 1.41L12 10.83l4.59 4.58L18 14z',
    list: 'M3 14h4v-4H3v4zm0 5h4v-4H3v4zM3 9h4V5H3v4zm5 5h13v-4H8v4zm0 5h13v-4H8v4zM8 5v4h13V5H8z',
    grid: 'M3 3v8h8V3H3zm6 6H5V5h4v4zm-6 4v8h8v-8H3zm6 6H5v-4h4v4zm4-16v8h8V3h-8zm6 6h-4V5h4v4zm-6 4v8h8v-8h-8zm6 6h-4v-4h4v4z',
    sort: 'M3 18h6v-2H3v2zM3 6v2h18V6H3zm0 7h12v-2H3v2z',
    eye: 'M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z',
    play: 'M8 5v14l11-7z',
    up: 'M4 12l1.41 1.41L11 7.83V20h2V7.83l5.58 5.59L20 12l-8-8-8 8z',
    down: 'M20 12l-1.41-1.41L13 16.17V4h-2v12.17l-5.58-5.59L4 12l8 8 8-8z',
    check: 'M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z',
    info: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z',
    move: 'M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm-6 12v-3h-4v-4h4V8l5 5-5 5z',
    copy: 'M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z',
    pause: 'M6 19h4V5H6v14zm8-14v14h4V5h-4z',
    refresh: 'M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z',
    selectAll: 'M3 5h2V3c-1.1 0-2 .9-2 2zm0 8h2v-2H3v2zm4 8h2v-2H7v2zM3 9h2V7H3v2zm10-6h-2v2h2V3zm6 0v2h2c0-1.1-.9-2-2-2zM5 21v-2H3c0 1.1.9 2 2 2zm-2-4h2v-2H3v2zM9 3H7v2h2V3zm2 18h2v-2h-2v2zm8-8h2v-2h-2v2zm0 8c1.1 0 2-.9 2-2h-2v2zm0-12h2V7h-2v2zm0 8h2v-2h-2v2zm-4 4h2v-2h-2v2zm0-16h2V3h-2v2zM7 17h10V7H7v10zm2-8h6v6H9V9z',
    comment: 'M21.99 4c0-1.1-.89-2-1.99-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h14l4 4-.01-18zM18 14H6v-2h12v2zm0-3H6V9h12v2zm0-3H6V6h12v2z',
    speed: 'M20.38 8.57l-1.23 1.85a8 8 0 0 1-.22 7.58H5.07A8 8 0 0 1 15.58 6.85l1.85-1.23A10 10 0 0 0 3.35 19a2 2 0 0 0 1.72 1h13.85a2 2 0 0 0 1.74-1 10 10 0 0 0-.27-10.44zm-9.79 6.84a2 2 0 0 0 2.83 0l5.66-8.49-8.49 5.66a2 2 0 0 0 0 2.83z',
}
const ic = (name, cls = '', color = '') =>
    `<svg class="ic ${cls}" viewBox="0 0 24 24" aria-hidden="true"${color ? ` style="color:${color}"` : ''}><path d="${P[name]}"/></svg>`

// ---------- 檔案類型 ----------
const TYPES = {
    image: { exts: 'jpg jpeg png gif webp bmp svg avif ico heic heif tif tiff', icon: 'image', color: '#d93025' },
    video: { exts: 'mp4 m4v mkv webm mov avi wmv flv ts mts m2ts mpg mpeg 3gp', icon: 'movie', color: '#d93025' },
    audio: { exts: 'mp3 wav flac aac ogg oga m4a wma opus mid midi', icon: 'audio', color: '#a142f4' },
    pdf: { exts: 'pdf', icon: 'doc', color: '#ea4335' },
    doc: { exts: 'doc docx odt rtf pages', icon: 'doc', color: '#4285f4' },
    sheet: { exts: 'xls xlsx xlsm ods csv numbers', icon: 'sheet', color: '#188038' },
    slide: { exts: 'ppt pptx odp key', icon: 'slide', color: '#f9ab00' },
    archive: { exts: 'zip rar 7z tar gz tgz bz2 xz iso cab', icon: 'archive', color: '#5f6368' },
    code: { exts: 'js mjs ts jsx tsx py rs go java kt c cc cpp h hpp cs php rb sh bat cmd ps1 html htm css scss sql lua swift vue', icon: 'code', color: '#1e8e3e' },
    text: { exts: 'txt md log ini cfg conf json xml yaml yml toml srt ass vtt nfo', icon: 'text', color: '#5f6368' },
}
const EXT_TYPE = {}
for (const [type, t] of Object.entries(TYPES))
    for (const e of t.exts.split(' '))
        EXT_TYPE[e] ??= type
const TEXT_EXTS = new Set([...TYPES.text.exts.split(' '), ...TYPES.code.exts.split(' '), 'csv'])
const BROWSER_IMAGES = new Set('jpg jpeg png gif webp bmp svg avif ico'.split(' '))
const BROWSER_VIDEO = new Set('mp4 m4v webm mov ogv'.split(' '))
const BROWSER_AUDIO = new Set('mp3 wav flac aac ogg oga m4a opus'.split(' '))

const FILTERS = [
    ['all', '全部'],
    ['folder', '資料夾'],
    ['image', '圖片'],
    ['video', '影片'],
    ['audio', '音訊'],
    ['doc', '文件'],
    ['archive', '壓縮檔'],
]
const DOC_TYPES = new Set(['pdf', 'doc', 'sheet', 'slide', 'text', 'code'])
const matchFilter = (e, f) => f === 'all' || (f === 'doc' ? DOC_TYPES.has(e.type) : e.type === f)

function typeIcon(e, cls = '') {
    if (e.isFolder) return ic('folder', cls, 'var(--text-2)')
    const t = TYPES[e.type]
    return t ? ic(t.icon, cls, t.color) : ic('file', cls, '#5f6368')
}

// ---------- 小工具 ----------
const $ = sel => document.querySelector(sel)
const esc = s => String(s ?? '').replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c])
const encPath = s => s.split('/').map(encodeURIComponent).join('/')
const safeDecode = s => { try { return decodeURIComponent(s) } catch { return s } }
const collator = new Intl.Collator('zh-Hant', { numeric: true, sensitivity: 'base' })
const coarse = matchMedia('(pointer: coarse)')

function store(key, value) {
    try {
        if (value === undefined) return localStorage.getItem('drive-ui:' + key)
        localStorage.setItem('drive-ui:' + key, value)
    } catch { return null }
}

function fmtSize(n) {
    if (n == null || n < 0) return '—'
    const u = ['位元組', 'KB', 'MB', 'GB', 'TB']
    let i = 0
    while (n >= 1024 && i < u.length - 1) { n /= 1024; i++ }
    return i ? `${n < 10 ? n.toFixed(1) : Math.round(n)} ${u[i]}` : `${Math.round(n)} ${u[0]}`
}

function fmtDate(v) {
    if (!v) return '—'
    const d = new Date(v), now = new Date()
    if (isNaN(d)) return '—'
    if (d.toDateString() === now.toDateString())
        return d.toLocaleTimeString('zh-TW', { hour: 'numeric', minute: '2-digit' })
    if (d.getFullYear() === now.getFullYear())
        return `${d.getMonth() + 1}月${d.getDate()}日`
    return `${d.getFullYear()}年${d.getMonth() + 1}月${d.getDate()}日`
}

async function api(cmd, params = {}) {
    const res = await fetch('/~/api/' + cmd, {
        method: 'POST',
        headers: { 'content-type': 'application/json', 'x-hfs-anti-csrf': '1' },
        body: JSON.stringify(params),
    })
    const text = await res.text()
    let data
    try { data = JSON.parse(text) } catch { data = text }
    if (!res.ok)
        throw Object.assign(new Error(typeof data === 'string' && data ? data : res.statusText), { status: res.status, data })
    return data
}

function errText(status) {
    return {
        401: '需要登入', 403: '沒有權限', 404: '找不到', 409: '已有同名項目',
        413: '檔案太大', 507: '伺服器空間不足', 0: '連線中斷',
    }[status] || `錯誤 ${status}`
}

// ---------- 狀態 ----------
const S = {
    path: location.pathname.endsWith('/') ? location.pathname : location.pathname.replace(/[^/]*$/, ''),
    list: [], props: {}, loading: false, error: null, loadedAt: 0,
    view: new URLSearchParams(location.search).get('view') || store('view') || (coarse.matches ? 'grid' : 'list'),
    sort: { key: store('sortKey') || 'name', dir: Number(store('sortDir')) || 1 },
    filter: 'all', q: '', deep: null, deepQ: '',
    sel: new Set(), anchor: null, visible: [],
    user: null, isAdmin: false, rootFolders: [],
    uploads: [], uploadsMin: false,
    viewer: null,
    details: store('details') === '1',
    folderSize: {},  // uri -> { bytes, files, folders } | 'loading'
    // 多通道下載：同時幾條連線（0 = 關閉，交給瀏覽器）
    dlThreads: Number(store('dlThreads') ?? 8) || 0,
    downloads: [],
}

function mkEntry(raw, base) {
    const isFolder = raw.n.endsWith('/')
    const full = isFolder ? raw.n.slice(0, -1) : raw.n
    const slash = full.lastIndexOf('/')
    const name = full.slice(slash + 1)
    const dot = name.lastIndexOf('.')
    const ext = !isFolder && dot > 0 ? name.slice(dot + 1).toLowerCase() : ''
    return {
        ...raw, key: raw.n, isFolder, name, ext,
        dir: slash < 0 ? '' : full.slice(0, slash),
        rel: full,
        uri: base + encPath(full) + (isFolder ? '/' : ''),
        type: isFolder ? 'folder' : EXT_TYPE[ext] || 'file',
        size: raw.s, mtime: raw.m || raw.c,
    }
}

const canDelete = e => !!(e.p?.includes('D') || (S.props.can_delete_children && !e.p?.includes('d')))
const canArchive = e => !!(e.p?.includes('A') || (S.props.can_archive && !e.p?.includes('a')))
const canDownload = e => !e.url && (e.isFolder ? canArchive(e) : true)
function cantOpen(e) {
    const x = e.isFolder && !e.web ? 'L' : 'R'
    return e.p?.includes(x) ? 'login' : e.p?.includes(x.toLowerCase()) ? 'forbidden' : ''
}
const selectedEntries = () => S.visible.filter(e => S.sel.has(e.key))

// ---------- 版面骨架 ----------
const app = $('#app')
app.innerHTML = `
<header class="top">
    <button class="icon-btn only-mobile" data-act="drawer" title="選單">${ic('menu')}</button>
    <a class="brand" href="/" data-nav="/"><img src="/~/plugins/drive-ui/icon.svg" alt=""><b>檔案分享</b></a>
    <div class="search">
        <button class="icon-btn" data-act="deepsearch" title="搜尋所有子資料夾">${ic('search')}</button>
        <input id="q" type="search" placeholder="搜尋這個資料夾" title="輸入即篩選；按 Enter 連同子資料夾一起搜尋" autocomplete="off" spellcheck="false">
        <button class="icon-btn" data-act="clearq" title="清除搜尋" hidden>${ic('close')}</button>
    </div>
    <div class="top-actions" id="acct"></div>
</header>
<aside class="side" id="side"></aside>
<section class="main" id="main">
    <div class="head"><div class="crumbs" id="crumbs"></div><div class="toolbar" id="toolbar"></div></div>
    <div class="progress" id="prog"></div>
    <div class="body">
        <div class="content" id="content" tabindex="-1"></div>
        <aside class="details" id="details" hidden></aside>
    </div>
</section>
<button class="fab" data-act="new" id="fab" title="新增" hidden>${ic('add')}</button>`

const el = {
    q: $('#q'), clearq: $('[data-act=clearq]'), acct: $('#acct'), side: $('#side'), main: $('#main'),
    crumbs: $('#crumbs'), toolbar: $('#toolbar'), prog: $('#prog'), content: $('#content'), fab: $('#fab'),
    details: $('#details'),
}

// ---------- 繪製 ----------
function render() {
    renderCrumbs()
    renderToolbar()
    renderContent()
    renderSide()
    renderDetails()
    el.prog.classList.toggle('on', S.loading)
    el.fab.hidden = !S.props.can_upload
    const segs = S.path.split('/').filter(Boolean)
    document.title = (segs.length ? safeDecode(segs.at(-1)) : '首頁') + ' – 檔案分享'
}

function renderAcct() {
    el.acct.innerHTML = `<button class="icon-btn" data-act="dlmenu" title="下載設定（多通道）">${ic('speed')}</button>` + (S.user
        ? `<button class="icon-btn" data-act="account" title="${esc(S.user)}"><span class="avatar">${esc(S.user[0].toUpperCase())}</span></button>`
        : `<button class="icon-btn" data-act="login" title="登入">${ic('account')}</button>`)
}

function renderSide() {
    const canUp = !!S.props.can_upload
    el.side.innerHTML = `
        ${canUp ? `<button class="new-btn" data-act="new">${ic('add')}新增</button>` : '<div style="height:8px"></div>'}
        <a class="nav-item ${S.path === '/' ? 'active' : ''}" href="/" data-nav="/">${ic('home', 'sm')}<span>首頁</span></a>
        ${S.rootFolders.length ? '<div class="nav-title">資料夾</div>' : ''}
        ${S.rootFolders.map(f => `
            <a class="nav-item ${S.path.startsWith(f.uri) ? 'active' : ''}" href="${esc(f.uri)}" data-nav="${esc(f.uri)}">
                ${ic('folder', 'sm')}<span>${esc(f.name)}</span>
            </a>`).join('')}
        <div class="side-foot">由 HFS 提供<br><a href="?classic">切換到傳統介面</a></div>`
}

function renderCrumbs() {
    const parts = [{ name: '首頁', uri: '/' }]
    let acc = '/'
    for (const s of S.path.split('/').filter(Boolean)) {
        acc += s + '/'
        parts.push({ name: safeDecode(s), uri: acc })
    }
    if (S.deep) parts.push({ name: `「${S.deepQ}」的搜尋結果` })
    el.crumbs.innerHTML = parts.map((p, i) => i === parts.length - 1
        ? `<span class="crumb last" title="${esc(p.name)}">${esc(p.name)}</span>`
        : `<button class="crumb" data-nav="${esc(p.uri)}" title="${esc(p.name)}">${esc(p.name)}</button>${ic('chevron', 'sm')}`
    ).join('')
}

function renderToolbar() {
    const sel = selectedEntries()
    if (sel.length) {
        const one = sel.length === 1 ? sel[0] : null
        const b = (act, icon, title) => `<button class="icon-btn" data-act="${act}" title="${title}">${ic(icon)}</button>`
        el.toolbar.innerHTML = `<div class="selbar">
            ${b('clearsel', 'close', '取消選取')}
            <span class="count">已選取 ${sel.length} 個項目</span>
            ${sel.every(canDownload) ? b('dlsel', 'download', '下載') : ''}
            ${one ? b('linksel', 'link', '複製連結') : ''}
            ${one && canDelete(one) ? b('renamesel', 'edit', '重新命名') : ''}
            ${sel.every(canDelete) ? b('movesel', 'move', '移動到…') : ''}
            ${sel.every(e => !e.isFolder) ? b('copysel', 'copy', '複製到…') : ''}
            ${sel.every(canDelete) ? b('deletesel', 'delete', '刪除') : ''}
            <span class="spacer"></span>
            ${b('details', 'info', '詳細資訊')}
            ${one ? b('moresel', 'more', '更多動作') : ''}
        </div>`
        return
    }
    const source = S.deep ?? S.list
    const present = new Set(source.map(e => DOC_TYPES.has(e.type) ? 'doc' : e.type))
    const chips = FILTERS.filter(([k]) => k === 'all' || present.has(k) || S.filter === k)
    const sortName = { name: '名稱', mtime: '修改日期', size: '檔案大小' }[S.sort.key]
    el.toolbar.innerHTML = `
        ${chips.length > 2 ? chips.map(([k, label]) => `<button class="chip ${S.filter === k ? 'on' : ''}" data-filter="${k}">${label}</button>`).join('') : ''}
        <span class="spacer"></span>
        ${S.props.can_archive && S.list.length ? `<button class="icon-btn" data-act="dlfolder" title="把這個資料夾打包成 ZIP 下載">${ic('download')}</button>` : ''}
        <button class="chip" data-act="sortmenu" title="排序方式">${ic(S.sort.dir > 0 ? 'up' : 'down', 'xs')}${sortName}</button>
        <div class="seg">
            <button class="${S.view === 'list' ? 'on' : ''}" data-view="list" title="清單檢視">${ic('list')}</button>
            <button class="${S.view === 'grid' ? 'on' : ''}" data-view="grid" title="格狀檢視">${ic('grid')}</button>
        </div>
        <button class="icon-btn ${S.details ? 'on' : ''}" data-act="details" title="詳細資訊">${ic('info')}</button>`
}

function computeVisible() {
    let arr = S.deep ?? S.list
    if (S.q && !S.deep) {
        const q = S.q.toLowerCase()
        arr = arr.filter(e => e.name.toLowerCase().includes(q))
    }
    arr = arr.filter(e => matchFilter(e, S.filter))
    const { key, dir } = S.sort
    const val = e => key === 'size' ? (e.size ?? -1) : key === 'mtime' ? +new Date(e.mtime || 0) : 0
    return arr.slice().sort((a, b) => {
        if (a.isFolder !== b.isFolder) return a.isFolder ? -1 : 1
        const d = key === 'name' ? collator.compare(a.name, b.name) : val(a) - val(b) || collator.compare(a.name, b.name)
        return d * dir
    })
}

const EMPTY_ART = `<svg class="art" viewBox="0 0 220 160" aria-hidden="true">
    <ellipse cx="110" cy="146" rx="84" ry="8" fill="var(--card)"/>
    <path d="M38 38a8 8 0 0 1 8-8h42l12 12h74a8 8 0 0 1 8 8v82a8 8 0 0 1-8 8H46a8 8 0 0 1-8-8z" fill="#a8c7fa"/>
    <path d="M38 62a8 8 0 0 1 8-8h128a8 8 0 0 1 8 8v70a8 8 0 0 1-8 8H46a8 8 0 0 1-8-8z" fill="#d3e3fd"/>
    <rect x="92" y="78" width="36" height="44" rx="4" fill="#fff"/>
    <path d="M100 92h20M100 100h20M100 108h12" stroke="#a8c7fa" stroke-width="4" stroke-linecap="round"/>
    <circle cx="172" cy="34" r="12" fill="#fbbc04"/>
    <circle cx="44" cy="20" r="6" fill="#34a853"/>
</svg>`

function emptyState() {
    const st = (title, text = '', btn = '') => `<div class="empty">${EMPTY_ART}<h2>${title}</h2>${text ? `<p>${text}</p>` : ''}${btn}</div>`
    if (S.error === 401)
        return st('需要登入才能查看', '這個資料夾需要帳號密碼。', `<button class="btn filled" data-act="login">${ic('account', 'sm')}登入</button>`)
    if (S.error === 403) return st('你沒有權限查看這個資料夾')
    if (S.error === 404) return st('找不到這個資料夾', '可能已經被移動或刪除。', '<button class="btn" data-nav="/">回到首頁</button>')
    if (S.error) return st('無法讀取資料夾', esc(errText(S.error)), '<button class="btn" data-act="reload">重試</button>')
    if (S.loading) return ''
    if (S.deep) return st(`找不到符合「${esc(S.deepQ)}」的項目`, '試試其他關鍵字。')
    if (S.q || S.filter !== 'all') return st('沒有符合條件的項目')
    return S.props.can_upload
        ? st('這個資料夾是空的', '把檔案拖曳到這裡，或使用「新增」按鈕上傳。', `<button class="btn filled" data-act="upfiles">${ic('upload', 'sm')}上傳檔案</button>`)
        : st('這個資料夾是空的')
}

function thumb(e) {
    if (e.type === 'image' && BROWSER_IMAGES.has(e.ext) && !cantOpen(e))
        return `<img src="${esc(e.uri)}" loading="lazy" decoding="async" alt="" draggable="false">`
    return e.type === 'video' ? `<span class="play">${ic('play')}</span>` : typeIcon(e, 'big')
}

function renderContent() {
    S.visible = computeVisible()
    const keep = new Set(S.visible.map(e => e.key))
    for (const k of [...S.sel]) if (!keep.has(k)) S.sel.delete(k)
    if (!S.visible.length) {
        el.content.innerHTML = emptyState()
        return
    }
    const deep = !!S.deep
    const isSel = e => S.sel.has(e.key) ? 'sel' : ''
    if (S.view === 'list') {
        const sortBtn = (key, label) => `<button data-sort="${key}">${label}${S.sort.key === key ? ic(S.sort.dir > 0 ? 'up' : 'down', 'xs') : ''}</button>`
        el.content.innerHTML = `<div class="list ${deep ? 'deep' : ''}">
            <div class="row header">
                <div class="hname"><input type="checkbox" class="selall" title="全選" ${S.sel.size && S.sel.size === S.visible.length ? 'checked' : ''}>${sortBtn('name', '名稱')}</div>
                ${deep ? '<div>位置</div>' : ''}
                <div>${sortBtn('mtime', '修改日期')}</div>
                <div>${sortBtn('size', '檔案大小')}</div>
                <div></div>
            </div>
            ${S.visible.map((e, i) => `
            <div class="row item ${isSel(e)}" data-i="${i}">
                <div class="name" draggable="true">${typeIcon(e)}<div class="nm"><span title="${esc(e.name)}">${esc(e.name)}${e.comment ? ` <i class="has-comment" title="${esc(e.comment)}">${ic('comment', 'xs')}</i>` : ''}</span><small>${fmtDate(e.mtime)}${e.isFolder ? '' : ' · ' + fmtSize(e.size)}</small></div></div>
                ${deep ? `<div class="meta" title="${esc(e.dir)}">${esc(e.dir || '（目前資料夾）')}</div>` : ''}
                <div class="meta">${fmtDate(e.mtime)}</div>
                <div class="meta">${e.isFolder ? '—' : fmtSize(e.size)}</div>
                <div class="acts">
                    ${canDownload(e) ? `<button class="icon-btn sm hide-m" data-act="dl" title="下載">${ic('download', 'sm')}</button>` : ''}
                    <button class="icon-btn sm hide-m" data-act="link" title="複製連結">${ic('link', 'sm')}</button>
                    <button class="icon-btn sm" data-act="more" title="更多動作">${ic('more', 'sm')}</button>
                </div>
            </div>`).join('')}
        </div>`
        return
    }
    const folders = [], files = []
    S.visible.forEach((e, i) => (e.isFolder ? folders : files).push([e, i]))
    el.content.innerHTML = `
        ${folders.length ? `<div class="sec-title">資料夾</div><div class="folders">${folders.map(([e, i]) => `
            <div class="fcard ${isSel(e)}" data-i="${i}" draggable="true" title="${esc(deep && e.dir ? e.dir + '/' + e.name : e.name)}">
                ${typeIcon(e)}<span>${esc(e.name)}</span>
                <button class="icon-btn sm" data-act="more" title="更多動作">${ic('more', 'sm')}</button>
            </div>`).join('')}</div>` : ''}
        ${files.length ? `<div class="sec-title">檔案</div><div class="files">${files.map(([e, i]) => `
            <div class="card ${isSel(e)}" data-i="${i}" draggable="true" title="${esc(deep && e.dir ? e.dir + '/' + e.name : e.name)}">
                <div class="ch">${typeIcon(e, 'sm')}<span>${esc(e.name)}</span>
                    <button class="icon-btn sm" data-act="more" title="更多動作">${ic('more', 'sm')}</button></div>
                <div class="thumb ${e.type === 'video' ? 'video' : ''}">${thumb(e)}</div>
            </div>`).join('')}</div>` : ''}`
}

function updateSel() {
    for (const node of el.content.querySelectorAll('[data-i]'))
        node.classList.toggle('sel', S.sel.has(S.visible[node.dataset.i]?.key))
    const all = el.content.querySelector('.selall')
    if (all) {
        all.checked = S.sel.size > 0 && S.sel.size === S.visible.length
        all.indeterminate = S.sel.size > 0 && S.sel.size < S.visible.length
    }
    renderToolbar()
    renderDetails()
}

// 縮圖載入失敗就換回圖示
el.content.addEventListener('error', ev => {
    const img = ev.target
    if (img.tagName !== 'IMG') return
    const card = img.closest('[data-i]')
    const e = card && S.visible[card.dataset.i]
    if (e) img.outerHTML = typeIcon(e, 'big')
}, true)

// ---------- 載入 ----------
async function load({ silent } = {}) {
    const path = S.path
    if (!silent) { S.loading = true; S.error = null; render() }
    try {
        const r = await api('get_file_list', { uri: path })
        if (path !== S.path) return
        S.props = r
        S.list = (r.list || []).map(x => mkEntry(x, path))
        S.error = null
    }
    catch (e) {
        if (path !== S.path) return
        if (!silent) { S.list = []; S.props = {} }
        S.error = e.status || 0
    }
    S.loading = false
    S.loadedAt = Date.now()
    render()
}

async function loadRootFolders() {
    try {
        const r = await api('get_file_list', { uri: '/', onlyFolders: true })
        S.rootFolders = (r.list || []).map(x => mkEntry(x, '/')).filter(e => e.isFolder)
            .sort((a, b) => collator.compare(a.name, b.name))
    }
    catch { S.rootFolders = [] }
    renderSide()
}

async function deepSearch(q) {
    q = q.trim()
    if (!q) return
    S.loading = true
    S.deepQ = q
    render()
    const path = S.path
    try {
        const r = await api('get_file_list', { uri: path, search: q, wild: 'no' })
        if (path !== S.path) return
        S.deep = (r.list || []).map(x => mkEntry(x, path))
        S.sel.clear()
    }
    catch (e) { toast('搜尋失敗：' + errText(e.status || 0)) }
    S.loading = false
    render()
}

function navigate(path, push = true) {
    closeMenu()
    document.body.classList.remove('drawer')
    S.sel.clear()
    S.anchor = null
    S.deep = null
    S.q = ''
    S.filter = 'all'
    el.q.value = ''
    el.clearq.hidden = true
    S.path = path
    if (push && location.pathname !== path) history.pushState(null, '', path)
    el.content.scrollTop = 0
    load()
}

let reloadTimer
function scheduleReload() {
    clearTimeout(reloadTimer)
    reloadTimer = setTimeout(() => { load({ silent: true }); if (S.path === '/') loadRootFolders() }, 400)
}

// ---------- 動作 ----------
function open(e) {
    const no = cantOpen(e)
    if (no === 'login') return loginDialog(() => open(e))
    if (no === 'forbidden') return toast('你沒有權限開啟這個項目')
    if (e.url) return window.open(e.url, '_blank', 'noopener')
    if (e.isFolder) return navigate(e.uri)
    openViewer(e)
}

function triggerDownload(href) {
    const a = document.createElement('a')
    a.href = href
    a.download = ''
    document.body.append(a)
    a.click()
    a.remove()
}

function download(items) {
    if (!items.length) return
    if (items.length === 1) {
        const e = items[0]
        if (!e.isFolder && S.dlThreads > 1 && (e.size ?? 0) >= MULTI_MIN_SIZE) return multiDownload(e)
        return triggerDownload(e.isFolder ? e.uri + '?get=zip' : e.uri + '?dl')
    }
    const list = items.map(e => encPath(e.rel)).join('//')
    triggerDownload(S.path + '?get=zip&list=' + encodeURIComponent(list))
}

async function copyText(text) {
    try {
        await navigator.clipboard.writeText(text)
        return true
    }
    catch { // 區網 http 不是安全環境，改用舊方法
        const ta = document.createElement('textarea')
        ta.value = text
        ta.style.cssText = 'position:fixed;opacity:0'
        document.body.append(ta)
        ta.select()
        const ok = document.execCommand('copy')
        ta.remove()
        return ok
    }
}

async function copyLink(e) {
    const url = location.origin + e.uri
    toast(await copyText(url) ? '已複製連結' : '無法複製，連結：' + url)
}

function renameDialog(e) {
    openDialog({
        title: '重新命名',
        fields: [{ label: '名稱', value: e.name, selectBase: !e.isFolder }],
        okLabel: '確定',
        async onOk([name]) {
            name = name.trim()
            if (!name || name === e.name) return
            if (/[\\/:*?"<>|]/.test(name)) throw new Error('名稱不能包含 \\ / : * ? " < > |')
            try { await api('rename', { uri: e.uri, dest: name }) }
            catch (err) { throw new Error('重新命名失敗：' + errText(err.status)) }
            toast(`已重新命名為「${name}」`)
            scheduleReload()
        },
    })
}

function deleteDialog(items) {
    const one = items.length === 1
    openDialog({
        title: one ? `要刪除「${items[0].name}」嗎？` : `要刪除 ${items.length} 個項目嗎？`,
        text: '檔案會從伺服器的電腦上永久刪除，無法復原。',
        okLabel: '刪除',
        danger: true,
        async onOk() {
            const results = await Promise.all(items.map(e =>
                fetch(e.uri, { method: 'DELETE', headers: { 'x-hfs-anti-csrf': '1' } })
                    .then(r => r.ok || r.status === 404 ? null : r.status, () => 0)))
            const failed = results.filter(x => x !== null)
            S.sel.clear()
            toast(failed.length ? `${items.length - failed.length} 個已刪除，${failed.length} 個失敗（${errText(failed[0])}）` : one ? '已刪除' : `已刪除 ${items.length} 個項目`)
            scheduleReload()
        },
    })
}

function newFolderDialog() {
    const path = S.path
    openDialog({
        title: '新資料夾',
        fields: [{ label: '資料夾名稱', value: '未命名資料夾', selectAll: true }],
        okLabel: '建立',
        async onOk([name]) {
            name = name.trim()
            if (!name) throw new Error('請輸入名稱')
            if (/[\\/:*?"<>|]/.test(name)) throw new Error('名稱不能包含 \\ / : * ? " < > |')
            try { await api('create_folder', { uri: path, name }) }
            catch (err) { throw new Error('建立失敗：' + errText(err.status)) }
            toast(`已建立「${name}」`)
            scheduleReload()
        },
    })
}

function loginDialog(after) {
    openDialog({
        title: '登入',
        fields: [
            { label: '帳號', autocomplete: 'username' },
            { label: '密碼', type: 'password', autocomplete: 'current-password' },
        ],
        okLabel: '登入',
        async onOk([username, password]) {
            if (!username) throw new Error('請輸入帳號')
            try {
                const r = await api('login', { username, password })
                S.user = r.username || username
                S.isAdmin = !!r.isAdmin
            }
            catch (err) { throw new Error(err.status === 401 ? '帳號或密碼錯誤' : '登入失敗：' + errText(err.status)) }
            renderAcct()
            toast(`已登入：${S.user}`)
            loadRootFolders()
            if (after) after()
            else load()
        },
    })
}

async function logout() {
    try { await api('logout') } catch {}
    S.user = null
    S.isAdmin = false
    renderAcct()
    toast('已登出')
    loadRootFolders()
    load()
}

// ---------- 選單 ----------
let menuEl = null
function closeMenu() { menuEl?.remove(); menuEl = null }

function showMenu(x, y, items) {
    closeMenu()
    // 去掉空項目，以及開頭、結尾、連續的分隔線
    items = items.filter(Boolean).filter((it, i, arr) => it !== '-' || (i > 0 && arr[i - 1] !== '-'))
    while (items[0] === '-') items.shift()
    while (items.at(-1) === '-') items.pop()
    if (!items.length) return
    menuEl = document.createElement('div')
    menuEl.className = 'menu'
    menuEl.innerHTML = items.map((it, i) => it === '-' ? '<hr>'
        : `<button data-mi="${i}" class="${it.danger ? 'danger' : ''}" ${it.disabled ? 'disabled' : ''}>${ic(it.icon, 'sm')}<span>${esc(it.label)}</span>${it.checked ? `<span style="margin-left:auto">${ic('check', 'sm')}</span>` : ''}</button>`).join('')
    document.body.append(menuEl)
    const r = menuEl.getBoundingClientRect()
    menuEl.style.left = Math.max(8, Math.min(x, innerWidth - r.width - 8)) + 'px'
    menuEl.style.top = Math.max(8, y + r.height > innerHeight - 8 ? y - r.height : y) + 'px'
    menuEl.addEventListener('click', ev => {
        const b = ev.target.closest('[data-mi]')
        if (!b) return
        const it = items[b.dataset.mi]
        closeMenu()
        it.run()
    })
}

function menuAt(anchor, items) {
    const r = anchor.getBoundingClientRect()
    showMenu(r.left, r.bottom + 4, items)
}

function itemMenu(e) {
    const sel = selectedEntries()
    if (sel.length > 1 && S.sel.has(e.key)) {
        return [
            sel.every(canDownload) && { icon: 'download', label: `下載 ${sel.length} 個項目（ZIP）`, run: () => download(sel) },
            '-',
            sel.every(canDelete) && { icon: 'move', label: '移動到…', run: () => moveCopyDialog(sel, false) },
            sel.every(x => !x.isFolder) && { icon: 'copy', label: '複製到…', run: () => moveCopyDialog(sel, true) },
            { icon: 'info', label: '詳細資訊', run: () => toggleDetails(true) },
            '-',
            sel.every(canDelete) && { icon: 'delete', label: `刪除 ${sel.length} 個項目`, danger: true, run: () => deleteDialog(sel) },
        ]
    }
    return [
        { icon: e.isFolder ? 'open' : 'eye', label: e.isFolder ? '開啟' : '預覽', run: () => open(e) },
        canDownload(e) && { icon: 'download', label: e.isFolder ? '下載（ZIP）' : '下載', run: () => download([e]) },
        { icon: 'link', label: '複製連結', run: () => copyLink(e) },
        S.deep && { icon: 'folder', label: '顯示所在位置', run: () => navigate(S.path + (e.dir ? encPath(e.dir) + '/' : '')) },
        '-',
        canDelete(e) && { icon: 'edit', label: '重新命名', run: () => renameDialog(e) },
        canDelete(e) && { icon: 'move', label: '移動到…', run: () => moveCopyDialog([e], false) },
        !e.isFolder && { icon: 'copy', label: '複製到…', run: () => moveCopyDialog([e], true) },
        { icon: 'info', label: '詳細資訊', run: () => toggleDetails(true) },
        '-',
        canDelete(e) && { icon: 'delete', label: '刪除', danger: true, run: () => deleteDialog([e]) },
    ]
}

function newMenu() {
    return [
        { icon: 'newFolder', label: '新資料夾', run: newFolderDialog },
        '-',
        { icon: 'upload', label: '上傳檔案', run: () => pickFiles(false) },
        { icon: 'folderUpload', label: '上傳資料夾', run: () => pickFiles(true) },
    ]
}

function sortMenu() {
    const pick = key => () => {
        S.sort = S.sort.key === key ? { key, dir: -S.sort.dir } : { key, dir: key === 'name' ? 1 : -1 }
        store('sortKey', S.sort.key)
        store('sortDir', S.sort.dir)
        renderToolbar()
        renderContent()
    }
    return [
        { icon: 'sort', label: '名稱', checked: S.sort.key === 'name', run: pick('name') },
        { icon: 'sort', label: '修改日期', checked: S.sort.key === 'mtime', run: pick('mtime') },
        { icon: 'sort', label: '檔案大小', checked: S.sort.key === 'size', run: pick('size') },
        '-',
        { icon: S.sort.dir > 0 ? 'down' : 'up', label: S.sort.dir > 0 ? '改為遞減' : '改為遞增', run: () => { S.sort.dir *= -1; store('sortDir', S.sort.dir); renderToolbar(); renderContent() } },
    ]
}

function accountMenu() {
    return [
        { icon: 'account', label: S.user, run() {} },
        S.isAdmin && { icon: 'open', label: '網頁管理介面', run: () => window.open('/~/admin/', '_blank') },
        '-',
        { icon: 'logout', label: '登出', run: logout },
    ]
}

// ---------- 對話框 / 通知 ----------
let dialogClose = null

function openDialog({ title, text, fields = [], okLabel = '確定', danger, onOk }) {
    const scrim = document.createElement('div')
    scrim.className = 'scrim'
    scrim.innerHTML = `<form class="dialog" novalidate>
        <h3>${esc(title)}</h3>
        ${text ? `<p>${esc(text)}</p>` : ''}
        ${fields.map((f, i) => `<div class="field">
            <input name="f${i}" id="f${i}" type="${f.type || 'text'}" value="${esc(f.value || '')}" autocomplete="${f.autocomplete || 'off'}" spellcheck="false">
            <label for="f${i}">${esc(f.label)}</label></div>`).join('')}
        <div class="err"></div>
        <div class="actions">
            <button type="button" class="btn" data-x>取消</button>
            <button type="submit" class="btn ${danger ? 'danger' : 'filled'}">${esc(okLabel)}</button>
        </div></form>`
    document.body.append(scrim)
    const form = scrim.firstElementChild
    const inputs = [...form.querySelectorAll('input')]
    const close = () => { scrim.remove(); if (dialogClose === close) dialogClose = null }
    dialogClose = close
    if (inputs[0]) {
        inputs[0].focus()
        const f = fields[0], v = inputs[0].value
        if (f.selectBase && v.lastIndexOf('.') > 0) inputs[0].setSelectionRange(0, v.lastIndexOf('.'))
        else if (f.selectBase || f.selectAll) inputs[0].select()
    } else form.querySelector('[type=submit]').focus()
    scrim.addEventListener('mousedown', ev => { if (ev.target === scrim) close() })
    form.querySelector('[data-x]').onclick = close
    form.onsubmit = async ev => {
        ev.preventDefault()
        const btn = form.querySelector('[type=submit]')
        btn.disabled = true
        try {
            await onOk(inputs.map(i => i.value))
            close()
        }
        catch (err) {
            form.querySelector('.err').textContent = err.message || String(err)
            btn.disabled = false
        }
    }
}

let toastTimer
function toast(msg, action) {
    $('.toast')?.remove()
    const t = document.createElement('div')
    t.className = 'toast'
    t.innerHTML = `<span>${esc(msg)}</span>${action ? `<button>${esc(action.label)}</button>` : ''}`
    if (action) t.querySelector('button').onclick = () => { t.remove(); action.run() }
    document.body.append(t)
    clearTimeout(toastTimer)
    toastTimer = setTimeout(() => t.remove(), 4000)
}

// ---------- 預覽 ----------
function previewKind(e) {
    if (e.type === 'image' && BROWSER_IMAGES.has(e.ext)) return 'image'
    if (BROWSER_VIDEO.has(e.ext)) return 'video'
    if (BROWSER_AUDIO.has(e.ext)) return 'audio'
    if (e.ext === 'pdf') return 'pdf'
    if (TEXT_EXTS.has(e.ext)) return 'text'
    return ''
}

function openViewer(e) {
    const files = S.visible.filter(x => !x.isFolder && !x.url)
    const node = document.createElement('div')
    node.className = 'viewer'
    document.body.append(node)
    S.viewer = { node, files, idx: Math.max(0, files.indexOf(e)) }
    renderViewer()
}

function closeViewer() {
    clearInterval(S.viewer?.timer)
    S.viewer?.node.remove()
    S.viewer = null
}

function toggleSlideshow() {
    const v = S.viewer
    if (!v) return
    if (v.timer) {
        clearInterval(v.timer)
        v.timer = null
    } else {
        v.timer = setInterval(() => {
            const n = v.files.length
            for (let k = 1; k <= n; k++) {
                const idx = (v.idx + k) % n
                if (previewKind(v.files[idx]) === 'image') { v.idx = idx; break }
            }
            renderViewer()
        }, 4000)
    }
    renderViewer()
}

function stepViewer(d) {
    const v = S.viewer
    if (!v) return
    const idx = v.idx + d
    if (idx < 0 || idx >= v.files.length) return
    v.idx = idx
    renderViewer()
}

function renderViewer() {
    const { node, files, idx } = S.viewer
    const e = files[idx]
    const src = esc(e.uri)
    const kind = previewKind(e)
    const body = {
        image: `<img src="${src}" alt="">`,
        video: `<video src="${src}" controls autoplay playsinline></video>`,
        audio: `<div class="noprev">${typeIcon(e, 'big')}<div>${esc(e.name)}</div><audio src="${src}" controls autoplay></audio></div>`,
        pdf: `<iframe src="${src}" title="${esc(e.name)}"></iframe>`,
        text: '<pre>載入中…</pre>',
    }[kind] ?? `<div class="noprev">${typeIcon(e, 'big')}<div>${esc(e.name)}</div><div style="color:#aaa">${fmtSize(e.size)} · 這個檔案無法預覽</div>
        <button class="btn filled" data-vact="dl">${ic('download', 'sm')}下載</button></div>`
    node.innerHTML = `
        <div class="vh">
            <button class="icon-btn" data-vact="close" title="返回 (Esc)">${ic('back')}</button>
            <div class="title">${typeIcon(e)}<span>${esc(e.name)}</span><small style="color:#aaa;flex:none">${idx + 1} / ${files.length}</small></div>
            ${files.filter(f => previewKind(f) === 'image').length > 1
                ? `<button class="icon-btn" data-vact="slideshow" title="${S.viewer.timer ? '停止投影片' : '投影片播放'}">${ic(S.viewer.timer ? 'pause' : 'play')}</button>` : ''}
            <button class="icon-btn" data-vact="dl" title="下載">${ic('download')}</button>
            <button class="icon-btn" data-vact="newtab" title="在新分頁開啟">${ic('open')}</button>
        </div>
        <div class="vb">
            ${idx > 0 ? `<button class="nav prev" data-vact="prev" title="上一個 (←)">${ic('chevronLeft')}</button>` : ''}
            ${body}
            ${idx < files.length - 1 ? `<button class="nav next" data-vact="next" title="下一個 (→)">${ic('chevron')}</button>` : ''}
        </div>`
    if (kind === 'text') {
        const LIMIT = 1024 * 1024
        fetch(e.uri, { headers: { Range: `bytes=0-${LIMIT - 1}` } })
            .then(r => r.ok ? r.text() : Promise.reject(r.status))
            .then(t => {
                const pre = node.querySelector('pre')
                if (pre && S.viewer?.files[S.viewer.idx] === e)
                    pre.textContent = t + (e.size > LIMIT ? '\n\n…（檔案太大，只顯示前 1 MB）' : '')
            })
            .catch(s => { const pre = node.querySelector('pre'); if (pre) pre.textContent = '無法載入：' + errText(s) })
    }
}

document.addEventListener('click', ev => {
    const v = S.viewer
    if (!v || !v.node.contains(ev.target)) return
    const act = ev.target.closest('[data-vact]')?.dataset.vact
    const e = v.files[v.idx]
    if (act === 'close') closeViewer()
    else if (act === 'prev') stepViewer(-1)
    else if (act === 'next') stepViewer(1)
    else if (act === 'dl') download([e])
    else if (act === 'newtab') window.open(e.uri, '_blank', 'noopener')
    else if (act === 'slideshow') toggleSlideshow()
    else if (ev.target.classList.contains('vb')) closeViewer()
})

// ---------- 多通道下載 ----------
// 大檔案切成多段，用多條 Range 連線同時下載；本機（安全環境）可直接寫進檔案，
// 區網 http 不是安全環境，只能先放記憶體再存檔。
const MULTI_MIN_SIZE = 16 * 1024 * 1024
const sleep = ms => new Promise(r => setTimeout(r, ms))
let dlSeq = 0

async function multiDownload(e) {
    const d = { id: ++dlSeq, e, total: e.size || 0, done: 0, status: 'starting', msg: '', threads: S.dlThreads,
        speed: 0, lastT: performance.now(), lastDone: 0, ctrls: new Set(), cancelled: false, toDisk: false }
    S.downloads.push(d)
    S.uploadsMin = false
    renderUploads()
    const drop = () => { S.downloads = S.downloads.filter(x => x !== d); renderUploads() }
    let writable = null
    try {
        // 先確認伺服器支援分段：要回 206 和 Content-Range
        const probe = await fetch(e.uri, { headers: { Range: 'bytes=0-0' }, cache: 'no-store' })
        const total = Number((probe.headers.get('Content-Range') || '').split('/')[1])
        await probe.arrayBuffer().catch(() => {})
        if (probe.status !== 206 || !total) {
            drop()
            return triggerDownload(e.uri + '?dl')
        }
        d.total = total
        if (window.showSaveFilePicker && window.isSecureContext) {
            try {
                const handle = await showSaveFilePicker({ suggestedName: e.name })
                writable = await handle.createWritable()
                d.toDisk = true
            }
            catch (err) {
                if (err.name === 'AbortError') return drop() // 使用者按了取消
            }
        }
        const chunk = Math.min(64 << 20, Math.max(4 << 20, Math.ceil(total / (d.threads * 3))))
        const ranges = []
        for (let s = 0; s < total; s += chunk) ranges.push([s, Math.min(total, s + chunk) - 1])
        const parts = new Array(ranges.length)
        let next = 0
        d.status = 'downloading'
        renderUploads()

        const worker = async () => {
            while (!d.cancelled && next < ranges.length) {
                const idx = next++
                const [a, b] = ranges[idx]
                for (let attempt = 1; ; attempt++) {
                    const ctrl = new AbortController()
                    d.ctrls.add(ctrl)
                    let got = 0
                    try {
                        const res = await fetch(e.uri, { headers: { Range: `bytes=${a}-${b}` }, signal: ctrl.signal, cache: 'no-store' })
                        if (res.status !== 206) throw new Error(errText(res.status))
                        const reader = res.body.getReader()
                        const bufs = []
                        for (;;) {
                            const { done, value } = await reader.read()
                            if (done) break
                            bufs.push(value)
                            got += value.length
                            d.done += value.length
                            dlTick(d)
                        }
                        if (got !== b - a + 1) throw new Error('片段不完整')
                        const blob = new Blob(bufs)
                        if (writable) await writable.write({ type: 'write', position: a, data: blob })
                        else parts[idx] = blob
                        break
                    }
                    catch (err) {
                        d.done -= got
                        if (d.cancelled) return
                        if (attempt >= 4) throw err
                        await sleep(600 * attempt) // 退避後重試這一段
                    }
                    finally { d.ctrls.delete(ctrl) }
                }
            }
        }
        await Promise.all(Array.from({ length: Math.min(d.threads, ranges.length) }, worker))
        if (d.cancelled) {
            await writable?.abort?.().catch(() => {})
            return
        }
        if (writable) await writable.close()
        else {
            const url = URL.createObjectURL(new Blob(parts, { type: 'application/octet-stream' }))
            const a = document.createElement('a')
            a.href = url
            a.download = e.name
            document.body.append(a)
            a.click()
            a.remove()
            setTimeout(() => URL.revokeObjectURL(url), 120_000)
        }
        d.status = 'done'
        d.done = d.total
    }
    catch (err) {
        d.status = 'error'
        d.msg = err.message || String(err)
        for (const c of d.ctrls) c.abort()
        await writable?.abort?.().catch(() => {})
    }
    renderUploads()
}

function dlTick(d) {
    const now = performance.now()
    if (now - d.lastT >= 700) {
        const inst = (d.done - d.lastDone) / ((now - d.lastT) / 1000)
        d.speed = d.speed ? d.speed * 0.5 + inst * 0.5 : inst
        d.lastT = now
        d.lastDone = d.done
    }
    renderUploadsSoon()
}

function cancelDownload(d) {
    d.cancelled = true
    d.status = 'error'
    d.msg = '已取消'
    for (const c of d.ctrls) c.abort()
    renderUploads()
}

function dlMenu() {
    const pick = n => () => {
        S.dlThreads = n
        store('dlThreads', String(n))
        toast(n ? `多通道下載：${n} 條連線（16 MB 以上的檔案）` : '已關閉多通道下載，改由瀏覽器直接下載')
    }
    return [
        { icon: 'speed', label: '多通道下載（大檔案更快）', disabled: true, run() {} },
        '-',
        { icon: 'download', label: '關閉（瀏覽器直接下載）', checked: S.dlThreads === 0, run: pick(0) },
        { icon: 'speed', label: '4 條連線', checked: S.dlThreads === 4, run: pick(4) },
        { icon: 'speed', label: '8 條連線（建議）', checked: S.dlThreads === 8, run: pick(8) },
        { icon: 'speed', label: '16 條連線', checked: S.dlThreads === 16, run: pick(16) },
    ]
}

// ---------- 上傳 ----------
let uploadsEl = null
let uploadSeq = 0
const MAX_PARALLEL = 2

function pickFiles(folder) {
    const inp = document.createElement('input')
    inp.type = 'file'
    inp.multiple = true
    if (folder) inp.webkitdirectory = true
    inp.onchange = () => enqueue([...inp.files].map(f => ({ file: f, rel: f.webkitRelativePath || f.name })), S.path)
    inp.click()
}

async function enqueue(items, dest) {
    if (!items.length) return
    let mode = ''
    if (dest === S.path) { // 只能檢查目前看得到的檔名（第一層）
        const existing = new Set(S.list.filter(e => !e.isFolder).map(e => e.name.toLowerCase()))
        const conflicts = items.filter(it => !it.rel.includes('/') && existing.has(it.rel.toLowerCase()))
        if (conflicts.length) {
            const choices = [{ label: '略過', value: 'skip' }, { label: '保留兩者', value: 'rename', primary: !S.props.can_overwrite }]
            if (S.props.can_overwrite) choices.push({ label: '取代', value: 'overwrite', primary: true })
            mode = await choiceDialog(
                conflicts.length === 1 ? '檔案已經存在' : `有 ${conflicts.length} 個檔案已經存在`,
                conflicts.length === 1 ? `「${conflicts[0].rel}」已經在這個資料夾裡了，要怎麼處理？` : '這些檔名在資料夾裡已經有了，要怎麼處理？',
                choices)
            if (!mode) return
            if (mode === 'skip') {
                const skip = new Set(conflicts)
                items = items.filter(it => !skip.has(it))
                if (!items.length) return toast('已略過所有同名檔案')
            }
        }
    }
    for (const { file, rel } of items)
        S.uploads.push({ id: ++uploadSeq, file, rel, dest, loaded: 0, status: 'queued', msg: '', existing: mode === 'overwrite' ? 'overwrite' : '' })
    S.uploadsMin = false
    renderUploads()
    pump()
}

function pump() {
    while (S.uploads.filter(u => u.status === 'uploading').length < MAX_PARALLEL) {
        const u = S.uploads.find(u => u.status === 'queued')
        if (!u) break
        startUpload(u)
    }
}

function startUpload(u) {
    u.status = 'uploading'
    const xhr = new XMLHttpRequest()
    u.xhr = xhr
    xhr.open('PUT', u.dest + encPath(u.rel) + '?mtime=' + u.file.lastModified + (u.existing ? '&existing=' + u.existing : ''))
    xhr.upload.onprogress = ev => { u.loaded = ev.loaded; renderUploadsSoon() }
    xhr.onload = () => {
        const ok = xhr.status >= 200 && xhr.status < 300
        u.status = ok ? 'done' : 'error'
        u.msg = ok ? '' : errText(xhr.status)
        if (ok && u.dest === S.path) scheduleReload()
        finish()
    }
    xhr.onerror = () => { u.status = 'error'; u.msg = errText(0); finish() }
    xhr.onabort = () => { u.status = 'error'; u.msg = '已取消'; finish() }
    xhr.send(u.file)
    renderUploads()

    function finish() {
        u.xhr = null
        renderUploads()
        pump()
        if (!S.uploads.some(x => x.status === 'queued' || x.status === 'uploading')) {
            const bad = S.uploads.filter(x => x.status === 'error').length
            if (bad) toast(`有 ${bad} 個檔案上傳失敗`)
        }
    }
}

let uploadsRaf = 0
function renderUploadsSoon() {
    if (!uploadsRaf) uploadsRaf = requestAnimationFrame(() => { uploadsRaf = 0; renderUploads() })
}

function ring(p) {
    const c = 2 * Math.PI * 9
    return `<svg class="ring" viewBox="0 0 24 24"><circle class="bgc" cx="12" cy="12" r="9"/><circle class="fgc" cx="12" cy="12" r="9" stroke-dasharray="${c}" stroke-dashoffset="${c * (1 - p)}"/></svg>`
}

function renderUploads() {
    if (!S.uploads.length && !S.downloads.length) { uploadsEl?.remove(); uploadsEl = null; return }
    if (!uploadsEl) {
        uploadsEl = document.createElement('div')
        uploadsEl.className = 'uploads'
        document.body.append(uploadsEl)
        uploadsEl.addEventListener('click', ev => {
            const act = ev.target.closest('[data-uact]')?.dataset.uact
            if (act === 'min') { S.uploadsMin = !S.uploadsMin; renderUploads() }
            if (act === 'close') {
                for (const u of S.uploads) u.xhr?.abort()
                for (const d of S.downloads) if (d.status === 'starting' || d.status === 'downloading') cancelDownload(d)
                S.uploads = []
                S.downloads = []
                renderUploads()
            }
            const cancel = ev.target.closest('[data-dcancel]')
            if (cancel) {
                const d = S.downloads.find(x => x.id === Number(cancel.dataset.dcancel))
                if (d) cancelDownload(d)
            }
        })
    }
    const activeUp = S.uploads.filter(u => u.status === 'queued' || u.status === 'uploading').length
    const activeDown = S.downloads.filter(d => d.status === 'starting' || d.status === 'downloading').length
    const active = activeUp + activeDown
    const done = S.uploads.filter(u => u.status === 'done').length + S.downloads.filter(d => d.status === 'done').length
    const title = active
        ? [activeDown && `正在下載 ${activeDown} 個`, activeUp && `正在上傳 ${activeUp} 個`].filter(Boolean).join('、')
        : `已完成 ${done} 個傳輸`
    uploadsEl.classList.toggle('min', S.uploadsMin)
    const downRows = S.downloads.slice().reverse().map(d => {
        const running = d.status === 'starting' || d.status === 'downloading'
        const status = d.status === 'done' ? ic('checkCircle', '', '#188038')
            : d.status === 'error' ? ic('error', '', 'var(--danger)')
            : ring(d.total ? d.done / d.total : 0)
        const sub = d.status === 'error' ? `<small class="bad">${esc(d.msg)}</small>`
            : d.status === 'starting' ? '<small>準備中…</small>'
            : d.status === 'downloading' ? `<small>${fmtSize(d.done)} / ${fmtSize(d.total)} · ${fmtSize(d.speed)}/s · ${d.threads} 條連線${d.toDisk ? '' : ''}</small>`
            : `<small>已下載 ${fmtSize(d.total)}${d.toDisk ? '' : '，請到瀏覽器的下載清單查看'}</small>`
        return `<div class="urow">${ic('download', '', 'var(--primary)')}<div class="un"><div title="${esc(d.e.name)}">${esc(d.e.name)}</div>${sub}</div>
            ${running ? `<button class="icon-btn sm" data-dcancel="${d.id}" title="取消">${ic('close', 'sm')}</button>` : ''}${status}</div>`
    }).join('')
    uploadsEl.innerHTML = `
        <div class="uh"><b>${title}</b>
            <button class="icon-btn" data-uact="min" title="${S.uploadsMin ? '展開' : '收合'}">${ic(S.uploadsMin ? 'expandLess' : 'expandMore')}</button>
            <button class="icon-btn" data-uact="close" title="${active ? '全部取消' : '關閉'}">${ic('close')}</button>
        </div>
        <div class="ul">${downRows}${S.uploads.slice().reverse().map(u => {
            const e = mkEntry({ n: u.rel }, '')
            const status = u.status === 'done' ? ic('checkCircle', '', '#188038')
                : u.status === 'error' ? ic('error', '', 'var(--danger)')
                : ring(u.file.size ? u.loaded / u.file.size : 0)
            const sub = u.status === 'error' ? `<small class="bad">${esc(u.msg)}</small>`
                : u.status === 'uploading' ? `<small>${fmtSize(u.loaded)} / ${fmtSize(u.file.size)}</small>`
                : u.status === 'queued' ? '<small>等待中</small>' : ''
            return `<div class="urow">${typeIcon(e)}<div class="un"><div title="${esc(u.rel)}">${esc(e.name)}</div>${sub}</div>${status}</div>`
        }).join('')}</div>`
}

// 拖放上傳
let dragDepth = 0
let dropPill = null
const hasFiles = ev => [...(ev.dataTransfer?.types || [])].includes('Files')

function showDrop(on) {
    const allowed = !!S.props.can_upload
    el.main.classList.toggle('dropping', on && allowed)
    if (on && !dropPill) {
        const segs = S.path.split('/').filter(Boolean)
        const where = segs.length ? safeDecode(segs.at(-1)) : '首頁'
        dropPill = document.createElement('div')
        dropPill.className = 'drop-pill' + (allowed ? '' : ' denied')
        dropPill.innerHTML = allowed
            ? `${ic('upload')}<span>將檔案拖放到這裡，即可上傳到「${esc(where)}」</span>`
            : `${ic('error')}<span>「${esc(where)}」沒有開放上傳${S.user ? '' : '，可以試試登入'}</span>`
        document.body.append(dropPill)
    }
    if (!on) { dropPill?.remove(); dropPill = null }
}

addEventListener('dragenter', ev => {
    if (!hasFiles(ev)) return
    ev.preventDefault()
    if (++dragDepth === 1) showDrop(true)
})
addEventListener('dragover', ev => {
    if (!hasFiles(ev)) return
    ev.preventDefault()
    // 放到某個資料夾上 = 上傳到那個資料夾；停久一點會自動打開
    const t = dropTargetFrom(ev)
    clearDropTargets()
    springTo(t?.uri ?? null)
    if (t) t.node.classList.add('drop-target')
    ev.dataTransfer.dropEffect = t || S.props.can_upload ? 'copy' : 'none'
})
addEventListener('dragleave', ev => {
    if (!hasFiles(ev)) return
    if (--dragDepth <= 0) { dragDepth = 0; showDrop(false); clearDropTargets(); springTo(null) }
})
addEventListener('drop', async ev => {
    if (!hasFiles(ev)) return
    ev.preventDefault()
    const t = dropTargetFrom(ev)
    dragDepth = 0
    showDrop(false)
    clearDropTargets()
    springTo(null)
    if (!t && !S.props.can_upload) return toast('你沒有上傳到這個資料夾的權限')
    const dest = t ? t.uri : S.path
    const items = await collectDropped(ev.dataTransfer)
    if (t) toast(`上傳到「${safeDecode(dest.split('/').filter(Boolean).at(-1) || '首頁')}」`)
    enqueue(items, dest)
})

async function collectDropped(dt) {
    // 一定要在 await 之前把 entry 取出來，事件結束後 dataTransfer 就失效了
    const entries = [...dt.items].map(i => i.webkitGetAsEntry?.()).filter(Boolean)
    if (!entries.length) return [...dt.files].map(f => ({ file: f, rel: f.name }))
    const out = []
    async function walk(entry, prefix) {
        if (entry.isFile) {
            const f = await new Promise((res, rej) => entry.file(res, rej))
            out.push({ file: f, rel: prefix + f.name })
        }
        else if (entry.isDirectory) {
            const reader = entry.createReader()
            let batch
            do {
                batch = await new Promise((res, rej) => reader.readEntries(res, rej))
                for (const child of batch) await walk(child, prefix + entry.name + '/')
            } while (batch.length)
        }
    }
    for (const e of entries) await walk(e, '')
    return out
}

// ---------- 詳細資訊面板 ----------
const TYPE_NAMES = { folder: '資料夾', image: '圖片', video: '影片', audio: '音訊', pdf: 'PDF 文件', doc: '文件', sheet: '試算表', slide: '簡報', archive: '壓縮檔', code: '程式碼', text: '文字檔' }
const typeName = e => TYPE_NAMES[e.type] || (e.ext ? e.ext.toUpperCase() + ' 檔案' : '檔案')
const fullDate = v => v ? new Date(v).toLocaleString('zh-TW', { dateStyle: 'long', timeStyle: 'short' }) : '—'
let detailsEntry = null

function currentFolderEntry() {
    const segs = S.path.split('/').filter(Boolean)
    return { isFolder: true, isCurrent: true, key: '', type: 'folder', uri: S.path, rel: '', dir: '',
        name: segs.length ? safeDecode(segs.at(-1)) : '首頁', comment: S.props.comment }
}

function toggleDetails(force) {
    S.details = force ?? !S.details
    store('details', S.details ? '1' : '0')
    renderToolbar()
    renderDetails()
}

function renderDetails() {
    el.details.hidden = !S.details
    el.main.classList.toggle('with-details', S.details)
    if (!S.details) return
    const head = (icon, name) => `<div class="d-head">${icon}<b title="${esc(name)}">${esc(name)}</b><button class="icon-btn" data-act="details" title="關閉">${ic('close')}</button></div>`
    const sel = selectedEntries()
    if (sel.length > 1) {
        const bytes = sel.reduce((a, e) => a + (e.isFolder ? 0 : e.size || 0), 0)
        detailsEntry = null
        el.details.innerHTML = head(ic('selectAll'), `已選取 ${sel.length} 個項目`)
            + `<div class="d-preview">${ic('selectAll', 'big', 'var(--text-3)')}</div>`
            + `<dl><dt>檔案</dt><dd>${sel.filter(e => !e.isFolder).length} 個，共 ${fmtSize(bytes)}</dd><dt>資料夾</dt><dd>${sel.filter(e => e.isFolder).length} 個</dd></dl>`
        return
    }
    const e = sel[0] || currentFolderEntry()
    detailsEntry = e
    const fs = S.folderSize[e.uri]
    const size = !e.isFolder ? fmtSize(e.size)
        : fs === 'loading' ? '計算中…'
        : fs ? `${fmtSize(fs.bytes)}（${fs.files} 個檔案、${fs.folders} 個資料夾）`
        : '<button class="link" data-act="foldersize">計算大小</button>'
    const where = e.isCurrent ? safeDecode(S.path.replace(/[^/]*\/$/, '')) || '/'
        : safeDecode(S.path) + (e.dir ? e.dir + '/' : '')
    const preview = e.type === 'image' && BROWSER_IMAGES.has(e.ext) && !cantOpen(e)
        ? `<img src="${esc(e.uri)}" alt="">` : typeIcon(e, 'big')
    const canComment = !!S.props.can_comment
    el.details.innerHTML = head(typeIcon(e), e.name) + `
        <div class="d-preview">${preview}</div>
        <h4>檔案詳細資料</h4>
        <dl>
            <dt>類型</dt><dd>${typeName(e)}</dd>
            <dt>大小</dt><dd>${size}</dd>
            <dt>位置</dt><dd>${esc(where)}</dd>
            ${e.mtime ? `<dt>修改日期</dt><dd>${fullDate(e.mtime)}</dd>` : ''}
            ${e.c ? `<dt>建立日期</dt><dd>${fullDate(e.c)}</dd>` : ''}
        </dl>
        <h4>說明</h4>
        ${canComment
            ? `<textarea class="d-comment" placeholder="加上說明，讓其他人知道這是什麼">${esc(e.comment || '')}</textarea>
               <button class="btn filled sm" data-act="savecomment">儲存說明</button>`
            : `<p class="muted">${esc(e.comment || '沒有說明')}</p>`}
        ${e.isCurrent ? '' : `<div class="d-actions">
            ${canDownload(e) ? `<button class="btn" data-act="dl1">${ic('download', 'sm')}下載</button>` : ''}
            <button class="btn" data-act="link1">${ic('link', 'sm')}複製連結</button></div>`}`
}

async function computeFolderSize(e) {
    S.folderSize[e.uri] = 'loading'
    renderDetails()
    try {
        S.folderSize[e.uri] = await api('get_folder_size', { uri: e.uri, id: Math.random().toString(36).slice(2) })
    }
    catch (err) {
        delete S.folderSize[e.uri]
        toast('無法計算大小：' + errText(err.status))
    }
    renderDetails()
}

async function saveComment(e) {
    const text = el.details.querySelector('.d-comment')?.value ?? ''
    try {
        await api('comment', { uri: e.uri, comment: text })
        e.comment = text
        toast('已儲存說明')
        scheduleReload()
    }
    catch (err) { toast('儲存說明失敗：' + errText(err.status)) }
}

// ---------- 移動 / 複製 ----------
function choiceDialog(title, text, choices) {
    return new Promise(resolve => {
        const scrim = document.createElement('div')
        scrim.className = 'scrim'
        scrim.innerHTML = `<div class="dialog"><h3>${esc(title)}</h3><p>${esc(text)}</p><div class="actions">
            <button class="btn" data-v="">取消</button>
            ${choices.map(c => `<button class="btn ${c.primary ? 'filled' : ''}" data-v="${c.value}">${esc(c.label)}</button>`).join('')}
        </div></div>`
        const finish = v => { scrim.remove(); if (dialogClose === cancel) dialogClose = null; resolve(v) }
        const cancel = () => finish(null)
        dialogClose = cancel
        scrim.addEventListener('click', ev => { const b = ev.target.closest('[data-v]'); if (b) finish(b.dataset.v || null) })
        scrim.addEventListener('mousedown', ev => { if (ev.target === scrim) cancel() })
        document.body.append(scrim)
        scrim.querySelector('.btn.filled')?.focus()
    })
}

function folderPicker({ title, okLabel, items, onPick }) {
    const scrim = document.createElement('div')
    scrim.className = 'scrim'
    scrim.innerHTML = `<div class="dialog picker"><h3>${esc(title)}</h3>
        <div class="p-crumbs"></div><div class="p-list"></div><div class="err"></div>
        <div class="actions"><button class="btn" data-x>取消</button><button class="btn filled" data-ok>${esc(okLabel)}</button></div></div>`
    document.body.append(scrim)
    const box = scrim.firstElementChild
    const close = () => { scrim.remove(); if (dialogClose === close) dialogClose = null }
    dialogClose = close
    const blocked = items.filter(e => e.isFolder).map(e => e.uri)
    const parents = new Set(items.map(e => e.uri.replace(/[^/]+\/?$/, '')))
    let path = S.path
    async function go(p) {
        path = p
        const parts = [{ name: '首頁', uri: '/' }]
        let acc = '/'
        for (const s of p.split('/').filter(Boolean)) { acc += s + '/'; parts.push({ name: safeDecode(s), uri: acc }) }
        box.querySelector('.p-crumbs').innerHTML = parts.map((x, i) => i === parts.length - 1
            ? `<b>${esc(x.name)}</b>` : `<button class="link" data-p="${esc(x.uri)}">${esc(x.name)}</button>${ic('chevron', 'xs')}`).join('')
        const list = box.querySelector('.p-list')
        const ok = box.querySelector('[data-ok]')
        const err = box.querySelector('.err')
        list.innerHTML = '<div class="p-note">載入中…</div>'
        ok.disabled = true
        let canUp = false
        try {
            const r = await api('get_file_list', { uri: p, onlyFolders: true })
            if (path !== p) return
            canUp = !!r.can_upload
            const folders = (r.list || []).map(x => mkEntry(x, p)).filter(e => e.isFolder).sort((a, b) => collator.compare(a.name, b.name))
            list.innerHTML = folders.length ? folders.map(f => `<button class="p-item" data-p="${esc(f.uri)}" ${blocked.includes(f.uri) ? 'disabled' : ''}>
                ${ic('folder', 'sm', 'var(--text-2)')}<span>${esc(f.name)}</span>${ic('chevron', 'sm')}</button>`).join('')
                : '<div class="p-note">沒有子資料夾</div>'
        }
        catch (e) { list.innerHTML = `<div class="p-note">${esc(errText(e.status))}</div>` }
        const same = parents.size === 1 && parents.has(p)
        const inside = blocked.some(b => p.startsWith(b))
        ok.disabled = !canUp || same || inside
        err.textContent = !canUp ? '這個資料夾不能放入檔案（沒有上傳權限）' : same ? '項目已經在這個資料夾裡' : inside ? '不能把資料夾放進它自己裡面' : ''
    }
    box.addEventListener('click', ev => {
        const nav = ev.target.closest('[data-p]')
        if (nav && !nav.disabled) return go(nav.dataset.p)
        if (ev.target.closest('[data-x]')) return close()
        const ok = ev.target.closest('[data-ok]')
        if (ok && !ok.disabled) { close(); onPick(path) }
    })
    scrim.addEventListener('mousedown', ev => { if (ev.target === scrim) close() })
    go(path)
}

async function transfer(items, dest, copy) {
    const verb = copy ? '複製' : '移動'
    if (!copy && !items.every(canDelete)) return toast('你沒有移動這些項目的權限')
    try {
        const r = await api(copy ? 'copy_files' : 'move_files', { uri_from: items.map(e => e.uri), uri_to: dest })
        const failed = (r.errors || []).filter(Boolean)
        S.sel.clear()
        toast(failed.length ? `${items.length - failed.length} 個已${verb}，${failed.length} 個失敗（${errText(failed[0])}）`
            : `已${verb} ${items.length} 個項目`, dest !== S.path ? { label: '前往', run: () => navigate(dest) } : null)
        scheduleReload()
        if (dest === '/' || S.path === '/') loadRootFolders()
    }
    catch (e) { toast(`${verb}失敗：${errText(e.status)}`) }
}

function moveCopyDialog(items, copy) {
    const what = items.length === 1 ? `「${items[0].name}」` : `${items.length} 個項目`
    folderPicker({
        title: `${copy ? '複製' : '移動'}${what}到…`,
        okLabel: copy ? '複製到這裡' : '移到這裡',
        items,
        onPick: dest => transfer(items, dest, copy),
    })
}

// ---------- 拖曳：拖到資料夾＝移動；拖到桌面＝下載 ----------
let dragItems = null
const clearDropTargets = () => document.querySelectorAll('.drop-target').forEach(n => n.classList.remove('drop-target'))

el.content.addEventListener('dragstart', ev => {
    const item = itemFromEvent(ev)
    if (!item) return
    if (!S.sel.has(item.key)) { S.sel = new Set([item.key]); S.anchor = S.visible.indexOf(item); updateSel() }
    dragItems = selectedEntries()
    const dt = ev.dataTransfer
    dt.effectAllowed = 'copyMove'
    dt.setData('application/x-hfs-items', String(dragItems.length))
    dt.setData('text/uri-list', dragItems.map(e => location.origin + e.uri).join('\r\n'))
    if (dragItems.length === 1 && canDownload(item)) {
        const url = location.origin + (item.isFolder ? item.uri + '?get=zip' : item.uri + '?dl')
        dt.setData('DownloadURL', `application/octet-stream:${(item.isFolder ? item.name + '.zip' : item.name).replace(/:/g, '_')}:${url}`)
    }
    // 自訂拖曳圖示：圖示 + 名稱或「N 個項目」
    const ghost = document.createElement('div')
    ghost.className = 'drag-ghost'
    ghost.innerHTML = `${typeIcon(item, 'sm')}<span>${dragItems.length > 1 ? `${dragItems.length} 個項目` : esc(item.name)}</span>`
        + (dragItems.length > 1 ? `<b>${dragItems.length}</b>` : '')
    document.body.append(ghost)
    dt.setDragImage(ghost, 18, 18)
    setTimeout(() => ghost.remove())
})
el.content.addEventListener('dragend', () => { dragItems = null; clearDropTargets(); springTo(null) })

/** 拖曳時游標下的資料夾（清單/格狀的資料夾、麵包屑、側邊欄） */
function dropTargetFrom(ev) {
    const card = ev.target.closest('[data-i]')
    if (card && el.content.contains(card)) {
        const e = S.visible[card.dataset.i]
        return e?.isFolder && !dragItems?.includes(e) ? { node: card, uri: e.uri } : null
    }
    const nav = ev.target.closest('[data-nav]')
    if (nav && nav.dataset.nav !== S.path) return { node: nav, uri: nav.dataset.nav }
    return null
}

// 拖著停在資料夾上 0.8 秒就自動打開它（彈簧資料夾）
let spring = { uri: null, timer: 0 }
function springTo(uri) {
    if (spring.uri === uri) return
    clearTimeout(spring.timer)
    spring = { uri, timer: uri ? setTimeout(() => { spring.uri = null; navigate(uri) }, 800) : 0 }
}

document.addEventListener('dragover', ev => {
    if (!dragItems) return
    const t = dropTargetFrom(ev)
    clearDropTargets()
    springTo(t?.uri ?? null)
    if (!t) return
    ev.preventDefault()
    ev.dataTransfer.dropEffect = 'move'
    t.node.classList.add('drop-target')
})
document.addEventListener('drop', ev => {
    if (!dragItems) return
    const t = dropTargetFrom(ev)
    clearDropTargets()
    springTo(null)
    if (!t) return
    ev.preventDefault()
    const items = dragItems
    dragItems = null
    transfer(items, t.uri, false)
})

// ---------- 框選 ----------
// 在空白處（清單模式也可以從日期、大小欄位）按住拖曳，畫出方框選取；Ctrl / Shift 為加選
let marquee = null
let suppressClick = false

el.content.addEventListener('mousedown', ev => {
    if (ev.button !== 0 || coarse.matches) return
    const t = ev.target
    if (t.closest('button, input, textarea, a, .hname, .row.header')) return
    const item = t.closest('[data-i]')
    if (item && (S.view === 'grid' || t.closest('.name'))) return // 這些地方是拖曳搬移的把手
    ev.preventDefault()
    const r = el.content.getBoundingClientRect()
    const sx = el.content.scrollLeft - r.left, sy = el.content.scrollTop - r.top
    marquee = {
        x0: ev.clientX + sx, y0: ev.clientY + sy, x: ev.clientX, y: ev.clientY,
        base: ev.ctrlKey || ev.metaKey || ev.shiftKey ? new Set(S.sel) : new Set(),
        // 項目位置換算成捲動座標，捲動時不必重算
        items: [...el.content.querySelectorAll('[data-i]')].map(n => {
            const b = n.getBoundingClientRect()
            return { key: S.visible[n.dataset.i]?.key, l: b.left + sx, t: b.top + sy, r: b.right + sx, b: b.bottom + sy }
        }),
        moved: false, box: null, raf: 0, scroll: 0,
    }
})

function updateMarquee() {
    const m = marquee
    if (!m) return
    const r = el.content.getBoundingClientRect()
    const x = Math.max(r.left, Math.min(r.right, m.x)) - r.left + el.content.scrollLeft
    const y = Math.max(r.top, Math.min(r.bottom, m.y)) - r.top + el.content.scrollTop
    if (!m.moved && Math.hypot(x - m.x0, y - m.y0) < 5) return
    if (!m.moved) {
        m.moved = true
        m.box = document.createElement('div')
        m.box.className = 'marquee'
        el.content.append(m.box)
    }
    const l = Math.min(x, m.x0), t = Math.min(y, m.y0), w = Math.abs(x - m.x0), h = Math.abs(y - m.y0)
    Object.assign(m.box.style, { left: l + 'px', top: t + 'px', width: w + 'px', height: h + 'px' })
    const sel = new Set(m.base)
    for (const it of m.items)
        if (it.key && it.l < l + w && it.r > l && it.t < t + h && it.b > t) sel.add(it.key)
    const changed = sel.size !== S.sel.size || [...sel].some(k => !S.sel.has(k))
    if (changed) {
        S.sel = sel
        if (!m.raf) m.raf = requestAnimationFrame(() => { m.raf = 0; updateSel() })
    }
}

function autoScroll() {
    const m = marquee
    if (!m) return
    const r = el.content.getBoundingClientRect()
    const edge = 40
    const dy = m.y < r.top + edge ? -(r.top + edge - m.y) : m.y > r.bottom - edge ? m.y - (r.bottom - edge) : 0
    if (dy) {
        el.content.scrollTop += Math.sign(dy) * Math.min(30, Math.abs(dy) / 2 + 4)
        updateMarquee()
    }
    m.scroll = requestAnimationFrame(autoScroll)
}

document.addEventListener('mousemove', ev => {
    if (!marquee) return
    marquee.x = ev.clientX
    marquee.y = ev.clientY
    updateMarquee()
    if (marquee?.moved && !marquee.scroll) marquee.scroll = requestAnimationFrame(autoScroll)
})

document.addEventListener('mouseup', () => {
    const m = marquee
    if (!m) return
    marquee = null
    cancelAnimationFrame(m.scroll)
    m.box?.remove()
    if (m.moved) {
        suppressClick = true // 放開時瀏覽器會補一個 click，別讓它把選取清掉
        setTimeout(() => { suppressClick = false }, 0)
        S.anchor = S.visible.findIndex(e => S.sel.has(e.key))
        updateSel()
    }
})

// ---------- 鍵盤移動選取 ----------
function moveSelection(key, extend) {
    if (!S.visible.length) return
    const cur = S.anchor ?? -1
    let step = key === 'ArrowDown' || key === 'ArrowRight' ? 1 : -1
    if (S.view === 'grid' && (key === 'ArrowDown' || key === 'ArrowUp') && cur >= 0) {
        const node = el.content.querySelector(`[data-i="${cur}"]`)
        step *= node ? getComputedStyle(node.parentElement).gridTemplateColumns.split(' ').length : 1
    }
    const next = Math.max(0, Math.min(S.visible.length - 1, cur < 0 ? 0 : cur + step))
    if (extend && cur >= 0) S.sel.add(S.visible[next].key)
    else S.sel = new Set([S.visible[next].key])
    S.anchor = next
    updateSel()
    el.content.querySelector(`[data-i="${next}"]`)?.scrollIntoView({ block: 'nearest' })
}

function blankMenu() {
    const up = !!S.props.can_upload
    return [
        up && { icon: 'newFolder', label: '新資料夾', run: newFolderDialog },
        up && '-',
        up && { icon: 'upload', label: '上傳檔案', run: () => pickFiles(false) },
        up && { icon: 'folderUpload', label: '上傳資料夾', run: () => pickFiles(true) },
        '-',
        S.props.can_archive && S.list.length && { icon: 'download', label: '下載這個資料夾（ZIP）', run: () => triggerDownload(S.path + '?get=zip') },
        S.visible.length && { icon: 'selectAll', label: '全選', run: () => { S.sel = new Set(S.visible.map(e => e.key)); updateSel() } },
        { icon: 'refresh', label: '重新整理', run: () => load() },
        { icon: 'info', label: '詳細資訊', run: () => toggleDetails(true) },
        !up && '-',
        !up && { icon: 'upload', label: '這個資料夾沒有開放上傳', disabled: true, run() {} },
    ]
}

// ---------- 事件 ----------
function itemFromEvent(ev) {
    const node = ev.target.closest('[data-i]')
    return node && el.content.contains(node) ? S.visible[node.dataset.i] : null
}

function selectClick(e, ev) {
    const i = S.visible.indexOf(e)
    if (ev.shiftKey && S.anchor != null) {
        const [a, b] = [Math.min(S.anchor, i), Math.max(S.anchor, i)]
        if (!(ev.ctrlKey || ev.metaKey)) S.sel.clear()
        for (let k = a; k <= b; k++) S.sel.add(S.visible[k].key)
    }
    else if (ev.ctrlKey || ev.metaKey) {
        S.sel.has(e.key) ? S.sel.delete(e.key) : S.sel.add(e.key)
        S.anchor = i
    }
    else {
        S.sel = new Set([e.key])
        S.anchor = i
    }
    updateSel()
}

document.addEventListener('click', ev => {
    if (suppressClick) { suppressClick = false; return }
    if (S.viewer?.node.contains(ev.target)) return
    if (menuEl && !menuEl.contains(ev.target)) closeMenu()
    if (document.body.classList.contains('drawer') && !el.side.contains(ev.target) && !ev.target.closest('[data-act=drawer]'))
        document.body.classList.remove('drawer')

    if (ev.target.classList?.contains('selall')) {
        S.sel = ev.target.checked ? new Set(S.visible.map(e => e.key)) : new Set()
        return updateSel()
    }
    if (el.details.contains(ev.target) && !ev.target.closest('[data-act]')) return
    const nav = ev.target.closest('[data-nav]')
    if (nav) {
        if (ev.ctrlKey || ev.metaKey || ev.button) return // 讓瀏覽器開新分頁
        ev.preventDefault()
        return navigate(nav.dataset.nav)
    }
    const filter = ev.target.closest('[data-filter]')
    if (filter) {
        S.filter = filter.dataset.filter
        S.sel.clear()
        renderToolbar()
        renderContent()
        return
    }
    const view = ev.target.closest('[data-view]')
    if (view) {
        S.view = view.dataset.view
        store('view', S.view)
        renderToolbar()
        renderContent()
        return
    }
    const sort = ev.target.closest('[data-sort]')
    if (sort) {
        const key = sort.dataset.sort
        S.sort = S.sort.key === key ? { key, dir: -S.sort.dir } : { key, dir: key === 'name' ? 1 : -1 }
        store('sortKey', S.sort.key)
        store('sortDir', S.sort.dir)
        renderToolbar()
        renderContent()
        return
    }

    const actBtn = ev.target.closest('[data-act]')
    const item = itemFromEvent(ev)
    if (actBtn) {
        const act = actBtn.dataset.act
        const sel = selectedEntries()
        switch (act) {
            case 'drawer': document.body.classList.toggle('drawer'); return
            case 'new': return menuAt(actBtn, newMenu())
            case 'upfiles': return pickFiles(false)
            case 'login': return loginDialog()
            case 'account': return menuAt(actBtn, accountMenu())
            case 'dlmenu': return menuAt(actBtn, dlMenu())
            case 'reload': return load()
            case 'clearq': el.q.value = ''; S.q = ''; S.deep = null; el.clearq.hidden = true; render(); return
            case 'deepsearch': return deepSearch(el.q.value)
            case 'sortmenu': return menuAt(actBtn, sortMenu())
            case 'clearsel': S.sel.clear(); updateSel(); return
            case 'dlsel': return download(sel)
            case 'linksel': return sel[0] && copyLink(sel[0])
            case 'renamesel': return sel[0] && renameDialog(sel[0])
            case 'deletesel': return deleteDialog(sel)
            case 'moresel': return sel[0] && menuAt(actBtn, itemMenu(sel[0]))
            case 'movesel': return moveCopyDialog(sel, false)
            case 'copysel': return moveCopyDialog(sel, true)
            case 'details': return toggleDetails()
            case 'dlfolder': return triggerDownload(S.path + '?get=zip')
            case 'foldersize': return detailsEntry && computeFolderSize(detailsEntry)
            case 'savecomment': return detailsEntry && saveComment(detailsEntry)
            case 'dl1': return detailsEntry && download([detailsEntry])
            case 'link1': return detailsEntry && copyLink(detailsEntry)
        }
        if (item) {
            if (!S.sel.has(item.key)) { S.sel = new Set([item.key]); S.anchor = S.visible.indexOf(item); updateSel() }
            if (act === 'dl') return download([item])
            if (act === 'link') return copyLink(item)
            if (act === 'more') return menuAt(actBtn, itemMenu(item))
        }
        return
    }
    if (item) {
        if (coarse.matches && !ev.ctrlKey && !ev.shiftKey && !S.sel.size) return open(item)
        return selectClick(item, ev)
    }
    if (el.content.contains(ev.target) && S.sel.size) {
        S.sel.clear()
        updateSel()
    }
})

el.content.addEventListener('dblclick', ev => {
    const item = itemFromEvent(ev)
    if (item && !ev.target.closest('[data-act]')) open(item)
})

el.content.addEventListener('contextmenu', ev => {
    const item = itemFromEvent(ev)
    if (item) {
        ev.preventDefault()
        if (!S.sel.has(item.key)) { S.sel = new Set([item.key]); S.anchor = S.visible.indexOf(item); updateSel() }
        return showMenu(ev.clientX, ev.clientY, itemMenu(item))
    }
    ev.preventDefault()
    if (S.sel.size) { S.sel.clear(); updateSel() }
    showMenu(ev.clientX, ev.clientY, blankMenu())
})

el.q.addEventListener('input', () => {
    S.q = el.q.value
    S.deep = null
    el.clearq.hidden = !S.q
    renderCrumbs()
    renderToolbar()
    renderContent()
})
el.q.addEventListener('keydown', ev => {
    if (ev.key === 'Enter') deepSearch(el.q.value)
    if (ev.key === 'Escape') { el.q.value = ''; el.q.dispatchEvent(new Event('input')); el.q.blur() }
})

document.addEventListener('keydown', ev => {
    if (ev.key === 'Escape') {
        if (menuEl) return closeMenu()
        if (dialogClose) return dialogClose()
        if (S.viewer) return closeViewer()
        if (S.sel.size) { S.sel.clear(); updateSel() }
        return
    }
    if (S.viewer) {
        if (ev.key === 'ArrowLeft') stepViewer(-1)
        if (ev.key === 'ArrowRight') stepViewer(1)
        return
    }
    if (dialogClose || /^(INPUT|TEXTAREA|SELECT)$/.test(document.activeElement?.tagName)) return
    const sel = selectedEntries()
    if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === 'a') {
        ev.preventDefault()
        S.sel = new Set(S.visible.map(e => e.key))
        updateSel()
    }
    else if (ev.key === 'Delete' && sel.length && sel.every(canDelete)) deleteDialog(sel)
    else if (ev.key === 'Enter' && sel.length === 1) open(sel[0])
    else if (ev.key === 'F2' && sel.length === 1 && canDelete(sel[0])) { ev.preventDefault(); renameDialog(sel[0]) }
    else if (ev.key === 'Backspace' && S.path !== '/') navigate(S.path.replace(/[^/]*\/$/, ''))
    else if (/^Arrow(Up|Down|Left|Right)$/.test(ev.key)) { ev.preventDefault(); moveSelection(ev.key, ev.shiftKey) }
})

addEventListener('popstate', () => {
    closeViewer()
    navigate(location.pathname.endsWith('/') ? location.pathname : location.pathname.replace(/[^/]*$/, ''), false)
})
addEventListener('scroll', closeMenu, true)
addEventListener('resize', closeMenu)
addEventListener('focus', () => { if (Date.now() - S.loadedAt > 15000 && !S.loading) load({ silent: true }) })

// ---------- 啟動 ----------
renderAcct()
render()
api('refresh_session').then(r => {
    S.user = r.username || null
    S.isAdmin = !!r.isAdmin
    renderAcct()
}).catch(() => {})
loadRootFolders()
load()
// HFS 遇到需要登入的檔案網址也會送這個頁面
if (!location.pathname.endsWith('/'))
    setTimeout(() => loginDialog(() => location.reload()), 300)
})()
