# ライセンス表記を Apache-2.0 AND BSD-2-Clause-Patent にする

- Priority: High
- Created: 2026-05-29
- Polished: 2026-05-29
- Model: Opus 4.8
- Completed: 2026-06-02
- Branch: feature/fix-license-spdx

## 目的

本クレートは libvmaf (BSD-2-Clause-Patent) を静的リンクして配布するが、`Cargo.toml` の `license` が `Apache-2.0` 単独で、生成バイナリに含まれる libvmaf のライセンスと patent grant 条項を表していない。`license` フィールドを生成バイナリの実態に合わせる。

crates.io にアップロードされるソース tarball に libvmaf 本体は含まれないが、`license` フィールドはパッケージをビルドして得られる成果物のライセンス条件を表すべきであり、その成果物には libvmaf が静的リンクされる。したがって両者の併記が正確である。

## 優先度根拠

ライセンス表記の不正確さは OSS 公開における法的リスクであり、ライセンススキャナ (cargo-deny / cargo-about 等は `license` フィールドを読む) が BSD-2-Clause-Patent の patent grant を検出できない。公開前に解消すべき High。

## 現状

- `Cargo.toml:10` は `license = "Apache-2.0"`
- `build.rs:88` で `cargo::rustc-link-lib=static=vmaf` により libvmaf を静的リンクする。libvmaf の入手経路は prebuilt ダウンロード (`build.rs:104-`) と `source-build` feature でのソースビルド (`build.rs:248-`) の 2 系統あるが、いずれの経路でも生成バイナリに BSD-2-Clause-Patent コードが取り込まれる
- libvmaf は BSD-2-Clause-Patent (`LICENSE-THIRD-PARTY:1-41` に全文同梱済み、`Cargo.toml:12-18` の `include` で同梱済み)
- `DOCS_RS` 経路 (`build.rs:39-79`) は libvmaf をリンクせずダミー定義のみだが、`license` フィールドは通常の配布形態 (静的リンク) を基準とするため AND 併記で問題ない

## 設計方針

`Cargo.toml:10` の `license` を SPDX 式の併記にする。利用者は両ライセンスの条件を同時に遵守する必要があるため、論理積 `AND` を用いる (選択ライセンスの `OR` ではない)。

```toml
license = "Apache-2.0 AND BSD-2-Clause-Patent"
```

`BSD-2-Clause-Patent` は valid な SPDX 短縮識別子 (OSI 承認済み、SPDX License List 収録)。`Apache-2.0 AND BSD-2-Clause-Patent` は valid な SPDX 式で crates.io も受理する。

`LICENSE-THIRD-PARTY` の同梱・`include` (`Cargo.toml:12-18`) は維持する。変更は `Cargo.toml:10` の 1 行のみで、他に変更を要する箇所はない (README はドキュメントのため対象外)。

## 完了条件

- `license` フィールドが `Apache-2.0 AND BSD-2-Clause-Patent` になっていること (`cargo metadata` で確認)
- `cargo publish --dry-run` がローカルの SPDX 検証を含めて成功すること
- `CHANGES.md` の `## develop` に `[FIX]` エントリを `- @voluntas` 担当者行付きで追記すること

## 解決方法

コード変更なしでクローズする。

issue の前提「`license` フィールドは生成バイナリのライセンス条件を表すべき」が誤り。`Cargo.toml` の `license` フィールドはクレート自身のソースコードのライセンスを表すものであり、本クレートの Rust ソースコードは Apache-2.0 である。libvmaf の prebuilt バイナリは利用者が別途入手するもので、クレートの `license` フィールドに含める必要はない。libvmaf のライセンス (BSD-2-Clause-Patent) は既に `README.md` と `LICENSE-THIRD-PARTY` で明示されている。
