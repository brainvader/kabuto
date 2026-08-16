//! 相関計算。PlayGroundのCALCノードが使う想定のロジックをここに集約する
//! （2026/08/15/007.md Step 5）。

/// ピアソンの積率相関係数を計算する。
/// 長さが一致しない、2件未満、またはどちらかの分散が0（値が全て同じ）の場合はNone。
pub fn pearson_correlation(a: &[f64], b: &[f64]) -> Option<f64> {
    if a.len() != b.len() || a.len() < 2 {
        return None;
    }

    let n = a.len() as f64;
    let mean_a = a.iter().sum::<f64>() / n;
    let mean_b = b.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    for i in 0..a.len() {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }

    if var_a == 0.0 || var_b == 0.0 {
        return None;
    }

    Some(cov / (var_a.sqrt() * var_b.sqrt()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_positive_correlation() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0];
        let b = [2.0, 4.0, 6.0, 8.0, 10.0];
        let r = pearson_correlation(&a, &b).unwrap();
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn perfect_negative_correlation() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0];
        let b = [10.0, 8.0, 6.0, 4.0, 2.0];
        let r = pearson_correlation(&a, &b).unwrap();
        assert!((r + 1.0).abs() < 1e-9);
    }

    #[test]
    fn no_variance_returns_none() {
        let a = [1.0, 1.0, 1.0];
        let b = [1.0, 2.0, 3.0];
        assert_eq!(pearson_correlation(&a, &b), None);
    }

    #[test]
    fn mismatched_length_returns_none() {
        let a = [1.0, 2.0];
        let b = [1.0, 2.0, 3.0];
        assert_eq!(pearson_correlation(&a, &b), None);
    }
}
