use shiguredo_vmaf::{
    BuiltinModel, Context, ContextConfig, Error, LogLevel, Model, Picture, PoolingMethod, version,
};

/// ダミー I420 フレームを生成する
///
/// Y プレーンはフレーム番号に応じたグラデーション、UV プレーンは 128 固定。
fn generate_dummy_i420(
    width: usize,
    height: usize,
    frame_index: usize,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let y_size = width * height;
    let uv_width = width.div_ceil(2);
    let uv_height = height.div_ceil(2);
    let uv_size = uv_width * uv_height;

    let mut y = vec![0u8; y_size];
    for row in 0..height {
        for col in 0..width {
            y[row * width + col] = ((col + row + frame_index * 7) % 256) as u8;
        }
    }

    let u = vec![128u8; uv_size];
    let v = vec![128u8; uv_size];

    (y, u, v)
}

/// 参照フレームにノイズを加えた劣化フレームを生成する
fn generate_degraded_i420(
    width: usize,
    height: usize,
    frame_index: usize,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let (mut y, u, v) = generate_dummy_i420(width, height, frame_index);
    for pixel in &mut y {
        *pixel = pixel.saturating_add(32);
    }
    (y, u, v)
}

#[test]
fn version_文字列が空でない() {
    let v = version();
    assert!(!v.is_empty(), "version() が空文字列を返した");
}

#[test]
fn 同一フレームの_vmaf_スコアは高得点() {
    let width = 192;
    let height = 108;
    let (y, u, v) = generate_dummy_i420(width, height, 0);

    let mut ctx = Context::new(ContextConfig::default()).expect("Context の生成に失敗");
    let model = Model::load_builtin(BuiltinModel::V061).expect("Model の読み込みに失敗");
    ctx.use_model(&model).expect("use_model に失敗");

    let ref_pic = Picture::from_i420(&y, &u, &v, width as u32, height as u32)
        .expect("ref Picture の生成に失敗");
    let dist_pic = Picture::from_i420(&y, &u, &v, width as u32, height as u32)
        .expect("dist Picture の生成に失敗");

    ctx.read_pictures(Some(ref_pic), Some(dist_pic), 0)
        .expect("read_pictures に失敗");
    ctx.read_pictures(None, None, 0).expect("flush に失敗");

    let score = ctx
        .score_at_index(&model, 0)
        .expect("score_at_index に失敗");
    // libvmaf v3.2.0 / vmaf_v0.6.1 では同一フレームでも 100 にならない場合がある
    assert!(
        score > 95.0,
        "同一フレームの VMAF スコアは高得点のはず: got {score}"
    );
}

#[test]
fn 劣化フレームの_vmaf_スコアは低得点() {
    let width = 192;
    let height = 108;
    let (ref_y, ref_u, ref_v) = generate_dummy_i420(width, height, 0);
    let (dist_y, dist_u, dist_v) = generate_degraded_i420(width, height, 0);

    let mut ctx = Context::new(ContextConfig::default()).expect("Context の生成に失敗");
    let model = Model::load_builtin(BuiltinModel::V061).expect("Model の読み込みに失敗");
    ctx.use_model(&model).expect("use_model に失敗");

    let ref_pic = Picture::from_i420(&ref_y, &ref_u, &ref_v, width as u32, height as u32)
        .expect("ref Picture の生成に失敗");
    let dist_pic = Picture::from_i420(&dist_y, &dist_u, &dist_v, width as u32, height as u32)
        .expect("dist Picture の生成に失敗");

    ctx.read_pictures(Some(ref_pic), Some(dist_pic), 0)
        .expect("read_pictures に失敗");
    ctx.read_pictures(None, None, 0).expect("flush に失敗");

    let score = ctx
        .score_at_index(&model, 0)
        .expect("score_at_index に失敗");
    assert!(
        score < 90.0,
        "劣化フレームの VMAF スコアは低得点のはず: got {score}"
    );
}

