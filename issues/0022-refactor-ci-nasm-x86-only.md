# CI / Release の nasm インストールを x86_64 限定にする

- Priority: Low
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Branch: feature/refactor-ci-nasm-x86-only

## 目的

NASM は x86_64 SIMD アセンブリ用で arm64 では不要だが、CI / Release が arm64 ランナーにも無条件でインストールしている。x86_64 ランナーのみに条件付けする。

## 優先度根拠

ビルド時間と依存をわずかに増やすだけで害は小さい。Low。

## 現状

nasm のインストールは **5 箇所**にある。

- `ci.yml:28`: fmt-clippy ジョブ（`runs-on: ubuntu-24.04` = x86_64）の apt-get
- `ci.yml:57`: test ジョブ Ubuntu の apt-get（`build-essential meson ninja-build nasm xxd` を 1 行で）
- `ci.yml:61`: test ジョブ macOS の `brew install meson nasm`
- `release.yml:69`: build-prebuilt Ubuntu の apt-get
- `release.yml:74`: build-prebuilt macOS の `brew install meson nasm`

CI test matrix（`ci.yml:44-49`）は ubuntu-24.04 / ubuntu-24.04-arm / ubuntu-22.04 / ubuntu-22.04-arm / macos-26 / macos-15。GitHub のホスト型 macOS は現状すべて arm64。arm64 では nasm は不要。

libvmaf の `src/meson.build:45-65` で `find_program('nasm')` は `if host_machine.cpu_family().startswith('x86')` ガード下にあり、arm64 / aarch64 は NEON を使い nasm を要求しない。したがって arm64 ランナーで nasm を削除しても source-build は壊れない（確認済み）。逆に x86_64 では source-build に nasm が必須。

## 設計方針

nasm が必要なのは x86_64 Linux ランナー（ubuntu-24.04 / ubuntu-22.04、および fmt-clippy）のみ。次のように整理する。

- **x86_64 Linux**: nasm を引き続きインストールする
- **arm64 Linux（ubuntu-*-arm）**: nasm を入れない
- **macOS（全 arm64）**: nasm を入れない（`brew install meson nasm` → `brew install meson`）
- **fmt-clippy（`ci.yml:28`、ubuntu-24.04 = x86_64）**: nasm を残す（変更対象外。誤って消すと source-build の clippy が壊れる）

実装:

- Ubuntu の apt-get は nasm をまとめているため、共通インストール（`build-essential meson ninja-build xxd`）と nasm を分け、nasm のステップに `if: runner.arch == 'X64'`（または `matrix.os == 'ubuntu-24.04' || matrix.os == 'ubuntu-22.04'`）を付ける
- macOS の `brew install` は arm64 のみなので nasm を無条件で外す

`runner.arch`（`X64` / `ARM64`）で判定すれば matrix に arch フィールドを足さずに条件付けできる。

README.md:44「NASM (x86_64 のみ)」は既に整合している（ドキュメントは対象外）。

## CHANGES.md

CI / Release の設定変更で公開 API・配布物に影響しないため、初回リリースの CHANGES.md 整理（0007）の方針に従う。単独の新規エントリは不要とする。

## 完了条件

- arm64 ランナー（ubuntu-*-arm、macos-26、macos-15）で nasm をインストールしないこと（YAML の静的確認 + arm64 CI ジョブが緑）
- x86_64 ランナー（ubuntu-24.04 / ubuntu-22.04、fmt-clippy 含む）では nasm を引き続きインストールし source-build が成功すること
