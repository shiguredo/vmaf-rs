use shiguredo_vmaf::{
    BuiltinModel, Context, ContextConfig, Model, Picture, PoolingMethod, version,
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
    // libvmaf v3.0.0 / vmaf_v0.6.1 では同一フレームでも 100 にならない場合がある
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
fn read_pictures_は寸法不一致でエラーを返す() {
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
        result.is_err(),
        "寸法不一致の read_pictures はエラーになるはず"
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
        result.is_err(),
        "スレッド設定でも寸法不一致の read_pictures はエラーになるはず"
    );
}

#[test]
fn プレーンサイズ不一致はエラーになる() {
    let width = 64;
    let height = 64;
    let y = vec![0u8; width * height];
    let u = vec![128u8; 1]; // 意図的にサイズ不一致
    let v = vec![128u8; (width / 2) * (height / 2)];

    let result = Picture::from_i420(&y, &u, &v, width as u32, height as u32);
    assert!(result.is_err(), "プレーンサイズ不一致はエラーになるはず");
}

#[test]
fn from_i420_は奇数幅を拒否する() {
    // 奇数幅 (3x4) は偶数寸法ガードで即座に EINVAL となる。
    // U/V は ceil クロマサイズ (2x2=4) で用意しており、ガードが無ければサイズ検証を
    // 通過してしまう。つまりこのテストはガードが効いていること自体を検証する。
    let width = 3u32;
    let height = 4u32;
    let y = vec![0u8; (width * height) as usize];
    let u = vec![128u8; 4];
    let v = vec![128u8; 4];

    let result = Picture::from_i420(&y, &u, &v, width, height);
    assert!(result.is_err(), "奇数幅は from_i420 で拒否されるはず");
}

#[test]
fn from_i420_は奇数高を拒否する() {
    let width = 4u32;
    let height = 3u32;
    let y = vec![0u8; (width * height) as usize];
    let u = vec![128u8; 4];
    let v = vec![128u8; 4];

    let result = Picture::from_i420(&y, &u, &v, width, height);
    assert!(result.is_err(), "奇数高は from_i420 で拒否されるはず");
}

#[test]
fn from_i420_は幅高とも奇数を拒否する() {
    let width = 3u32;
    let height = 3u32;
    let y = vec![0u8; (width * height) as usize];
    let u = vec![128u8; 4];
    let v = vec![128u8; 4];

    let result = Picture::from_i420(&y, &u, &v, width, height);
    assert!(
        result.is_err(),
        "幅高とも奇数の入力は from_i420 で拒否されるはず"
    );
}

#[test]
fn from_i420_は偶数寸法を受理する() {
    // 偶数寸法 (2x2) は従来どおり受理される (回帰確認)。
    let width = 2u32;
    let height = 2u32;
    let y = vec![0u8; (width * height) as usize];
    let u = vec![128u8; 1];
    let v = vec![128u8; 1];

    let result = Picture::from_i420(&y, &u, &v, width, height);
    assert!(result.is_ok(), "偶数寸法は from_i420 で受理されるはず");
}

#[test]
fn from_i420_はゼロ幅を拒否する() {
    let y = vec![];
    let u = vec![];
    let v = vec![];
    let result = Picture::from_i420(&y, &u, &v, 0, 2);
    assert!(result.is_err(), "width=0 は from_i420 で拒否されるはず");
}

#[test]
fn from_i420_はゼロ高を拒否する() {
    let y = vec![];
    let u = vec![];
    let v = vec![];
    let result = Picture::from_i420(&y, &u, &v, 2, 0);
    assert!(result.is_err(), "height=0 は from_i420 で拒否されるはず");
}

#[test]
fn from_i420_はゼロ寸法両方を拒否する() {
    let y = vec![];
    let u = vec![];
    let v = vec![];
    let result = Picture::from_i420(&y, &u, &v, 0, 0);
    assert!(
        result.is_err(),
        "width=height=0 は from_i420 で拒否されるはず"
    );
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