#[test]
fn read_pictures_は_flush_後に呼ぶと_エラーを返す() {
    let width = 192;
    let height = 108;
    let (y, u, v) = generate_dummy_i420(width, height, 0);

    let mut ctx = Context::new(ContextConfig::default()).expect("Context の生成に失敗");
    let model = Model::load_builtin(BuiltinModel::V061).expect("Model の読み込みに失敗");
    ctx.use_model(&model).expect("use_model に失敗");

    let ref_pic = Picture::from_i420(&y, &u, &v, width as u32, height as u32)
        .expect("ref Picture の生成に失敗");
    let dist_pic = Picture::from_i420(&y, &u, &v, width as u32, height as u32)
        .expect("dist Picture の生成に失敗");
    ctx.read_pictures(Some(ref_pic), Some(dist_pic), 0)
        .expect("read_pictures に失敗");
    ctx.read_pictures(None, None, 0).expect("flush に失敗");

    let ref_pic = Picture::from_i420(&y, &u, &v, width as u32, height as u32)
        .expect("ref Picture の生成に失敗");
    let dist_pic = Picture::from_i420(&y, &u, &v, width as u32, height as u32)
        .expect("dist Picture の生成に失敗");
    let result = ctx.read_pictures(Some(ref_pic), Some(dist_pic), 1);
    assert!(
        matches!(
            result,
            Err(Error::InvalidInput("read_pictures called after flush"))
        ),
        "flush 後の read_pictures は InvalidInput エラーになるはず: {result:?}"
    );
}

#[test]
fn score_pooled_は_index_逆転で_ffi_エラーを返す() {
    let width = 192;
    let height = 108;
    let (y, u, v) = generate_dummy_i420(width, height, 0);

    let mut ctx = Context::new(ContextConfig::default()).expect("Context の生成に失敗");
    let model = Model::load_builtin(BuiltinModel::V061).expect("Model の読み込みに失敗");
    ctx.use_model(&model).expect("use_model に失敗");

    let ref_pic = Picture::from_i420(&y, &u, &v, width as u32, height as u32)
        .expect("ref Picture の生成に失敗");
    let dist_pic = Picture::from_i420(&y, &u, &v, width as u32, height as u32)
        .expect("dist Picture の生成に失敗");
    ctx.read_pictures(Some(ref_pic), Some(dist_pic), 0)
        .expect("read_pictures に失敗");
    ctx.read_pictures(None, None, 0).expect("flush に失敗");

    // libvmaf の vmaf_score_pooled は index_low > index_high で -EINVAL を返す
    let result = ctx.score_pooled(&model, PoolingMethod::Mean, 5, 0);
    assert!(
        matches!(
            result,
            Err(Error::Ffi {
                code,
                function: "vmaf_score_pooled"
            }) if code < 0
        ),
        "index 逆転の score_pooled は Ffi エラーになるはず: {result:?}"
    );
}

#[test]
fn read_pictures_は寸法不一致で_ffi_エラーを返す() {
    // 参照 (64x64) と劣化 (32x32) で寸法が異なると、libvmaf の validate_pic_params が
    // ref と dist の寸法不一致を検出し read_pictures がエラーを返す。
    // このエラー経路で渡した Picture が drop 時に unref され、リークしないことが本修正の狙い。
    // テストはエラーが返ること自体を確認する (リーク量はメモリ計測が必要なため検証対象外)。
    let mut ctx = Context::new(ContextConfig::default()).expect("Context の生成に失敗");
    let model = Model::load_builtin(BuiltinModel::V061).expect("Model の読み込みに失敗");
    ctx.use_model(&model).expect("use_model に失敗");

    let (ref_y, ref_u, ref_v) = generate_dummy_i420(64, 64, 0);
    let (dist_y, dist_u, dist_v) = generate_dummy_i420(32, 32, 0);
    let ref_pic =
        Picture::from_i420(&ref_y, &ref_u, &ref_v, 64, 64).expect("ref Picture の生成に失敗");
    let dist_pic =
        Picture::from_i420(&dist_y, &dist_u, &dist_v, 32, 32).expect("dist Picture の生成に失敗");

    let result = ctx.read_pictures(Some(ref_pic), Some(dist_pic), 0);
    assert!(
        matches!(
            result,
            Err(Error::Ffi {
                code,
                function: "vmaf_read_pictures"
            }) if code < 0
        ),
        "寸法不一致の read_pictures は Ffi エラーになるはず: {result:?}"
    );
}

