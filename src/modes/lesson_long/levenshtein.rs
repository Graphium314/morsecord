use std::cmp::min;

/// スペースの挿入/削除を 0 コストにした Levenshtein + アラインメント
///
/// 戻り値: (aligned_a, aligned_b, distance)
pub fn compare_str(a: &str, b: &str) -> (String, String, usize) {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let (m, n) = (a_chars.len(), b_chars.len());

    // ---------- 1. DP テーブル ----------
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        dp[i][0] = dp[i - 1][0] + del_cost(a_chars[i - 1]);
    }
    for j in 1..=n {
        dp[0][j] = dp[0][j - 1] + ins_cost(b_chars[j - 1]);
    }

    for i in 1..=m {
        for j in 1..=n {
            let del = dp[i - 1][j] + del_cost(a_chars[i - 1]);
            let ins = dp[i][j - 1] + ins_cost(b_chars[j - 1]);
            let sub = dp[i - 1][j - 1] + sub_cost(a_chars[i - 1], b_chars[j - 1]);
            dp[i][j] = min(min(del, ins), sub);
        }
    }

    // ---------- 2. 逆トレース ----------
    let mut aligned_a = Vec::<char>::new();
    let mut aligned_b = Vec::<char>::new();
    let (mut i, mut j) = (m, n);

    while i > 0 || j > 0 {
        // マッチ
        if i > 0 && j > 0 && a_chars[i - 1] == b_chars[j - 1] && dp[i][j] == dp[i - 1][j - 1] {
            aligned_a.push(a_chars[i - 1]);
            aligned_b.push(b_chars[j - 1]);
            i -= 1;
            j -= 1;
            continue;
        }

        let cur = dp[i][j];
        let can_del = i > 0 && cur == dp[i - 1][j] + del_cost(a_chars[i - 1]);
        let can_ins = j > 0 && cur == dp[i][j - 1] + ins_cost(b_chars[j - 1]);
        let can_sub =
            i > 0 && j > 0 && cur == dp[i - 1][j - 1] + sub_cost(a_chars[i - 1], b_chars[j - 1]);

        // tie-break: 削除 → 挿入 → 置換
        if can_del {
            aligned_a.push(a_chars[i - 1]);
            aligned_b.push(' ');
            i -= 1;
        } else if can_ins {
            aligned_a.push(' ');
            aligned_b.push(b_chars[j - 1]);
            j -= 1;
        } else if can_sub {
            aligned_a.push(a_chars[i - 1]);
            aligned_b.push(b_chars[j - 1]);
            i -= 1;
            j -= 1;
        } else {
            unreachable!("DP trace back failed");
        }
    }

    aligned_a.reverse();
    aligned_b.reverse();

    (
        aligned_a.iter().collect(),
        aligned_b.iter().collect(),
        dp[m][n],
    )
}

/*----- コスト関数 -----*/
#[inline]
fn del_cost(c: char) -> usize {
    if c == ' ' {
        0
    } else {
        1
    }
}
#[inline]
fn ins_cost(c: char) -> usize {
    if c == ' ' {
        0
    } else {
        1
    }
}
#[inline]
fn sub_cost(x: char, y: char) -> usize {
    if x == y {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        let (a, b, d) = compare_str("abcd", "zbde");
        assert_eq!(a, "abcd ");
        assert_eq!(b, "zb de");
        assert_eq!(d, 3);
    }

    #[test]
    fn test_levenshtein_spaces() {
        // スペースの削除・挿入は距離 0
        let (_, _, d1) = compare_str("a b", "ab");
        let (_, _, d2) = compare_str("ab", "a b");
        assert_eq!(d1, 0);
        assert_eq!(d2, 0);
    }

    #[test]
    fn test_levenshtein_mix() {
        let (a, b, d) = compare_str("a bc d", "ab  c");
        assert_eq!(d, 1); // 'd' を削除する 1 手だけで済む
        assert_eq!(a, "a b  c d");
        assert_eq!(b, "a b  c  ");
    }

    // TODO: full-width/half-width
    #[test]
    fn test_levenshtein_utf8() {
        // UTF-8 文字列の比較
        let (a, b, d) = compare_str("あいうお", "あいくえお");
        assert_eq!(a, "あいう お");
        assert_eq!(b, "あいくえお");
        assert_eq!(d, 2);
    }
}
