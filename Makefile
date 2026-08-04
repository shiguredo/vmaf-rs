.PHONY: test test-quick codec-bench codec-match codec-bench-all cover check clippy clippy-all fmt clean

# 全テスト (重い統合テスト含む)
test:
	cargo test --workspace --features source-build

# CI 相当の軽いテスト (バインディングのみ)
test-quick:
	cargo test --features source-build --test test_lib

# ローカル試行: コンテンツ × コーデック × ビットレートの表を出力 (assert なし)
codec-bench:
	cargo test --features source-build --test codec_vmaf local_vmaf_bench_report -- --nocapture

# ローカル試行: 目標 VMAF ごとに AOM / VP9 の kbps を二分探索
codec-match:
	cargo test --features source-build --test codec_vmaf local_vmaf_y4m_same_score_bitrate_search -- --nocapture

# 統合テスト一式 + レポート
codec-bench-all:
	cargo test --features source-build --test codec_vmaf -- --nocapture

# 全テストカバレッジ付きで実行する
cover:
	cargo llvm-cov --tests --workspace --features source-build

check:
	cargo check --lib --features source-build

# cargo clippy を実行する (CI 相当: ライブラリのみ)
clippy:
	cargo clippy --lib --features source-build -- -D warnings

# ベンチマークテスト含む clippy
clippy-all:
	cargo clippy --workspace --all-targets --features source-build -- -D warnings

fmt:
	cargo fmt --all

clean:
	cargo clean
