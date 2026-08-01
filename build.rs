use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

// 依存ライブラリの名前
const LIB_NAME: &str = "vmaf";

// libvmaf の meson プロジェクトルート (clone 先リポジトリ内)
const LIBVMAF_DIR: &str = "libvmaf";

fn main() {
    // Cargo.toml か build.rs が更新されたら、依存ライブラリを再ビルドする
    println!("cargo::rerun-if-changed=Cargo.toml");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_SOURCE_BUILD");
    println!("cargo::rerun-if-env-changed=VMAF_TARGET");
    println!("cargo::rerun-if-env-changed=DOCS_RS");

    // 各種変数やビルドディレクトリのセットアップ
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("infallible"));
    let output_bindings_path = out_dir.join("bindings.rs");

    if env::var("DOCS_RS").is_ok() {
        // Docs.rs 向けのビルドでは git clone ができないので build.rs の処理はスキップして、
        // 代わりに、ドキュメント生成時に最低限必要な構造体だけをダミーで出力している。
        //
        // 参考: https://docs.rs/about/builds
        fs::write(
            output_bindings_path,
            r#"
// docs.rs 向けダミー定義
//
// docs.rs ビルドは libvmaf を clone・リンクできないため、本体 (src/lib.rs) が参照する
// 型・フィールド・定数・関数だけをダミーで定義してドキュメント生成と型チェックを通す。
// docs.rs ビルドは libvmaf にリンクせず関数本体も実行しないが、型・フィールドの順序と
// サイズを bindgen が生成する実 bindings (libvmaf v3.2.0) に一致させることで、
// 本体との型整合と保守時の乖離防止を図る。
// libvmaf の更新で構造体が変わる可能性があるため、CI の docs-rs ジョブで
// cargo build --lib によりこのダミーと src/lib.rs の型整合を検証する
// (フィールド名・型の不足は検知できるが、repr(C) レイアウトのオフセット一致は検証範囲外)。
#[repr(C)]
pub struct VmafContext {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct VmafModel {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct VmafRef {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct VmafPicture {
    pub pix_fmt: VmafPixelFormat,
    pub bpc: std::ffi::c_uint,
    pub w: [std::ffi::c_uint; 3],
    pub h: [std::ffi::c_uint; 3],
    pub stride: [isize; 3],
    pub data: [*mut std::ffi::c_void; 3],
    pub ref_: *mut VmafRef,
    pub priv_: *mut std::ffi::c_void,
}

#[repr(C)]
pub struct VmafConfiguration {
    pub log_level: VmafLogLevel,
    pub n_threads: std::ffi::c_uint,
    pub n_subsample: std::ffi::c_uint,
    pub cpumask: u64,
    pub gpumask: u64,
}

#[repr(C)]
pub struct VmafModelConfig {
    pub name: *const std::ffi::c_char,
    pub flags: u64,
}

pub type VmafPixelFormat = u32;
pub const VmafPixelFormat_VMAF_PIX_FMT_YUV420P: VmafPixelFormat = 1;

pub type VmafLogLevel = u32;
pub const VmafLogLevel_VMAF_LOG_LEVEL_NONE: VmafLogLevel = 0;
pub const VmafLogLevel_VMAF_LOG_LEVEL_ERROR: VmafLogLevel = 1;
pub const VmafLogLevel_VMAF_LOG_LEVEL_WARNING: VmafLogLevel = 2;
pub const VmafLogLevel_VMAF_LOG_LEVEL_INFO: VmafLogLevel = 3;
pub const VmafLogLevel_VMAF_LOG_LEVEL_DEBUG: VmafLogLevel = 4;

pub type VmafModelFlags = u32;
pub const VmafModelFlags_VMAF_MODEL_FLAGS_DEFAULT: VmafModelFlags = 0;

// 実 bindings は unsafe extern "C" 関数のため、呼び出し側の unsafe ブロック要件を
// 一致させるべくダミーも unsafe fn として定義する。
pub unsafe fn vmaf_init(_vmaf: *mut *mut VmafContext, _cfg: VmafConfiguration) -> std::ffi::c_int { 0 }
pub unsafe fn vmaf_close(_vmaf: *mut VmafContext) -> std::ffi::c_int { 0 }
pub unsafe fn vmaf_model_load(_model: *mut *mut VmafModel, _cfg: *mut VmafModelConfig, _version: *const std::ffi::c_char) -> std::ffi::c_int { 0 }
pub unsafe fn vmaf_model_destroy(_model: *mut VmafModel) {}
pub unsafe fn vmaf_use_features_from_model(_vmaf: *mut VmafContext, _model: *mut VmafModel) -> std::ffi::c_int { 0 }
pub unsafe fn vmaf_picture_alloc(_pic: *mut VmafPicture, _pix_fmt: VmafPixelFormat, _bpc: std::ffi::c_uint, _w: std::ffi::c_uint, _h: std::ffi::c_uint) -> std::ffi::c_int { 0 }
pub unsafe fn vmaf_picture_unref(_pic: *mut VmafPicture) -> std::ffi::c_int { 0 }
pub unsafe fn vmaf_read_pictures(_vmaf: *mut VmafContext, _ref: *mut VmafPicture, _dist: *mut VmafPicture, _index: std::ffi::c_uint) -> std::ffi::c_int { 0 }
pub unsafe fn vmaf_score_at_index(_vmaf: *mut VmafContext, _model: *mut VmafModel, _score: *mut f64, _index: std::ffi::c_uint) -> std::ffi::c_int { 0 }
pub unsafe fn vmaf_score_pooled(_vmaf: *mut VmafContext, _model: *mut VmafModel, _pool_method: VmafPoolingMethod, _score: *mut f64, _index_low: std::ffi::c_uint, _index_high: std::ffi::c_uint) -> std::ffi::c_int { 0 }
pub unsafe fn vmaf_version() -> *const std::ffi::c_char { std::ptr::null() }

pub type VmafPoolingMethod = u32;
pub const VmafPoolingMethod_VMAF_POOL_METHOD_MIN: u32 = 1;
pub const VmafPoolingMethod_VMAF_POOL_METHOD_MAX: u32 = 2;
pub const VmafPoolingMethod_VMAF_POOL_METHOD_MEAN: u32 = 3;
pub const VmafPoolingMethod_VMAF_POOL_METHOD_HARMONIC_MEAN: u32 = 4;
"#,
        )
        .expect("write file error");
        return;
    }

    let output_lib_dir = if should_use_prebuilt() {
        download_prebuilt(&out_dir)
    } else {
        build_from_source(&out_dir, &output_bindings_path)
    };

    println!("cargo::rustc-link-search={}", output_lib_dir.display());
    println!("cargo::rustc-link-lib=static={LIB_NAME}");

    // libvmaf は C++ コードを含むため C++ 標準ライブラリをリンクする
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo::rustc-link-lib=c++");
    } else if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo::rustc-link-lib=stdc++");
    }
}

