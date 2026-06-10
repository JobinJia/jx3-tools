# 发布流程

项目使用 [`bumpp`](https://github.com/antfu/bumpp) 统一维护版本号，并同步更新
`package.json` 与 `src-tauri/tauri.conf.json`。打包时 Tauri 将直接读取
`tauri.conf.json` 中的版本，因此产出的安装包、执行文件都会携带正确的版本号。

## 步骤

1. 确认工作区干净、已经完成代码合并。
2. 运行需要的验证流程（`pnpm lint`、`pnpm test:unit`、`pnpm tauri build` 等）。
3. 根据需要的语义版本号执行：

   ```bash
   pnpm release patch   # 或者 minor / major
   ```

   该命令会：
   - 提示选择版本（或直接使用传入的参数）
   - 自动修改 `package.json` 与 `src-tauri/tauri.conf.json`
   - 生成提交 `chore(release): vX.Y.Z`
   - 打上 `vX.Y.Z` 的 Git tag

4. 检查提交与标签后，执行 `git push --follow-tags` 推送到远端。
5. 运行 `pnpm tauri build` 生成带版本号的安装包/压缩包，将其附加到 Release 页面。

## 注意

- 配置在 `bump.config.cjs`（bumpp v11 用 unconfig 自动发现此文件名，**不要**改名或回退到
  `.bumpp.config.cjs` + `--config`，那个标志名是错的、不会生效）。配置已设 `push: false`、
  `all: true`，并用 `execute` 钩子在提交前同步 `src-tauri/Cargo.lock` 里 jx3-tools 的版本。
- 因为 `all: true` 会把工作区内所有已跟踪改动一并提交，**运行 `pnpm release` 前务必确保工作区干净**。
- 三个版本文件（`package.json` / `src-tauri/tauri.conf.json` / `src-tauri/Cargo.toml`）必须保持
  同步：bumpp 以 package.json 版本为基准对其余文件做文本替换，不同步会导致替换匹配不到而被跳过。
- 默认不会自动 `push`，以便在推送前先核对；确认后执行 `git push --follow-tags`。
- 正式安装包由 `release.yml` 工作流构建：`gh workflow run release.yml -f version=vX.Y.Z`（标签需已推送）。