#[test]
fn read_pictures_はスレッド設定でも寸法不一致でエラーを返す() {
    // n_threads>0 でスレッドプール付き Context を生成する。寸法不一致は
    // validate_pic_params で早期に弾かれるため threaded_read_pictures_batch までは
    // 到達しないが、スレッド設定の Context でも read_pictures がエラー時に
    // パニックせず Err を返すことを確認する。
    let config = ContextConfig {
        n_threads: 2,
        ..ContextConfig::default()
    };
    let mut ctx = Context::new(config).expect("Context の生成に失敗");
    let model = Model::load_builtin(BuiltinModel::V061).expect("Model の読み込みに失敗");
    ctx.use_model(&model).expect("use_model に失敗");

    let (ref_y, ref_u, ref_v) = generate_dummy_i420(64, 64, 0);
    let (dist_y, dist_u, dist_v) = generate_dummy_i420(32, 32, 0);
    let ref_pic =
        Picture::from_i420(&ref_y, &ref_u, &ref_v, 64, 64).expect("ref Picture の生成に失敗");
    let dist_pic =
        Picture::from_i420(&dist_y, &dist_u, &dist_v, 32, 32).expect("dist Picture の生成に失敗");

    let result = ctx.read_pictures(Some(ref_pic), Some(dist_pic), 0);
    assert!(
        matches!(
            result,
            Err(Error::Ffi {
                code,
                function: "vmaf_read_pictures"
            }) if code < 0
        ),
        "スレッド設定でも寸法不一致の read_pictures は Ffi エラーになるはず: {result:?}"
    );
}

#[test]
fn from_i420_はゼロ寸法を拒否する() {
    // ゼロ寸法は確定境界値のため単体テストで検証する (PBT では生成確率が低く、
    // 全ケースでゼロ寸法が生成されない可能性がある)。
    for (w, h) in [(0, 2), (2, 0), (0, 0)] {
        let result = Picture::from_i420(&[], &[], &[], w, h);
        assert!(
            result.is_err(),
            "width={w} height={h} は from_i420 で拒否されるはず"
        );
    }
}

#[test]
fn 複数フレームを_mean_でプールできる() {
    let width = 192;
    let height = 108;
    let frame_count: u32 = 3;

    let mut ctx = Context::new(ContextConfig::default()).expect("Context の生成に失敗");
    let model = Model::load_builtin(BuiltinModel::V061).expect("Model の読み込みに失敗");
    ctx.use_model(&model).expect("use_model に失敗");

    for i in 0..frame_count {
        let (y, u, v) = generate_dummy_i420(width, height, i as usize);
        let (dy, du, dv) = generate_degraded_i420(width, height, i as usize);
        let ref_pic = Picture::from_i420(&y, &u, &v, width as u32, height as u32)
            .expect("ref Picture の生成に失敗");
        let dist_pic = Picture::from_i420(&dy, &du, &dv, width as u32, height as u32)
            .expect("dist Picture の生成に失敗");
        ctx.read_pictures(Some(ref_pic), Some(dist_pic), i)
            .expect("read_pictures に失敗");
    }
    ctx.read_pictures(None, None, 0).expect("flush に失敗");

    let mut scores = Vec::new();
    for i in 0..frame_count {
        let score = ctx
            .score_at_index(&model, i)
            .expect("score_at_index に失敗");
        scores.push(score);
    }

    let mean_expected = scores.iter().sum::<f64>() / scores.len() as f64;
    let pooled = ctx
        .score_pooled(&model, PoolingMethod::Mean, 0, frame_count - 1)
        .expect("score_pooled に失敗");

    let diff = (pooled - mean_expected).abs();
    assert!(
        diff < 0.1,
        "pooled Mean は score_at_index の平均と一致するはず: pooled={pooled}, expected={mean_expected}"
    );
}

#[test]
fn builtin_model_のバージョン文字列が正しい() {
    assert_eq!(BuiltinModel::V061.version_str(), "vmaf_v0.6.1");
    assert_eq!(BuiltinModel::BV063.version_str(), "vmaf_b_v0.6.3");
    assert_eq!(BuiltinModel::V061Neg.version_str(), "vmaf_v0.6.1neg");
    assert_eq!(BuiltinModel::V4k061.version_str(), "vmaf_4k_v0.6.1");
    assert_eq!(BuiltinModel::V4k061Neg.version_str(), "vmaf_4k_v0.6.1neg");
}

#[test]
fn 全組み込みモデルをロードできる() {
    // BV063 (vmaf_b_v0.6.3) はフリービルド版 libvmaf に含まれていないため除外
    let models = [
        (BuiltinModel::V061, "V061"),
        (BuiltinModel::V061Neg, "V061Neg"),
        (BuiltinModel::V4k061, "V4k061"),
        (BuiltinModel::V4k061Neg, "V4k061Neg"),
    ];
    let mut failures = Vec::new();
    for (model, name) in models {
        if let Err(e) = Model::load_builtin(model) {
            failures.push(format!("{name}: {e}"));
        }
    }
    assert!(
        failures.is_empty(),
        "ロードに失敗したモデル:\n{}",
        failures.join("\n")
    );
}

