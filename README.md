# vmaf-rs

[![crates.io](https://img.shields.io/crates/v/shiguredo_vmaf.svg)](https://crates.io/crates/shiguredo_vmaf)
[![docs.rs](https://docs.rs/shiguredo_vmaf/badge.svg)](https://docs.rs/shiguredo_vmaf)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![GitHub Actions](https://github.com/shiguredo/vmaf-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/shiguredo/vmaf-rs/actions/workflows/ci.yml)
[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?logo=discord&logoColor=white)](https://discord.gg/shiguredo)

## About Shiguredo's open source software

We will not respond to PRs or issues that have not been discussed on Discord. Also, Discord is only available in Japanese.

Please read <https://github.com/shiguredo/oss> before use.

## 時雨堂のオープンソースソフトウェアについて

利用前に <https://github.com/shiguredo/oss> をお読みください。

## 概要

[VMAF](https://github.com/Netflix/vmaf) (Video Multi-Method Assessment Fusion) の Rust バインディングです。

## 特徴

- フルリファレンス VMAF スコア計算 (8-bit I420)
- 組み込み VMAF モデル対応
- prebuilt バイナリによる高速ビルド (デフォルト)
- ソースからのビルドも可能 (`--features source-build`)

## 動作要件

- Ubuntu 24.04 x86_64
- Ubuntu 24.04 arm64
- Ubuntu 22.04 x86_64
- Ubuntu 22.04 arm64
- macOS 26 arm64
- macOS 15 arm64

### ソースビルド時の追加要件

- Git
- C/C++ コンパイラ (`build-essential` 等)
- Meson + Ninja
- NASM (x86_64 のみ)
- xxd (Ubuntu のみ apt でインストール。macOS は Xcode CLT 付属)

```bash
# Ubuntu
sudo apt-get install -y build-essential meson ninja-build nasm xxd

# macOS (xxd は Xcode CLT 付属)
brew install meson nasm
```

## ビルド

デフォルトでは GitHub Releases から prebuilt バイナリをダウンロードしてビルドします。

```bash
cargo build
```

### ソースからビルド

VMAF をソースからビルドする場合は `source-build` feature を有効にしてください。

```bash
cargo build --features source-build
```

### docs.rs 向けビルド

VMAF がない環境では、docs.rs 向けのドキュメント生成のみ可能です。

```bash
DOCS_RS=1 cargo doc --no-deps --no-default-features
```

## 使い方

```rust
use shiguredo_vmaf::{BuiltinModel, Context, ContextConfig, Model, Picture};

let mut ctx = Context::new(ContextConfig::default())?;
let model = Model::load_builtin(BuiltinModel::V061)?;
ctx.use_model(&model)?;

let (y, u, v) = /* I420 ピクセルデータ */;
let ref_pic = Picture::from_i420(&y, &u, &v, width, height)?;
let dist_pic = Picture::from_i420(&y, &u, &v, width, height)?;

ctx.read_pictures(Some(ref_pic), Some(dist_pic), 0)?;
ctx.read_pictures(None, None, 0)?; // flush

let score = ctx.score_at_index(&model, 0)?;
println!("VMAF score: {score}");
```

## 環境変数

| 変数 | 説明 |
|---|---|
| `VMAF_TARGET` | prebuilt バイナリのプラットフォーム名を明示的に指定する |
| `VMAF_BENCH_BITRATES` | ローカルベンチ (`local_vmaf_ベンチレポート`) のビットレート一覧 (kbps, カンマ区切り。例: `7,18,34,100`) |

## ローカルでのコーデック + VMAF 試行

CI では実行しません。**ローカルでコンテンツ別・ビットレート別の挙動を見る** 用途向けです。
リアルタイム符号化設定を使います。

```bash
# 表形式レポート (assert なし)
make codec-bench

# ビットレートを指定
VMAF_BENCH_BITRATES=7,18,34,100 make codec-bench

# 統合テスト + レポート一式
make codec-bench-all
```

合成コンテンツ (面談・スポーツ系の簡易代理):

| ラベル | イメージ |
|---|---|
| `conferencing (static)` | 面談・静止スライド |
| `conferencing (motion)` | 面談 + UI 要素の移動 |
| `sports-proxy` | モータースポーツ / ゲーム系の高動き代理 |

macOS では Video Toolbox による **H.264 / H.265** もベンチ対象に含まれます (ハードウェア符号化対応 Mac のみ)。

実クリップ (Xiph 等) を試す場合は `local_videos/` に置き、今後の拡張用ディレクトリとして利用できます (git 管理外)。

## VMAF ライセンス

<https://github.com/Netflix/vmaf/blob/master/LICENSE>

```text
LICENSE - BSD+Patent
SPDX short identifier: BSD-2-Clause-Patent

Note: This license is designed to provide: a) a simple permissive license; b) that is compatible with the GNU General
Public License (GPL), version 2; and c) which also has an express patent grant included.

Copyright (c) 2020 Netflix, Inc.

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following
disclaimer in the documentation and/or other materials provided with the distribution.

Subject to the terms and conditions of this license, each copyright holder and contributor hereby grants to those
receiving rights under this license a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable (except
for failure to satisfy the conditions of this license) patent license to make, have made, use, offer to sell, sell,
import, and otherwise transfer this software, where such license applies only to those patent claims, already acquired
or hereafter acquired, licensable by such copyright holder or contributor that are necessarily infringed by:

(a) their Contribution(s) (the licensed copyrights of copyright holders and non-copyrightable additions of contributors,
in source or binary form) alone; or

(b) combination of their Contribution(s) with the work of authorship to which such Contribution(s) was added by such
copyright holder or contributor, if, at the time the Contribution is added, such addition causes such combination to be
necessarily infringed. The patent license shall not apply to any other combinations which include the Contribution.

Except as expressly stated above, no rights or licenses from any copyright holder or contributor is granted under this
license, whether expressly, by implication, estoppel or otherwise.

DISCLAIMER

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDERS OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```

## ライセンス

Apache License 2.0

```text
Copyright 2026-2026, Shiguredo Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