// source-build feature が有効でなければ prebuilt を使う
fn should_use_prebuilt() -> bool {
    env::var("CARGO_FEATURE_SOURCE_BUILD").is_err()
}

// prebuilt バイナリをダウンロードして展開する
fn download_prebuilt(out_dir: &Path) -> PathBuf {
    let target = get_target_platform();
    let version = env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION is not set");
    let base_url = format!(
        "https://github.com/shiguredo/vmaf-rs/releases/download/{}",
        version
    );
    let archive_name = format!("vmaf-{}.tar.gz", target);
    let archive_url = format!("{}/{}", base_url, archive_name);
    let sha256_url = format!("{}/{}.sha256", base_url, archive_name);

    let archive_path = out_dir.join("prebuilt.tar.gz");
    let sha256_path = out_dir.join("prebuilt.sha256");
    let prebuilt_dir = out_dir.join("prebuilt");
    fs::create_dir_all(&prebuilt_dir).expect("failed to create prebuilt directory");

    // curl でアーカイブをダウンロード
    eprintln!("downloading prebuilt library: {}", archive_url);
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "--connect-timeout",
            "30",
            "--max-time",
            "300",
            "--retry",
            "3",
            "--retry-delay",
            "5",
            "-o",
        ])
        .arg(&archive_path)
        .arg(&archive_url)
        .output()
        .expect("failed to execute curl. Ensure curl is installed");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "failed to download prebuilt library: {}\n{}",
            archive_url, stderr
        );
    }

    // curl で SHA256 チェックサムをダウンロード
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "--connect-timeout",
            "30",
            "--max-time",
            "300",
            "--retry",
            "3",
            "--retry-delay",
            "5",
            "-o",
        ])
        .arg(&sha256_path)
        .arg(&sha256_url)
        .output()
        .expect("failed to execute curl");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "failed to download SHA256 checksum: {}\n{}",
            sha256_url, stderr
        );
    }

    // SHA256 を検証
    verify_sha256(&archive_path, &sha256_path);

    // tar で展開
    let status = Command::new("tar")
        .args(["xzf"])
        .arg(&archive_path)
        .arg("-C")
        .arg(&prebuilt_dir)
        .status()
        .expect("failed to execute tar. Ensure tar is installed");
    if !status.success() {
        panic!("failed to extract prebuilt archive");
    }

    // ライブラリファイルを OUT_DIR/lib/ にコピー
    let lib_dir = out_dir.join("lib");
    fs::create_dir_all(&lib_dir).expect("failed to create lib directory");
    fs::copy(
        prebuilt_dir.join("lib").join("libvmaf.a"),
        lib_dir.join("libvmaf.a"),
    )
    .expect("failed to copy libvmaf.a");

    // bindings.rs を OUT_DIR/ にコピー
    fs::copy(
        prebuilt_dir.join("bindings.rs"),
        out_dir.join("bindings.rs"),
    )
    .expect("failed to copy bindings.rs");

    lib_dir
}

