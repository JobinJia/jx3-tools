// bumpp 配置。文件名必须是 `bump.config.*`（bumpp v11 用 unconfig 自动发现，
// 旧的 `.bumpp.config.cjs` + `--config` 标志均无效：正确的自定义路径标志是
// `--configFilePath`，但约定俗成直接用 `bump.config.cjs` 让其自动加载）。
//
// 关键：三个版本文件必须保持同步。bumpp 以 package.json 的版本为「当前版本」，
// 对 tauri.conf.json / Cargo.toml 做的是「全局文本替换当前版本串」，若三者不同步
// 会导致非 package.json 文件因匹配不到旧版本串而被跳过。
module.exports = {
  files: [
    'package.json',
    'src-tauri/tauri.conf.json',
    'src-tauri/Cargo.toml',
  ],
  commit: 'chore(release): v%s',
  tag: 'v%s',
  // 不自动 push，便于推送前核对构建产物（见 docs/RELEASE.md）
  push: false,
  // 连同 execute 改动的 Cargo.lock 一起提交（release 前需保证工作区干净）
  all: true,
  // 在写完版本文件、提交之前同步 Cargo.lock 里 jx3-tools 包自身的版本，
  // 只改该包块，绝不碰其它依赖（避免误伤同版本号的依赖）
  execute({ state }) {
    const fs = require('node:fs')
    const lockPath = 'src-tauri/Cargo.lock'
    const lock = fs.readFileSync(lockPath, 'utf8')
    const next = lock.replace(
      /(name = "jx3-tools"\nversion = ")[^"]+(")/,
      `$1${state.newVersion}$2`,
    )
    if (next !== lock)
      fs.writeFileSync(lockPath, next)
  },
}
