// Google 雲端硬碟風格的分享網頁：HFS 要送前端時，改送我們自己的頁面。
// 權限、登入、檔案傳輸仍由 HFS 處理；網址加上 ?classic 可回到原本的 HFS 介面。

exports.description = "Google 雲端硬碟風格的分享網頁（網址加 ?classic 可回到原本介面）"
exports.version = 1
exports.apiRequired = 8

const fs = require('fs')
const path = require('path')

exports.init = () => {
    const pub = path.join(__dirname, 'public')

    function assetVersion() {
        return ['app.js', 'app.css']
            .map(f => Math.round(fs.statSync(path.join(pub, f)).mtimeMs).toString(36))
            .join('')
    }

    return {
        middleware(ctx) {
            if ('classic' in ctx.query || ctx.path.startsWith('/~/'))
                return
            return () => { // upstream：HFS 已經決定好回應之後
                if (!ctx.state.serveApp || ctx.method !== 'GET')
                    return
                try {
                    const html = fs.readFileSync(path.join(pub, 'index.html'), 'utf8')
                    ctx.type = 'html'
                    ctx.set('Cache-Control', 'no-cache')
                    ctx.body = html.replace(/\{\{v\}\}/g, assetVersion())
                }
                catch (e) {
                    console.error('drive-ui:', e)
                }
            }
        },
    }
}
