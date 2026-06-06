# README.md の macOS ソースビルド要件に ninja を追加する

- Priority: Low
- Created: 2026-06-06
- Model: DeepSeek V4 Pro
- Branch: feature/fix-readme-macos-build-deps
- Polished: 2026-06-06
- Completed: 2026-06-07

## 目的

README.md の macOS 向けソースビルド手順に `ninja` を明示的に追加し、Linux 側の手順との対称性を保つ。

## 優先度根拠

- meson の Homebrew formula が依存として ninja を自動インストールするため実際には問題ないが、明示性の観点で改善。
- 影響範囲は README 1 行のみ。

## 現状

`README.md:52`:
```bash
brew install meson nasm
```

Linux 側 (`README.md:49`):
```bash
sudo apt-get install -y build-essential meson ninja-build nasm xxd
```

## 解決方法

`README.md:52` を以下に変更する:
```bash
brew install meson ninja nasm
```
