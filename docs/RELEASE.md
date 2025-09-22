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

- 默认不会自动 `push`，以便在推送前先核对构建产物。
- 如果需要同时构建再打包，可以在 `pnpm release` 后手动执行 `pnpm tauri build`。
- 如需在 CI 中使用，可在命令后追加 `--push --pushScript "git push --follow-tags"` 等参数。
