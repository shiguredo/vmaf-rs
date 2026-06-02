use proptest::prelude::*;
use shiguredo_vmaf::Picture;

/// 偶数・非ゼロの比較的小さい幅と高さを生成する Strategy。
/// libvmaf が実メモリを確保する Ok 経路を通すため、2〜256 の偶数に制限する。
fn even_nonzero_dim() -> impl Strategy<Value = u32> {
    (1..=128u32).prop_map(|n| n * 2)
}

/// 偶数・非ゼロの width / height と、それに整合するプレーンデータを生成する Strategy。
fn valid_i420() -> impl Strategy<Value = (Vec<u8>, Vec<u8>, Vec<u8>, u32, u32)> {
    (even_nonzero_dim(), even_nonzero_dim()).prop_flat_map(|(w, h)| {
        let y_size = (w as usize) * (h as usize);
        let uv_size = ((w / 2) as usize) * ((h / 2) as usize);
        (
            Just(w),
            Just(h),
            prop::collection::vec(any::<u8>(), y_size),
            prop::collection::vec(any::<u8>(), uv_size),
            prop::collection::vec(any::<u8>(), uv_size),
        )
            .prop_map(|(w, h, y, u, v)| (y, u, v, w, h))
    })
}

proptest! {
    /// 偶数・非ゼロかつプレーン長が整合する入力は from_i420 が成功する。
    /// 0002（奇数拒否）の適用後であり、0002 で偶数受理を保証している。
    /// 注: 0017（ゼロ拒否）未適用のためゼロ寸法は本プロパティの範囲外。
    /// 0017 完了後にゼロ拒否プロパティを追加する。
    #[test]
    fn from_i420_は偶数非ゼロかつプレーン長整合で成功する(
        (y, u, v, w, h) in valid_i420()
    ) {
        let result = Picture::from_i420(&y, &u, &v, w, h);
        prop_assert!(result.is_ok());
    }

    /// 奇数寸法（幅・高さの少なくとも一方が奇数）は from_i420 がエラーを返す。
    /// 0002 のプロパティ。
    #[test]
    fn from_i420_は奇数寸法を拒否する(
        y in prop::collection::vec(any::<u8>(), 0..=256),
        u in prop::collection::vec(any::<u8>(), 0..=256),
        v in prop::collection::vec(any::<u8>(), 0..=256),
        (w, h) in (0u32..=256, 0u32..=256)
            .prop_filter("少なくとも一方が奇数", |(w, h)| w % 2 != 0 || h % 2 != 0),
    ) {
        let result = Picture::from_i420(&y, &u, &v, w, h);
        prop_assert!(result.is_err());
    }

    /// 偶数寸法だがプレーン長が期待値と一致しない場合は from_i420 がエラーを返す。
    /// 少なくとも 1 つのプレーン長が width*height または (width/2)*(height/2) と異なる入力を生成する。
    #[test]
    fn from_i420_はプレーン長不一致でエラーを返す(
        (y, u, v, w, h) in (even_nonzero_dim(), even_nonzero_dim(),
                             0usize..=1024, 0usize..=1024, 0usize..=1024)
            .prop_filter("少なくとも 1 つのプレーン長が不一致",
                |(w, h, yl, ul, vl)| {
                    let y_size = (*w as usize) * (*h as usize);
                    let uv_size = ((*w / 2) as usize) * ((*h / 2) as usize);
                    *yl != y_size || *ul != uv_size || *vl != uv_size
                }
            )
            .prop_map(|(w, h, yl, ul, vl)| {
                (vec![0u8; yl], vec![0u8; ul], vec![0u8; vl], w, h)
            }),
    ) {
        let result = Picture::from_i420(&y, &u, &v, w, h);
        prop_assert!(result.is_err());
    }
}
