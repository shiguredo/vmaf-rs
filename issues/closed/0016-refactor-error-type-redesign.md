# Error 型を入力検証エラーと FFI エラーで区別できるよう再設計する

- Priority: High
- Created: 2026-05-29
- Polished: 2026-06-06
- Completed: 2026-06-06
- Model: Opus 4.8
- Branch: feature/refactor-error-type-redesign

## 目的

`Error` 型が libvmaf 由来のエラーとクレート独自の入力検証エラーを同じ `code: c_int` に潰しており、`from_i420` のサイズエラーを `code: -22`（EINVAL）という魔法数で libvmaf エラーに偽装している。型として正直な表現にする。

`Error` のフィールドは現状すべて private（opaque struct）で、利用者は Display / Debug / `std::error::Error` 経由でしか扱えない。enum 化して public バリアントを与えるのは利用者にマッチ機能を追加する変更で既存コードを壊さず、かつ version 2026.0.0 は未公開のため後方互換上の問題はない。

## 優先度根拠

利用者が `code` を errno として誤解する。プラットフォーム非保証の魔法数依存もある。加えて本 issue は「エラー表現を 0016 に委ねる」と複数 issue が参照する依存ハブで、0017（ゼロ寸法拒否）が `Error::InvalidInput` 導入を本 issue の先行に依存し、0015（score_pooled）もリベース対象とする。`from_i420` の魔法数 `-22`（EINVAL）が公開 API に残存しており、version 2026.0.0 未公開のうちにエラー型を確定すべき。被依存の 0017 と同格（Medium）だと着手順序が逆転するリスクがあるため High。

## 現状

- `src/lib.rs:61-64`: `Error { code: c_int, function: &'static str }` で、libvmaf エラーと自前検証エラーを区別できない
- `src/lib.rs:67-73`: `Error::check(code, function)` が FFI ラッパ 6 箇所（`vmaf_init` 156, `vmaf_use_features_from_model` 164, `vmaf_read_pictures` 193, `vmaf_score_at_index` 212, `vmaf_model_load` 246, `vmaf_picture_alloc` 303）から使われている
- `src/lib.rs:293-298`: `from_i420` のサイズ不整合を `Error { code: -22, function: "Picture::from_i420" }` と libvmaf エラーに偽装。`-22`（EINVAL）はプラットフォーム非保証の魔法数で、`function` も実 C 関数名でなくメソッド名
- `src/lib.rs:76-79`: Display は `code={}` で生の負数を出すだけ。errno であることが伝わらない

## 設計方針

### enum 定義

```rust
pub enum Error {
    /// クレート側の入力検証エラー
    InvalidInput(&'static str),
    /// libvmaf FFI 由来エラー (負の errno code)
    Ffi { code: c_int, function: &'static str },
}
```

- `InvalidInput` は固定メッセージ `&'static str` を持つ（サイズ不一致・奇数寸法（0002）・ゼロ寸法（0017）の理由をメッセージで表す）。C 関数名フィールドを持たないため、現状の「`function` が実 C 関数名でない」問題（`from_i420`）が構造的に解消する
- `Ffi` は現状の `code` / `function` を保持し、`function` には実 C 関数名のみが入る
- 頭字語は Rust API Guidelines に従い `Ffi`（`FFI` ではない）。`InvalidInput` は `std::io::ErrorKind::InvalidInput` と同名で慣習的
- 将来のバリアント追加に備え `#[non_exhaustive]` を付けるか検討する
- `Error::check`（`src/lib.rs:67-73`）は `Error::Ffi { code, function }` を生成するよう変更する。FFI ラッパ 6 箇所は `check` 経由なので呼び出し側の変更は不要

### Display

`match self` で両バリアントを書く。

```rust
match self {
    Error::InvalidInput(msg) => write!(f, "{msg}"),
    Error::Ffi { code, function } => write!(
        f, "{function}() failed: {}",
        std::io::Error::from_raw_os_error(-code)
    ),
}
```

libvmaf は **負の** errno（例: `-22` = `-EINVAL`）を返すのに対し `std::io::Error::from_raw_os_error` は正の errno を期待する。`from_raw_os_error(code)`（負のまま）では "Unknown error: -22" になり errno 文字列化できないため、**`from_raw_os_error(-code)` と符号を反転する**（実機確認: `from_raw_os_error(22)` は "Invalid argument"）。これで依存追加なしに可読性が上がる。

`#[derive(Debug)]` と空の `impl std::error::Error` は両バリアント共通でそのまま維持する。

### 魔法数の撤廃

`src/lib.rs:293-298` の `Error { code: -22, ... }` を `Error::InvalidInput("...")` に置き換える。

## 関連 issue との整合

本 issue は複数 issue が「エラー表現は 0016 に委ねる」と参照する結節点である。

- 0002（High、奇数寸法拒否）は `from_i420` の検証エラーを暫定的に `-22` のまま実装する（0002 が明記）。本 issue がその `-22` ブロック（`src/lib.rs:293-298`）を `InvalidInput` に置換する具体的対象
- 0017（Medium、ゼロ寸法拒否、番号は 0016 より後）は 0016 完了後の `InvalidInput` を使う前提
- 0015（Medium、score_pooled 追加）が先に入ると `score_pooled` の戻り値 `Result<_, Error>` も `Error::check` 経由で本変更の対象に含まれる。どちらが先でも他方のリベースが必要
- 番号順（0002 → 0015 → 0016 → 0017）で対応すれば整合する

## CHANGES.md

`Error` は opaque struct で機能的な振る舞い（エラーを返すこと自体）は変わらず、型表現と Display 文字列の改善であるため、`## develop` の `### misc` に `[CHANGE]` エントリ（Display 出力が errno 文字列化される利用者可視の変更を含むため）を `- @voluntas` 付きで追記する。

## 完了条件

- `Error` が `InvalidInput` と `Ffi` のバリアントを持ち、入力検証エラーと FFI エラーが型で区別できること
- `from_i420` の検証エラーが `InvalidInput` になり、`-22` 魔法数が撤廃されていること
- `Ffi` の Display が `from_raw_os_error(-code)` により errno 文字列（例: "Invalid argument"）を含むこと
- `from_i420` の検証エラーが `InvalidInput` バリアントになることを確認する単体テストと、Display が errno 文字列を含むことを確認する単体テストがあること

## 解決方法

- `Error` を struct から enum `{ InvalidInput(&'static str), Ffi { code, function } }` に変更した
- `Error::check` は `Ffi` バリアントを生成するよう変更した（呼び出し側 6 箇所の変更不要）
- `from_i420` の `-22` 魔法数を `InvalidInput` に置き換えた
- Display で `from_raw_os_error(-code)` を使い errno 文字列化するよう変更した
- 変更ファイル: `src/lib.rs`（1 ファイル）
