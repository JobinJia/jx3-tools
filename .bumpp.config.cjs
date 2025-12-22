module.exports = {
  files: [
    'package.json',
    'src-tauri/tauri.conf.json',
    'src-tauri/Cargo.toml',
  ],
  commit: 'chore(release): v%s',
  tag: 'v%s',
  push: false,
}