// SHA256 チェックサムを検証する
fn verify_sha256(file_path: &Path, sha256_path: &Path) {
    let content = fs::read_to_string(sha256_path).expect("failed to read SHA256 checksum file");
    let expected = content
        .split_whitespace()
        .next()
        .expect("SHA256 checksum file is empty")
        .to_lowercase();

    if expected.len() != 64 || !expected.chars().all(|c| c.is_ascii_hexdigit()) {
        panic!(
            "SHA256 checksum file contains invalid hex string: {}...",
            &expected[..expected.len().min(32)]
        );
    }

    let actual = compute_sha256(file_path);
    if actual.len() != expected.len() || actual.len() != 64 {
        panic!(
            "SHA256 checksum length mismatch: expected={}, actual={}",
            expected.len(),
            actual.len()
        );
    }

    // 定数時間比較
    let mut diff: u8 = 0;
    for (a, e) in actual.bytes().zip(expected.bytes()) {
        diff |= a ^ e;
    }
    if diff != 0 {
        panic!(
            "SHA256 checksum mismatch:\n  expected: {}\n  actual:   {}",
            expected, actual
        );
    }
    eprintln!("SHA256 checksum verified: {}", actual);
}

// ファイルの SHA256 ハッシュを計算する
fn compute_sha256(path: &Path) -> String {
    let output = if cfg!(target_os = "macos") {
        // macOS: shasum を使用
        Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .expect("failed to execute shasum. Ensure shasum is installed")
    } else {
        // Linux: sha256sum を使用
        Command::new("sha256sum")
            .arg(path)
            .output()
            .expect("failed to execute sha256sum. Ensure coreutils is installed")
    };

    if !output.status.success() {
        panic!("failed to compute SHA256 checksum");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // shasum / sha256sum 出力形式: <hash>  <filename>
    stdout
        .split_whitespace()
        .next()
        .expect("unexpected shasum/sha256sum output format")
        .to_lowercase()
}

// ソースからビルドする
fn build_from_source(out_dir: &Path, output_bindings_path: &Path) -> PathBuf {
    let out_build_dir = out_dir.join("build/");
    let repo_dir = out_build_dir.join(LIB_NAME);
    let src_dir = repo_dir.join(LIBVMAF_DIR);
    let src_build_dir = src_dir.join("build/");
    let input_header_path = src_dir.join("include/libvmaf/libvmaf.h");
    let output_lib_dir = src_build_dir.join("src/");
    let _ = fs::remove_dir_all(&out_build_dir);
    fs::create_dir(&out_build_dir).expect("failed to create build directory");

    // 依存ライブラリのリポジトリを取得する
    git_clone_external_lib(&out_build_dir);

    // 依存ライブラリをビルドする
    fs::create_dir(&src_build_dir).expect("failed to create build directory");

    let mut meson_cmd = Command::new("meson");
    meson_cmd
        .arg("setup")
        .arg("--default-library=static")
        .arg("--buildtype=release")
        .arg("..")
        .current_dir(&src_build_dir);

    let success = meson_cmd.status().is_ok_and(|status| status.success());
    if !success {
        panic!("[meson] failed to build {LIB_NAME}");
    }

    let success = Command::new("ninja")
        .current_dir(&src_build_dir)
        .status()
        .is_ok_and(|status| status.success());
    if !success {
        panic!("[build] failed to build {LIB_NAME}");
    }

    // バインディングを生成する
    let include_dir = src_dir.join("include");
    bindgen::Builder::default()
        .header(input_header_path.to_str().expect("invalid header path"))
        .clang_arg(format!("-I{}", include_dir.display()))
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(output_bindings_path)
        .expect("failed to write bindings");

    output_lib_dir
}

// --- ヘルパー関数 ---

// CARGO_CFG_TARGET_OS + CARGO_CFG_TARGET_ARCH からプラットフォーム名を生成する
fn get_target_platform() -> String {
    if let Ok(target) = env::var("VMAF_TARGET") {
        return target;
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    match (target_os.as_str(), target_arch.as_str()) {
        ("linux", "x86_64") => format!("{}_x86_64", detect_linux_distro()),
        ("linux", "aarch64") => format!("{}_arm64", detect_linux_distro()),
        ("macos", "aarch64") => "macos_arm64".to_string(),
        ("macos", "x86_64") => "macos_x86_64".to_string(),
        _ => panic!("unsupported target: os={}, arch={}", target_os, target_arch),
    }
}

// /etc/os-release から Ubuntu バージョンを検出する
fn detect_linux_distro() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(version) = line.strip_prefix("VERSION_ID=") {
                let version = version.trim_matches('"');
                match version {
                    "22.04" | "24.04" | "26.04" => return format!("ubuntu-{}", version),
                    _ => {}
                }
            }
        }
    }
    panic!(
        "unsupported Linux distribution. \
         set VMAF_TARGET environment variable to specify the target explicitly"
    );
}