#[test]
fn 全ログレベルでコンテキストを生成できる() {
    for level in [
        LogLevel::None,
        LogLevel::Error,
        LogLevel::Warning,
        LogLevel::Info,
        LogLevel::Debug,
    ] {
        let config = ContextConfig {
            log_level: level,
            ..ContextConfig::default()
        };
        let _ = Context::new(config).expect("Context の生成に失敗");
    }
}

#[test]
fn 複数フレームを全プーリングメソッドで集計できる() {
    let width = 192;
    let height = 108;
    let frame_count: u32 = 3;

    let mut ctx = Context::new(ContextConfig::default()).expect("Context の生成に失敗");
    let model = Model::load_builtin(BuiltinModel::V061).expect("Model の読み込みに失敗");
    ctx.use_model(&model).expect("use_model に失敗");

    for i in 0..frame_count {
        let (y, u, v) = generate_dummy_i420(width, height, i as usize);
        let (dy, du, dv) = generate_degraded_i420(width, height, i as usize);
        let ref_pic = Picture::from_i420(&y, &u, &v, width as u32, height as u32)
            .expect("ref Picture の生成に失敗");
        let dist_pic = Picture::from_i420(&dy, &du, &dv, width as u32, height as u32)
            .expect("dist Picture の生成に失敗");
        ctx.read_pictures(Some(ref_pic), Some(dist_pic), i)
            .expect("read_pictures に失敗");
    }
    ctx.read_pictures(None, None, 0).expect("flush に失敗");

    let mut scores = Vec::new();
    for i in 0..frame_count {
        let score = ctx
            .score_at_index(&model, i)
            .expect("score_at_index に失敗");
        scores.push(score);
    }

    let min = ctx
        .score_pooled(&model, PoolingMethod::Min, 0, frame_count - 1)
        .expect("score_pooled Min に失敗");
    let max = ctx
        .score_pooled(&model, PoolingMethod::Max, 0, frame_count - 1)
        .expect("score_pooled Max に失敗");
    let mean = ctx
        .score_pooled(&model, PoolingMethod::Mean, 0, frame_count - 1)
        .expect("score_pooled Mean に失敗");
    let harmonic = ctx
        .score_pooled(&model, PoolingMethod::HarmonicMean, 0, frame_count - 1)
        .expect("score_pooled HarmonicMean に失敗");

    assert!(
        min <= mean && mean <= max,
        "Min <= Mean <= Max が成立するはず: min={min}, mean={mean}, max={max}"
    );

    // libvmaf の vmaf_feature_score_pooled は (スコア+1) の調和平均から 1 を引いた値を返す
    // 根拠: libvmaf/src/libvmaf.c v3.2.0 の HARMONIC_MEAN 実装 (i_sum += 1/(s+1), score = pic_cnt/i_sum - 1)
    // 将来 libvmaf の更新で変更される可能性がある
    let harmonic_expected =
        scores.len() as f64 / scores.iter().map(|s| 1.0 / (s + 1.0)).sum::<f64>() - 1.0;
    let harmonic_diff = (harmonic - harmonic_expected).abs();
    assert!(
        harmonic_diff < 0.01,
        "pooled HarmonicMean は libvmaf の実装式と一致するはず: pooled={harmonic}, expected={harmonic_expected}"
    );
}

#[test]
fn error_invalid_input_の表示が正しい() {
    let err = Error::InvalidInput("テストメッセージ");
    assert!(err.to_string().contains("テストメッセージ"));
}

#[test]
fn error_ffi_の表示が正しい() {
    let err = Error::Ffi {
        code: -1,
        function: "test_func",
    };
    let s = err.to_string();
    assert!(
        s.contains("test_func"),
        "Ffi エラー表示に関数名が含まれていない: {s}"
    );
    assert!(
        s.contains("failed"),
        "Ffi エラー表示に failure 表示が含まれていない: {s}"
    );
}

#[test]
fn error_は_std_error_トレイトを実装している() {
    let err = Error::InvalidInput("テスト");
    let _: &dyn std::error::Error = &err;
}