// 外部ライブラリのリポジトリを git clone する
fn git_clone_external_lib(build_dir: &Path) {
    let (git_url, version) = get_git_url_and_version();
    let repo_dir = build_dir.join(LIB_NAME);

    // shallow clone してタグをチェックアウトする
    //
    // アノテーティッドタグの場合 --branch が使えない git バージョンがあるため、
    // まず --no-checkout で clone してから fetch + checkout する
    let success = Command::new("git")
        .args(["clone", "--depth", "1", "--no-checkout"])
        .arg(&git_url)
        .arg(&repo_dir)
        .status()
        .is_ok_and(|status| status.success());
    if !success {
        panic!("failed to clone {LIB_NAME} repository");
    }

    let success = Command::new("git")
        .args(["fetch", "--depth", "1", "origin", "tag"])
        .arg(&version)
        .current_dir(&repo_dir)
        .status()
        .is_ok_and(|status| status.success());
    if !success {
        panic!("failed to fetch tag {version}");
    }

    let success = Command::new("git")
        .args(["checkout", "FETCH_HEAD"])
        .current_dir(&repo_dir)
        .status()
        .is_ok_and(|status| status.success());
    if !success {
        panic!("failed to checkout tag {version}");
    }
}

// Cargo.toml から依存ライブラリの URL とバージョンタグを取得する
fn get_git_url_and_version() -> (String, String) {
    let cargo_toml = shiguredo_toml::Value::Table(
        shiguredo_toml::from_str(include_str!("Cargo.toml")).expect("failed to parse Cargo.toml"),
    );
    if let Some((Some(url), Some(version))) = cargo_toml
        .get("package")
        .and_then(|v| v.get("metadata"))
        .and_then(|v| v.get("external-dependencies"))
        .and_then(|v| v.get(LIB_NAME))
        .map(|v| {
            (
                v.get("url").and_then(|s| s.as_str()),
                v.get("version").and_then(|s| s.as_str()),
            )
        })
    {
        (url.to_string(), version.to_string())
    } else {
        panic!(
            "Cargo.toml does not contain a valid [package.metadata.external-dependencies.{LIB_NAME}] table"
        );
    }
}
