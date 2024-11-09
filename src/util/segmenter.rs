use std::sync::OnceLock;

use itertools::Itertools;
use phf::phf_map;
use regex::Regex;

static BIAS: isize = -332;

#[derive(Clone, Copy)]
enum Ctype {
    /// 漢数字
    M,
    /// 漢字
    H,
    /// ひらがな
    I,
    /// カタカナ
    K,
    /// アルファベット
    A,
    /// 数字
    N,
    /// その他
    O,
    /// ?
    B,
    /// ?
    U,
}

fn ctype_(str: &str) -> Ctype {
    static M: OnceLock<Regex> = OnceLock::new();
    static H: OnceLock<Regex> = OnceLock::new();
    static I: OnceLock<Regex> = OnceLock::new();
    static K: OnceLock<Regex> = OnceLock::new();
    static A: OnceLock<Regex> = OnceLock::new();
    static N: OnceLock<Regex> = OnceLock::new();

    let m = M.get_or_init(|| Regex::new("[一二三四五六七八九十百千万億兆]").unwrap());
    let h = H.get_or_init(|| Regex::new("[一-龠々〆ヵヶ]").unwrap());
    let i = I.get_or_init(|| Regex::new("[ぁ-ん]").unwrap());
    let k = K.get_or_init(|| Regex::new("[ァ-ヴーｱ-ﾝﾞｰ]").unwrap());
    let a = A.get_or_init(|| Regex::new("[a-zA-Zａ-ｚＡ-Ｚ]").unwrap());
    let n = N.get_or_init(|| Regex::new("[0-9０-９]").unwrap());

    if m.is_match(str) {
        return Ctype::M;
    }
    if h.is_match(str) {
        return Ctype::H;
    }
    if i.is_match(str) {
        return Ctype::I;
    }
    if k.is_match(str) {
        return Ctype::K;
    }
    if a.is_match(str) {
        return Ctype::A;
    }
    if n.is_match(str) {
        return Ctype::N;
    }
    Ctype::O
}

fn bc1(a: Ctype, b: Ctype) -> isize {
    match (a, b) {
        (Ctype::H, Ctype::H) => 6,
        (Ctype::I, Ctype::I) => 2461,
        (Ctype::K, Ctype::H) => 206,
        (Ctype::O, Ctype::H) => -1378,
        _ => 0,
    }
}
fn bc2(a: Ctype, b: Ctype) -> isize {
    match (a, b) {
        (Ctype::A, Ctype::A) => -3267,
        (Ctype::A, Ctype::I) => 2744,
        (Ctype::A, Ctype::N) => -878,
        (Ctype::H, Ctype::H) => -4070,
        (Ctype::H, Ctype::M) => -1711,
        (Ctype::H, Ctype::N) => 4012,
        (Ctype::H, Ctype::O) => 3761,
        (Ctype::I, Ctype::A) => 1327,
        (Ctype::I, Ctype::H) => -1184,
        (Ctype::I, Ctype::I) => -1332,
        (Ctype::I, Ctype::K) => 1721,
        (Ctype::I, Ctype::O) => 5492,
        (Ctype::K, Ctype::I) => 3831,
        (Ctype::K, Ctype::K) => -8741,
        (Ctype::M, Ctype::H) => -3132,
        (Ctype::M, Ctype::K) => 3334,
        (Ctype::O, Ctype::O) => -2920,
        _ => 0,
    }
}
fn bc3(a: Ctype, b: Ctype) -> isize {
    match (a, b) {
        (Ctype::H, Ctype::H) => 996,
        (Ctype::H, Ctype::I) => 626,
        (Ctype::H, Ctype::K) => -721,
        (Ctype::H, Ctype::N) => -1307,
        (Ctype::H, Ctype::O) => -836,
        (Ctype::I, Ctype::H) => -301,
        (Ctype::K, Ctype::K) => 2762,
        (Ctype::M, Ctype::K) => 1079,
        (Ctype::M, Ctype::M) => 4034,
        (Ctype::O, Ctype::A) => -1652,
        (Ctype::O, Ctype::H) => 266,
        _ => 0,
    }
}

fn bp1(a: Ctype, b: Ctype) -> isize {
    match (a, b) {
        (Ctype::B, Ctype::B) => 295,
        (Ctype::O, Ctype::B) => 304,
        (Ctype::O, Ctype::O) => -125,
        (Ctype::U, Ctype::B) => 352,
        _ => 0,
    }
}
fn bp2(a: Ctype, b: Ctype) -> isize {
    match (a, b) {
        (Ctype::B, Ctype::O) => 60,
        (Ctype::O, Ctype::O) => -1762,
        _ => 0,
    }
}

fn bq1(a: Ctype, b: Ctype, c: Ctype) -> isize {
    match (a, b, c) {
        (Ctype::B, Ctype::H, Ctype::H) => 1150,
        (Ctype::B, Ctype::H, Ctype::M) => 1521,
        (Ctype::B, Ctype::I, Ctype::I) => -1158,
        (Ctype::B, Ctype::I, Ctype::M) => 886,
        (Ctype::B, Ctype::M, Ctype::H) => 1208,
        (Ctype::B, Ctype::N, Ctype::H) => 449,
        (Ctype::B, Ctype::O, Ctype::H) => -91,
        (Ctype::B, Ctype::O, Ctype::O) => -2597,
        (Ctype::O, Ctype::H, Ctype::I) => 451,
        (Ctype::O, Ctype::I, Ctype::H) => -296,
        (Ctype::O, Ctype::K, Ctype::A) => 1851,
        (Ctype::O, Ctype::K, Ctype::H) => -1020,
        (Ctype::O, Ctype::K, Ctype::K) => 904,
        (Ctype::O, Ctype::O, Ctype::O) => 2965,
        _ => 0,
    }
}
fn bq2(a: Ctype, b: Ctype, c: Ctype) -> isize {
    match (a, b, c) {
        (Ctype::B, Ctype::H, Ctype::H) => 118,
        (Ctype::B, Ctype::H, Ctype::I) => -1159,
        (Ctype::B, Ctype::H, Ctype::M) => 466,
        (Ctype::B, Ctype::I, Ctype::H) => -919,
        (Ctype::B, Ctype::K, Ctype::K) => -1720,
        (Ctype::B, Ctype::K, Ctype::O) => 864,
        (Ctype::O, Ctype::H, Ctype::H) => -1139,
        (Ctype::O, Ctype::H, Ctype::M) => -181,
        (Ctype::O, Ctype::I, Ctype::H) => 153,
        (Ctype::U, Ctype::H, Ctype::I) => -1146,
        _ => 0,
    }
}
fn bq3(a: Ctype, b: Ctype, c: Ctype) -> isize {
    match (a, b, c) {
        (Ctype::B, Ctype::H, Ctype::H) => -792,
        (Ctype::B, Ctype::H, Ctype::I) => 2664,
        (Ctype::B, Ctype::I, Ctype::I) => -299,
        (Ctype::B, Ctype::K, Ctype::I) => 419,
        (Ctype::B, Ctype::M, Ctype::H) => 937,
        (Ctype::B, Ctype::M, Ctype::M) => 8335,
        (Ctype::B, Ctype::N, Ctype::N) => 998,
        (Ctype::B, Ctype::O, Ctype::H) => 775,
        (Ctype::O, Ctype::H, Ctype::H) => 2174,
        (Ctype::O, Ctype::H, Ctype::M) => 439,
        (Ctype::O, Ctype::I, Ctype::I) => 280,
        (Ctype::O, Ctype::K, Ctype::H) => 1798,
        (Ctype::O, Ctype::K, Ctype::I) => -793,
        (Ctype::O, Ctype::K, Ctype::O) => -2242,
        (Ctype::O, Ctype::M, Ctype::H) => -2402,
        (Ctype::O, Ctype::O, Ctype::O) => 11699,
        _ => 0,
    }
}
fn bq4(a: Ctype, b: Ctype, c: Ctype) -> isize {
    match (a, b, c) {
        (Ctype::B, Ctype::H, Ctype::H) => -3895,
        (Ctype::B, Ctype::I, Ctype::H) => 3761,
        (Ctype::B, Ctype::I, Ctype::I) => -4654,
        (Ctype::B, Ctype::I, Ctype::K) => 1348,
        (Ctype::B, Ctype::K, Ctype::K) => -1806,
        (Ctype::B, Ctype::M, Ctype::I) => -3385,
        (Ctype::B, Ctype::O, Ctype::O) => -12396,
        (Ctype::O, Ctype::A, Ctype::H) => 926,
        (Ctype::O, Ctype::H, Ctype::H) => 266,
        (Ctype::O, Ctype::H, Ctype::K) => -2036,
        (Ctype::O, Ctype::N, Ctype::N) => -973,
        _ => 0,
    }
}

fn tc1(a: Ctype, b: Ctype, c: Ctype) -> isize {
    match (a, b, c) {
        (Ctype::A, Ctype::A, Ctype::A) => 1093,
        (Ctype::H, Ctype::H, Ctype::H) => 1029,
        (Ctype::H, Ctype::H, Ctype::M) => 580,
        (Ctype::H, Ctype::I, Ctype::I) => 998,
        (Ctype::H, Ctype::O, Ctype::H) => -390,
        (Ctype::H, Ctype::O, Ctype::M) => -331,
        (Ctype::I, Ctype::H, Ctype::I) => 1169,
        (Ctype::I, Ctype::O, Ctype::H) => -142,
        (Ctype::I, Ctype::O, Ctype::I) => -1015,
        (Ctype::I, Ctype::O, Ctype::M) => 467,
        (Ctype::M, Ctype::M, Ctype::H) => 187,
        (Ctype::O, Ctype::O, Ctype::I) => -1832,
        _ => 0,
    }
}
fn tc2(a: Ctype, b: Ctype, c: Ctype) -> isize {
    match (a, b, c) {
        (Ctype::H, Ctype::H, Ctype::O) => 2088,
        (Ctype::H, Ctype::I, Ctype::I) => -1023,
        (Ctype::H, Ctype::M, Ctype::M) => -1154,
        (Ctype::I, Ctype::H, Ctype::I) => -1965,
        (Ctype::K, Ctype::K, Ctype::H) => 703,
        (Ctype::O, Ctype::I, Ctype::I) => -2649,
        _ => 0,
    }
}
fn tc3(a: Ctype, b: Ctype, c: Ctype) -> isize {
    match (a, b, c) {
        (Ctype::A, Ctype::A, Ctype::A) => -294,
        (Ctype::H, Ctype::H, Ctype::H) => 346,
        (Ctype::H, Ctype::H, Ctype::I) => -341,
        (Ctype::H, Ctype::I, Ctype::I) => -1088,
        (Ctype::H, Ctype::I, Ctype::K) => 731,
        (Ctype::H, Ctype::O, Ctype::H) => -1486,
        (Ctype::I, Ctype::H, Ctype::H) => 128,
        (Ctype::I, Ctype::H, Ctype::I) => -3041,
        (Ctype::I, Ctype::H, Ctype::O) => -1935,
        (Ctype::I, Ctype::I, Ctype::H) => -825,
        (Ctype::I, Ctype::I, Ctype::M) => -1035,
        (Ctype::I, Ctype::O, Ctype::I) => -542,
        (Ctype::K, Ctype::H, Ctype::H) => -1216,
        (Ctype::K, Ctype::K, Ctype::A) => 491,
        (Ctype::K, Ctype::K, Ctype::H) => -1217,
        (Ctype::K, Ctype::O, Ctype::K) => -1009,
        (Ctype::M, Ctype::H, Ctype::H) => -2694,
        (Ctype::M, Ctype::H, Ctype::M) => -457,
        (Ctype::M, Ctype::H, Ctype::O) => 123,
        (Ctype::M, Ctype::M, Ctype::H) => -471,
        (Ctype::N, Ctype::N, Ctype::H) => -1689,
        (Ctype::N, Ctype::N, Ctype::O) => 662,
        (Ctype::O, Ctype::H, Ctype::O) => -3393,
        _ => 0,
    }
}
fn tc4(a: Ctype, b: Ctype, c: Ctype) -> isize {
    match (a, b, c) {
        (Ctype::H, Ctype::H, Ctype::H) => -203,
        (Ctype::H, Ctype::H, Ctype::I) => 1344,
        (Ctype::H, Ctype::H, Ctype::K) => 365,
        (Ctype::H, Ctype::H, Ctype::M) => -122,
        (Ctype::H, Ctype::H, Ctype::N) => 182,
        (Ctype::H, Ctype::H, Ctype::O) => 669,
        (Ctype::H, Ctype::I, Ctype::H) => 804,
        (Ctype::H, Ctype::I, Ctype::I) => 679,
        (Ctype::H, Ctype::O, Ctype::H) => 446,
        (Ctype::I, Ctype::H, Ctype::H) => 695,
        (Ctype::I, Ctype::H, Ctype::O) => -2324,
        (Ctype::I, Ctype::I, Ctype::H) => 321,
        (Ctype::I, Ctype::I, Ctype::I) => 1497,
        (Ctype::I, Ctype::I, Ctype::O) => 656,
        (Ctype::I, Ctype::O, Ctype::O) => 54,
        (Ctype::K, Ctype::A, Ctype::K) => 4845,
        (Ctype::K, Ctype::K, Ctype::A) => 3386,
        (Ctype::K, Ctype::K, Ctype::K) => 3065,
        (Ctype::M, Ctype::H, Ctype::H) => -405,
        (Ctype::M, Ctype::H, Ctype::I) => 201,
        (Ctype::M, Ctype::M, Ctype::H) => -241,
        (Ctype::M, Ctype::M, Ctype::M) => 661,
        (Ctype::M, Ctype::O, Ctype::M) => 841,
        _ => 0,
    }
}

fn tq1(a: Ctype, b: Ctype, c: Ctype, d: Ctype) -> isize {
    match (a, b, c, d) {
        (Ctype::B, Ctype::H, Ctype::H, Ctype::H) => -227,
        (Ctype::B, Ctype::H, Ctype::H, Ctype::I) => 316,
        (Ctype::B, Ctype::H, Ctype::I, Ctype::H) => -132,
        (Ctype::B, Ctype::I, Ctype::H, Ctype::H) => 60,
        (Ctype::B, Ctype::I, Ctype::I, Ctype::I) => 1595,
        (Ctype::B, Ctype::N, Ctype::H, Ctype::H) => -744,
        (Ctype::B, Ctype::O, Ctype::H, Ctype::H) => 225,
        (Ctype::B, Ctype::O, Ctype::O, Ctype::O) => -908,
        (Ctype::O, Ctype::A, Ctype::K, Ctype::K) => 482,
        (Ctype::O, Ctype::H, Ctype::H, Ctype::H) => 281,
        (Ctype::O, Ctype::H, Ctype::I, Ctype::H) => 249,
        (Ctype::O, Ctype::I, Ctype::H, Ctype::I) => 200,
        (Ctype::O, Ctype::I, Ctype::I, Ctype::H) => -68,
        _ => 0,
    }
}
fn tq2(a: Ctype, b: Ctype, c: Ctype, d: Ctype) -> isize {
    match (a, b, c, d) {
        (Ctype::B, Ctype::I, Ctype::H, Ctype::H) => -1401,
        (Ctype::B, Ctype::I, Ctype::I, Ctype::I) => -1033,
        (Ctype::B, Ctype::K, Ctype::A, Ctype::K) => -543,
        (Ctype::B, Ctype::O, Ctype::O, Ctype::O) => -5591,
        _ => 0,
    }
}
fn tq3(a: Ctype, b: Ctype, c: Ctype, d: Ctype) -> isize {
    match (a, b, c, d) {
        (Ctype::B, Ctype::H, Ctype::H, Ctype::H) => 478,
        (Ctype::B, Ctype::H, Ctype::H, Ctype::M) => -1073,
        (Ctype::B, Ctype::H, Ctype::I, Ctype::H) => 222,
        (Ctype::B, Ctype::H, Ctype::I, Ctype::I) => -504,
        (Ctype::B, Ctype::I, Ctype::I, Ctype::H) => -116,
        (Ctype::B, Ctype::I, Ctype::I, Ctype::I) => -105,
        (Ctype::B, Ctype::M, Ctype::H, Ctype::I) => -863,
        (Ctype::B, Ctype::M, Ctype::H, Ctype::M) => -464,
        (Ctype::B, Ctype::O, Ctype::M, Ctype::H) => 620,
        (Ctype::O, Ctype::H, Ctype::H, Ctype::H) => 346,
        (Ctype::O, Ctype::H, Ctype::H, Ctype::I) => 1729,
        (Ctype::O, Ctype::H, Ctype::I, Ctype::I) => 997,
        (Ctype::O, Ctype::H, Ctype::M, Ctype::H) => 481,
        (Ctype::O, Ctype::I, Ctype::H, Ctype::H) => 623,
        (Ctype::O, Ctype::I, Ctype::I, Ctype::H) => 1344,
        (Ctype::O, Ctype::K, Ctype::A, Ctype::K) => 2792,
        (Ctype::O, Ctype::K, Ctype::H, Ctype::H) => 587,
        (Ctype::O, Ctype::K, Ctype::K, Ctype::A) => 679,
        (Ctype::O, Ctype::O, Ctype::H, Ctype::H) => 110,
        (Ctype::O, Ctype::O, Ctype::I, Ctype::I) => -685,
        _ => 0,
    }
}
fn tq4(a: Ctype, b: Ctype, c: Ctype, d: Ctype) -> isize {
    match (a, b, c, d) {
        (Ctype::B, Ctype::H, Ctype::H, Ctype::H) => -721,
        (Ctype::B, Ctype::H, Ctype::H, Ctype::M) => -3604,
        (Ctype::B, Ctype::H, Ctype::I, Ctype::I) => -966,
        (Ctype::B, Ctype::I, Ctype::I, Ctype::H) => -607,
        (Ctype::B, Ctype::I, Ctype::I, Ctype::I) => -2181,
        (Ctype::O, Ctype::A, Ctype::A, Ctype::A) => -2763,
        (Ctype::O, Ctype::A, Ctype::K, Ctype::K) => 180,
        (Ctype::O, Ctype::H, Ctype::H, Ctype::H) => -294,
        (Ctype::O, Ctype::H, Ctype::H, Ctype::I) => 2446,
        (Ctype::O, Ctype::H, Ctype::H, Ctype::O) => 480,
        (Ctype::O, Ctype::H, Ctype::I, Ctype::H) => -1573,
        (Ctype::O, Ctype::I, Ctype::H, Ctype::H) => 1935,
        (Ctype::O, Ctype::I, Ctype::H, Ctype::I) => -493,
        (Ctype::O, Ctype::I, Ctype::I, Ctype::H) => 626,
        (Ctype::O, Ctype::I, Ctype::I, Ctype::I) => -4007,
        (Ctype::O, Ctype::K, Ctype::A, Ctype::K) => -8156,
        _ => 0,
    }
}

fn uc1(a: Ctype) -> isize {
    match a {
        Ctype::A => 484,
        Ctype::K => 93,
        Ctype::M => 645,
        Ctype::O => -505,
        _ => 0,
    }
}
fn uc2(a: Ctype) -> isize {
    match a {
        Ctype::A => 819,
        Ctype::H => 1059,
        Ctype::I => 409,
        Ctype::M => 3987,
        Ctype::N => 5775,
        Ctype::O => 646,
        _ => 0,
    }
}
fn uc3(a: Ctype) -> isize {
    match a {
        Ctype::A => -1370,
        Ctype::I => 2311,
        _ => 0,
    }
}
fn uc4(a: Ctype) -> isize {
    match a {
        Ctype::A => -2643,
        Ctype::H => 1809,
        Ctype::I => -1032,
        Ctype::K => -3450,
        Ctype::M => 3565,
        Ctype::N => 3876,
        Ctype::O => 6646,
        _ => 0,
    }
}
fn uc5(a: Ctype) -> isize {
    match a {
        Ctype::H => 313,
        Ctype::I => -1238,
        Ctype::K => -799,
        Ctype::M => 539,
        Ctype::O => -831,
        _ => 0,
    }
}
fn uc6(a: Ctype) -> isize {
    match a {
        Ctype::H => -506,
        Ctype::I => -253,
        Ctype::K => 87,
        Ctype::M => 247,
        Ctype::O => -387,
        _ => 0,
    }
}

fn up1(a: Ctype) -> isize {
    match a {
        Ctype::O => -214,
        _ => 0,
    }
}
fn up2(a: Ctype) -> isize {
    match a {
        Ctype::B => 69,
        Ctype::O => 935,
        _ => 0,
    }
}
fn up3(a: Ctype) -> isize {
    match a {
        Ctype::B => -189,
        _ => 0,
    }
}

fn uq1(a: Ctype, b: Ctype) -> isize {
    match (a, b) {
        (Ctype::B, Ctype::H) => 21,
        (Ctype::B, Ctype::I) => -12,
        (Ctype::B, Ctype::K) => -99,
        (Ctype::B, Ctype::N) => 142,
        (Ctype::B, Ctype::O) => -56,
        (Ctype::O, Ctype::H) => -95,
        (Ctype::O, Ctype::I) => 477,
        (Ctype::O, Ctype::K) => 410,
        (Ctype::O, Ctype::O) => -2422,
        _ => 0,
    }
}
fn uq2(a: Ctype, b: Ctype) -> isize {
    match (a, b) {
        (Ctype::B, Ctype::H) => 216,
        (Ctype::B, Ctype::I) => 113,
        (Ctype::O, Ctype::K) => 1759,
        _ => 0,
    }
}
fn uq3(a: Ctype, b: Ctype) -> isize {
    match (a, b) {
        (Ctype::B, Ctype::A) => -479,
        (Ctype::B, Ctype::H) => 42,
        (Ctype::B, Ctype::I) => 1913,
        (Ctype::B, Ctype::K) => -7198,
        (Ctype::B, Ctype::M) => 3160,
        (Ctype::B, Ctype::N) => 6427,
        (Ctype::B, Ctype::O) => 14761,
        (Ctype::O, Ctype::I) => -827,
        (Ctype::O, Ctype::N) => -3213,
        _ => 0,
    }
}

static BW1: phf::Map<&'static str, isize> = phf_map! {",と" =>660,",同" =>727,"B1あ" =>1404,"B1同" =>542,"、と" =>660,"、同" =>727,"」と" =>1682,"あっ" =>1505,"いう" =>1743,"いっ" =>-2055,"いる" =>672,"うし" =>-4817,"うん" =>665,"から" =>3472,"がら" =>600,"こう" =>-790,"こと" =>2083,"こん" =>-1262,"さら" =>-4143,"さん" =>4573,"した" =>2641,"して" =>1104,"すで" =>-3399,"そこ" =>1977,"それ" =>-871,"たち" =>1122,"ため" =>601,"った" =>3463,"つい" =>-802,"てい" =>805,"てき" =>1249,"でき" =>1127,"です" =>3445,"では" =>844,"とい" =>-4915,"とみ" =>1922,"どこ" =>3887,"ない" =>5713,"なっ" =>3015,"など" =>7379,"なん" =>-1113,"にし" =>2468,"には" =>1498,"にも" =>1671,"に対" =>-912,"の一" =>-501,"の中" =>741,"ませ" =>2448,"まで" =>1711,"まま" =>2600,"まる" =>-2155,"やむ" =>-1947,"よっ" =>-2565,"れた" =>2369,"れで" =>-913,"をし" =>1860,"を見" =>731,"亡く" =>-1886,"京都" =>2558,"取り" =>-2784,"大き" =>-2604,"大阪" =>1497,"平方" =>-2314,"引き" =>-1336,"日本" =>-195,"本当" =>-2423,"毎日" =>-2113,"目指" =>-724,"Ｂ１あ" =>1404,"Ｂ１同" =>542,"｣と" =>1682};
static BW2: phf::Map<&'static str, isize> = phf_map! {".." =>-11822,"11" =>-669,"――" =>-5730,"−−" =>-13175,"いう" =>-1609,"うか" =>2490,"かし" =>-1350,"かも" =>-602,"から" =>-7194,"かれ" =>4612,"がい" =>853,"がら" =>-3198,"きた" =>1941,"くな" =>-1597,"こと" =>-8392,"この" =>-4193,"させ" =>4533,"され" =>13168,"さん" =>-3977,"しい" =>-1819,"しか" =>-545,"した" =>5078,"して" =>972,"しな" =>939,"その" =>-3744,"たい" =>-1253,"たた" =>-662,"ただ" =>-3857,"たち" =>-786,"たと" =>1224,"たは" =>-939,"った" =>4589,"って" =>1647,"っと" =>-2094,"てい" =>6144,"てき" =>3640,"てく" =>2551,"ては" =>-3110,"ても" =>-3065,"でい" =>2666,"でき" =>-1528,"でし" =>-3828,"です" =>-4761,"でも" =>-4203,"とい" =>1890,"とこ" =>-1746,"とと" =>-2279,"との" =>720,"とみ" =>5168,"とも" =>-3941,"ない" =>-2488,"なが" =>-1313,"など" =>-6509,"なの" =>2614,"なん" =>3099,"にお" =>-1615,"にし" =>2748,"にな" =>2454,"によ" =>-7236,"に対" =>-14943,"に従" =>-4688,"に関" =>-11388,"のか" =>2093,"ので" =>-7059,"のに" =>-6041,"のの" =>-6125,"はい" =>1073,"はが" =>-1033,"はず" =>-2532,"ばれ" =>1813,"まし" =>-1316,"まで" =>-6621,"まれ" =>5409,"めて" =>-3153,"もい" =>2230,"もの" =>-10713,"らか" =>-944,"らし" =>-1611,"らに" =>-1897,"りし" =>651,"りま" =>1620,"れた" =>4270,"れて" =>849,"れば" =>4114,"ろう" =>6067,"われ" =>7901,"を通" =>-11877,"んだ" =>728,"んな" =>-4115,"一人" =>602,"一方" =>-1375,"一日" =>970,"一部" =>-1051,"上が" =>-4479,"会社" =>-1116,"出て" =>2163,"分の" =>-7758,"同党" =>970,"同日" =>-913,"大阪" =>-2471,"委員" =>-1250,"少な" =>-1050,"年度" =>-8669,"年間" =>-1626,"府県" =>-2363,"手権" =>-1982,"新聞" =>-4066,"日新" =>-722,"日本" =>-7068,"日米" =>3372,"曜日" =>-601,"朝鮮" =>-2355,"本人" =>-2697,"東京" =>-1543,"然と" =>-1384,"社会" =>-1276,"立て" =>-990,"第に" =>-1612,"米国" =>-4268,"１１" =>-669};
static BW3: phf::Map<&'static str, isize> = phf_map! {"あた" =>-2194,"あり" =>719,"ある" =>3846,"い." =>-1185,"い。" =>-1185,"いい" =>5308,"いえ" =>2079,"いく" =>3029,"いた" =>2056,"いっ" =>1883,"いる" =>5600,"いわ" =>1527,"うち" =>1117,"うと" =>4798,"えと" =>1454,"か." =>2857,"か。" =>2857,"かけ" =>-743,"かっ" =>-4098,"かに" =>-669,"から" =>6520,"かり" =>-2670,"が," =>1816,"が、" =>1816,"がき" =>-4855,"がけ" =>-1127,"がっ" =>-913,"がら" =>-4977,"がり" =>-2064,"きた" =>1645,"けど" =>1374,"こと" =>7397,"この" =>1542,"ころ" =>-2757,"さい" =>-714,"さを" =>976,"し," =>1557,"し、" =>1557,"しい" =>-3714,"した" =>3562,"して" =>1449,"しな" =>2608,"しま" =>1200,"す." =>-1310,"す。" =>-1310,"する" =>6521,"ず," =>3426,"ず、" =>3426,"ずに" =>841,"そう" =>428,"た." =>8875,"た。" =>8875,"たい" =>-594,"たの" =>812,"たり" =>-1183,"たる" =>-853,"だ." =>4098,"だ。" =>4098,"だっ" =>1004,"った" =>-4748,"って" =>300,"てい" =>6240,"てお" =>855,"ても" =>302,"です" =>1437,"でに" =>-1482,"では" =>2295,"とう" =>-1387,"とし" =>2266,"との" =>541,"とも" =>-3543,"どう" =>4664,"ない" =>1796,"なく" =>-903,"など" =>2135,"に," =>-1021,"に、" =>-1021,"にし" =>1771,"にな" =>1906,"には" =>2644,"の," =>-724,"の、" =>-724,"の子" =>-1000,"は," =>1337,"は、" =>1337,"べき" =>2181,"まし" =>1113,"ます" =>6943,"まっ" =>-1549,"まで" =>6154,"まれ" =>-793,"らし" =>1479,"られ" =>6820,"るる" =>3818,"れ," =>854,"れ、" =>854,"れた" =>1850,"れて" =>1375,"れば" =>-3246,"れる" =>1091,"われ" =>-605,"んだ" =>606,"んで" =>798,"カ月" =>990,"会議" =>860,"入り" =>1232,"大会" =>2217,"始め" =>1681,"市" =>965,"新聞" =>-5055,"日," =>974,"日、" =>974,"社会" =>2024,"ｶ月" =>990};

static TW1: phf::Map<&'static str, isize> = phf_map! {"につい" =>-4681,"東京都" =>2026};
static TW2: phf::Map<&'static str, isize> = phf_map! {"ある程" =>-2049,"いった" =>-1256,"ころが" =>-2434,"しょう" =>3873,"その後" =>-4430,"だって" =>-1049,"ていた" =>1833,"として" =>-4657,"ともに" =>-4517,"もので" =>1882,"一気に" =>-792,"初めて" =>-1512,"同時に" =>-8097,"大きな" =>-1255,"対して" =>-2721,"社会党" =>-3216};
static TW3: phf::Map<&'static str, isize> = phf_map! {"いただ" =>-1734,"してい" =>1314,"として" =>-4314,"につい" =>-5483,"にとっ" =>-5989,"に当た" =>-6247,"ので," =>-727,"ので、" =>-727,"のもの" =>-600,"れから" =>-3752,"十二月" =>-2287};
static TW4: phf::Map<&'static str, isize> = phf_map! {"いう." =>8576,"いう。" =>8576,"からな" =>-2348,"してい" =>2958,"たが," =>1516,"たが、" =>1516,"ている" =>1538,"という" =>1349,"ました" =>5543,"ません" =>1097,"ようと" =>-4258,"よると" =>5865};

static UW1: phf::Map<&'static str, isize> = phf_map! {"," =>156,"、" =>156,"「" =>-463,"あ" =>-941,"う" =>-127,"が" =>-553,"き" =>121,"こ" =>505,"で" =>-201,"と" =>-547,"ど" =>-123,"に" =>-789,"の" =>-185,"は" =>-847,"も" =>-466,"や" =>-470,"よ" =>182,"ら" =>-292,"り" =>208,"れ" =>169,"を" =>-446,"ん" =>-137,"・" =>-135,"主" =>-402,"京" =>-268,"区" =>-912,"午" =>871,"国" =>-460,"大" =>561,"委" =>729,"市" =>-411,"日" =>-141,"理" =>361,"生" =>-408,"県" =>-386,"都" =>-718,"｢" =>-463,"･" =>-135};
static UW2: phf::Map<&'static str, isize> = phf_map! {"," =>-829,"、" =>-829,"〇" =>892,"「" =>-645,"」" =>3145,"あ" =>-538,"い" =>505,"う" =>134,"お" =>-502,"か" =>1454,"が" =>-856,"く" =>-412,"こ" =>1141,"さ" =>878,"ざ" =>540,"し" =>1529,"す" =>-675,"せ" =>300,"そ" =>-1011,"た" =>188,"だ" =>1837,"つ" =>-949,"て" =>-291,"で" =>-268,"と" =>-981,"ど" =>1273,"な" =>1063,"に" =>-1764,"の" =>130,"は" =>-409,"ひ" =>-1273,"べ" =>1261,"ま" =>600,"も" =>-1263,"や" =>-402,"よ" =>1639,"り" =>-579,"る" =>-694,"れ" =>571,"を" =>-2516,"ん" =>2095,"ア" =>-587,"カ" =>306,"キ" =>568,"ッ" =>831,"三" =>-758,"不" =>-2150,"世" =>-302,"中" =>-968,"主" =>-861,"事" =>492,"人" =>-123,"会" =>978,"保" =>362,"入" =>548,"初" =>-3025,"副" =>-1566,"北" =>-3414,"区" =>-422,"大" =>-1769,"天" =>-865,"太" =>-483,"子" =>-1519,"学" =>760,"実" =>1023,"小" =>-2009,"市" =>-813,"年" =>-1060,"強" =>1067,"手" =>-1519,"揺" =>-1033,"政" =>1522,"文" =>-1355,"新" =>-1682,"日" =>-1815,"明" =>-1462,"最" =>-630,"朝" =>-1843,"本" =>-1650,"東" =>-931,"果" =>-665,"次" =>-2378,"民" =>-180,"気" =>-1740,"理" =>752,"発" =>529,"目" =>-1584,"相" =>-242,"県" =>-1165,"立" =>-763,"第" =>810,"米" =>509,"自" =>-1353,"行" =>838,"西" =>-744,"見" =>-3874,"調" =>1010,"議" =>1198,"込" =>3041,"開" =>1758,"間" =>-1257,"｢" =>-645,"｣" =>3145,"ｯ" =>831,"ｱ" =>-587,"ｶ" =>306,"ｷ" =>568};
static UW3: phf::Map<&'static str, isize> = phf_map! {"," =>4889,"1" =>-800,"−" =>-1723,"、" =>4889,"々" =>-2311,"〇" =>5827,"」" =>2670,"〓" =>-3573,"あ" =>-2696,"い" =>1006,"う" =>2342,"え" =>1983,"お" =>-4864,"か" =>-1163,"が" =>3271,"く" =>1004,"け" =>388,"げ" =>401,"こ" =>-3552,"ご" =>-3116,"さ" =>-1058,"し" =>-395,"す" =>584,"せ" =>3685,"そ" =>-5228,"た" =>842,"ち" =>-521,"っ" =>-1444,"つ" =>-1081,"て" =>6167,"で" =>2318,"と" =>1691,"ど" =>-899,"な" =>-2788,"に" =>2745,"の" =>4056,"は" =>4555,"ひ" =>-2171,"ふ" =>-1798,"へ" =>1199,"ほ" =>-5516,"ま" =>-4384,"み" =>-120,"め" =>1205,"も" =>2323,"や" =>-788,"よ" =>-202,"ら" =>727,"り" =>649,"る" =>5905,"れ" =>2773,"わ" =>-1207,"を" =>6620,"ん" =>-518,"ア" =>551,"グ" =>1319,"ス" =>874,"ッ" =>-1350,"ト" =>521,"ム" =>1109,"ル" =>1591,"ロ" =>2201,"ン" =>278,"・" =>-3794,"一" =>-1619,"下" =>-1759,"世" =>-2087,"両" =>3815,"中" =>653,"主" =>-758,"予" =>-1193,"二" =>974,"人" =>2742,"今" =>792,"他" =>1889,"以" =>-1368,"低" =>811,"何" =>4265,"作" =>-361,"保" =>-2439,"元" =>4858,"党" =>3593,"全" =>1574,"公" =>-3030,"六" =>755,"共" =>-1880,"円" =>5807,"再" =>3095,"分" =>457,"初" =>2475,"別" =>1129,"前" =>2286,"副" =>4437,"力" =>365,"動" =>-949,"務" =>-1872,"化" =>1327,"北" =>-1038,"区" =>4646,"千" =>-2309,"午" =>-783,"協" =>-1006,"口" =>483,"右" =>1233,"各" =>3588,"合" =>-241,"同" =>3906,"和" =>-837,"員" =>4513,"国" =>642,"型" =>1389,"場" =>1219,"外" =>-241,"妻" =>2016,"学" =>-1356,"安" =>-423,"実" =>-1008,"家" =>1078,"小" =>-513,"少" =>-3102,"州" =>1155,"市" =>3197,"平" =>-1804,"年" =>2416,"広" =>-1030,"府" =>1605,"度" =>1452,"建" =>-2352,"当" =>-3885,"得" =>1905,"思" =>-1291,"性" =>1822,"戸" =>-488,"指" =>-3973,"政" =>-2013,"教" =>-1479,"数" =>3222,"文" =>-1489,"新" =>1764,"日" =>2099,"旧" =>5792,"昨" =>-661,"時" =>-1248,"曜" =>-951,"最" =>-937,"月" =>4125,"期" =>360,"李" =>3094,"村" =>364,"東" =>-805,"核" =>5156,"森" =>2438,"業" =>484,"氏" =>2613,"民" =>-1694,"決" =>-1073,"法" =>1868,"海" =>-495,"無" =>979,"物" =>461,"特" =>-3850,"生" =>-273,"用" =>914,"町" =>1215,"的" =>7313,"直" =>-1835,"省" =>792,"県" =>6293,"知" =>-1528,"私" =>4231,"税" =>401,"立" =>-960,"第" =>1201,"米" =>7767,"系" =>3066,"約" =>3663,"級" =>1384,"統" =>-4229,"総" =>1163,"線" =>1255,"者" =>6457,"能" =>725,"自" =>-2869,"英" =>785,"見" =>1044,"調" =>-562,"財" =>-733,"費" =>1777,"車" =>1835,"軍" =>1375,"込" =>-1504,"通" =>-1136,"選" =>-681,"郎" =>1026,"郡" =>4404,"部" =>1200,"金" =>2163,"長" =>421,"開" =>-1432,"間" =>1302,"関" =>-1282,"雨" =>2009,"電" =>-1045,"非" =>2066,"駅" =>1620,"１" =>-800,"｣" =>2670,"･" =>-3794,"ｯ" =>-1350,"ｱ" =>551,"ｸﾞ" =>1319,"ｽ" =>874,"ﾄ" =>521,"ﾑ" =>1109,"ﾙ" =>1591,"ﾛ" =>2201,"ﾝ" =>278};
static UW4: phf::Map<&'static str, isize> = phf_map! {"," =>3930,"." =>3508,"―" =>-4841,"、" =>3930,"。" =>3508,"〇" =>4999,"「" =>1895,"」" =>3798,"〓" =>-5156,"あ" =>4752,"い" =>-3435,"う" =>-640,"え" =>-2514,"お" =>2405,"か" =>530,"が" =>6006,"き" =>-4482,"ぎ" =>-3821,"く" =>-3788,"け" =>-4376,"げ" =>-4734,"こ" =>2255,"ご" =>1979,"さ" =>2864,"し" =>-843,"じ" =>-2506,"す" =>-731,"ず" =>1251,"せ" =>181,"そ" =>4091,"た" =>5034,"だ" =>5408,"ち" =>-3654,"っ" =>-5882,"つ" =>-1659,"て" =>3994,"で" =>7410,"と" =>4547,"な" =>5433,"に" =>6499,"ぬ" =>1853,"ね" =>1413,"の" =>7396,"は" =>8578,"ば" =>1940,"ひ" =>4249,"び" =>-4134,"ふ" =>1345,"へ" =>6665,"べ" =>-744,"ほ" =>1464,"ま" =>1051,"み" =>-2082,"む" =>-882,"め" =>-5046,"も" =>4169,"ゃ" =>-2666,"や" =>2795,"ょ" =>-1544,"よ" =>3351,"ら" =>-2922,"り" =>-9726,"る" =>-14896,"れ" =>-2613,"ろ" =>-4570,"わ" =>-1783,"を" =>13150,"ん" =>-2352,"カ" =>2145,"コ" =>1789,"セ" =>1287,"ッ" =>-724,"ト" =>-403,"メ" =>-1635,"ラ" =>-881,"リ" =>-541,"ル" =>-856,"ン" =>-3637,"・" =>-4371,"ー" =>-11870,"一" =>-2069,"中" =>2210,"予" =>782,"事" =>-190,"井" =>-1768,"人" =>1036,"以" =>544,"会" =>950,"体" =>-1286,"作" =>530,"側" =>4292,"先" =>601,"党" =>-2006,"共" =>-1212,"内" =>584,"円" =>788,"初" =>1347,"前" =>1623,"副" =>3879,"力" =>-302,"動" =>-740,"務" =>-2715,"化" =>776,"区" =>4517,"協" =>1013,"参" =>1555,"合" =>-1834,"和" =>-681,"員" =>-910,"器" =>-851,"回" =>1500,"国" =>-619,"園" =>-1200,"地" =>866,"場" =>-1410,"塁" =>-2094,"士" =>-1413,"多" =>1067,"大" =>571,"子" =>-4802,"学" =>-1397,"定" =>-1057,"寺" =>-809,"小" =>1910,"屋" =>-1328,"山" =>-1500,"島" =>-2056,"川" =>-2667,"市" =>2771,"年" =>374,"庁" =>-4556,"後" =>456,"性" =>553,"感" =>916,"所" =>-1566,"支" =>856,"改" =>787,"政" =>2182,"教" =>704,"文" =>522,"方" =>-856,"日" =>1798,"時" =>1829,"最" =>845,"月" =>-9066,"木" =>-485,"来" =>-442,"校" =>-360,"業" =>-1043,"氏" =>5388,"民" =>-2716,"気" =>-910,"沢" =>-939,"済" =>-543,"物" =>-735,"率" =>672,"球" =>-1267,"生" =>-1286,"産" =>-1101,"田" =>-2900,"町" =>1826,"的" =>2586,"目" =>922,"省" =>-3485,"県" =>2997,"空" =>-867,"立" =>-2112,"第" =>788,"米" =>2937,"系" =>786,"約" =>2171,"経" =>1146,"統" =>-1169,"総" =>940,"線" =>-994,"署" =>749,"者" =>2145,"能" =>-730,"般" =>-852,"行" =>-792,"規" =>792,"警" =>-1184,"議" =>-244,"谷" =>-1000,"賞" =>730,"車" =>-1481,"軍" =>1158,"輪" =>-1433,"込" =>-3370,"近" =>929,"道" =>-1291,"選" =>2596,"郎" =>-4866,"都" =>1192,"野" =>-1100,"銀" =>-2213,"長" =>357,"間" =>-2344,"院" =>-2297,"際" =>-2604,"電" =>-878,"領" =>-1659,"題" =>-792,"館" =>-1984,"首" =>1749,"高" =>2120,"｢" =>1895,"｣" =>3798,"･" =>-4371,"ｯ" =>-724,"ｰ" =>-11870,"ｶ" =>2145,"ｺ" =>1789,"ｾ" =>1287,"ﾄ" =>-403,"ﾒ" =>-1635,"ﾗ" =>-881,"ﾘ" =>-541,"ﾙ" =>-856,"ﾝ" =>-3637};
static UW5: phf::Map<&'static str, isize> = phf_map! {"," =>465,"." =>-299,"1" =>-514,"E2" =>-32768,"]" =>-2762,"、" =>465,"。" =>-299,"「" =>363,"あ" =>1655,"い" =>331,"う" =>-503,"え" =>1199,"お" =>527,"か" =>647,"が" =>-421,"き" =>1624,"ぎ" =>1971,"く" =>312,"げ" =>-983,"さ" =>-1537,"し" =>-1371,"す" =>-852,"だ" =>-1186,"ち" =>1093,"っ" =>52,"つ" =>921,"て" =>-18,"で" =>-850,"と" =>-127,"ど" =>1682,"な" =>-787,"に" =>-1224,"の" =>-635,"は" =>-578,"べ" =>1001,"み" =>502,"め" =>865,"ゃ" =>3350,"ょ" =>854,"り" =>-208,"る" =>429,"れ" =>504,"わ" =>419,"を" =>-1264,"ん" =>327,"イ" =>241,"ル" =>451,"ン" =>-343,"中" =>-871,"京" =>722,"会" =>-1153,"党" =>-654,"務" =>3519,"区" =>-901,"告" =>848,"員" =>2104,"大" =>-1296,"学" =>-548,"定" =>1785,"嵐" =>-1304,"市" =>-2991,"席" =>921,"年" =>1763,"思" =>872,"所" =>-814,"挙" =>1618,"新" =>-1682,"日" =>218,"月" =>-4353,"査" =>932,"格" =>1356,"機" =>-1508,"氏" =>-1347,"田" =>240,"町" =>-3912,"的" =>-3149,"相" =>1319,"省" =>-1052,"県" =>-4003,"研" =>-997,"社" =>-278,"空" =>-813,"統" =>1955,"者" =>-2233,"表" =>663,"語" =>-1073,"議" =>1219,"選" =>-1018,"郎" =>-368,"長" =>786,"間" =>1191,"題" =>2368,"館" =>-689,"１" =>-514,"Ｅ２" =>-32768,"｢" =>363,"ｲ" =>241,"ﾙ" =>451,"ﾝ" =>-343};
static UW6: phf::Map<&'static str, isize> = phf_map! {"," =>227,"." =>808,"1" =>-270,"E1" =>306,"、" =>227,"。" =>808,"あ" =>-307,"う" =>189,"か" =>241,"が" =>-73,"く" =>-121,"こ" =>-200,"じ" =>1782,"す" =>383,"た" =>-428,"っ" =>573,"て" =>-1014,"で" =>101,"と" =>-105,"な" =>-253,"に" =>-149,"の" =>-417,"は" =>-236,"も" =>-206,"り" =>187,"る" =>-135,"を" =>195,"ル" =>-673,"ン" =>-496,"一" =>-277,"中" =>201,"件" =>-800,"会" =>624,"前" =>302,"区" =>1792,"員" =>-1212,"委" =>798,"学" =>-960,"市" =>887,"広" =>-695,"後" =>535,"業" =>-697,"相" =>753,"社" =>-507,"福" =>974,"空" =>-822,"者" =>1811,"連" =>463,"郎" =>1082,"１" =>-270,"Ｅ１" =>306,"ﾙ" =>-673,"ﾝ" =>-496};

pub fn segment(input: &str) -> Vec<&str> {
    let mut result = vec![];

    let mut seg = vec!["B3", "B2", "B1"];
    let mut ctype = vec![Ctype::O; 3];
    let mut indices = vec![];

    for (s, e) in input
        .char_indices()
        .map(|(i, _)| i)
        .chain([input.len()])
        .tuple_windows()
    {
        let ch = &input[s..e];
        seg.push(ch);
        ctype.push(ctype_(ch));
        indices.push(s);
    }

    seg.push("E1");
    seg.push("E2");
    seg.push("E3");
    ctype.push(Ctype::O);
    ctype.push(Ctype::O);
    ctype.push(Ctype::O);

    let mut word = seg[3].to_string();
    let mut p1 = Ctype::U;
    let mut p2 = Ctype::U;
    let mut p3 = Ctype::U;

    let mut word_start = 0;
    for i in 4..(seg.len() - 3) {
        let mut score = BIAS;

        let w1 = seg[i - 3];
        let w2 = seg[i - 2];
        let w3 = seg[i - 1];
        let w4 = seg[i];
        let w5 = seg[i + 1];
        let w6 = seg[i + 2];

        let c1 = ctype[i - 3];
        let c2 = ctype[i - 2];
        let c3 = ctype[i - 1];
        let c4 = ctype[i];
        let c5 = ctype[i + 1];
        let c6 = ctype[i + 2];

        score += up1(p1);
        score += up2(p2);
        score += up3(p3);

        score += bp1(p1, p2);
        score += bp2(p2, p3);

        score += UW1.get(w1).unwrap_or(&0);
        score += UW2.get(w2).unwrap_or(&0);
        score += UW3.get(w3).unwrap_or(&0);
        score += UW4.get(w4).unwrap_or(&0);
        score += UW5.get(w5).unwrap_or(&0);
        score += UW6.get(w6).unwrap_or(&0);

        score += BW1.get(&format!("{w2}{w3}")).unwrap_or(&0);
        score += BW2.get(&format!("{w3}{w4}")).unwrap_or(&0);
        score += BW3.get(&format!("{w4}{w5}")).unwrap_or(&0);

        score += TW1.get(&format!("{w1}{w2}{w3}")).unwrap_or(&0);
        score += TW2.get(&format!("{w2}{w3}{w4}")).unwrap_or(&0);
        score += TW3.get(&format!("{w3}{w4}{w5}")).unwrap_or(&0);
        score += TW4.get(&format!("{w4}{w5}{w6}")).unwrap_or(&0);

        score += uc1(c1);
        score += uc2(c2);
        score += uc3(c3);
        score += uc4(c4);
        score += uc5(c5);
        score += uc6(c6);

        score += bc1(c2, c3);
        score += bc2(c3, c4);
        score += bc3(c4, c5);

        score += tc1(c1, c2, c3);
        score += tc2(c2, c3, c4);
        score += tc3(c3, c4, c5);
        score += tc4(c4, c5, c6);

        score += uq1(p1, c1);
        score += uq2(p2, c2);
        score += uq3(p3, c3);

        score += bq1(p2, c2, c3);
        score += bq2(p2, c3, c4);
        score += bq3(p3, c2, c3);
        score += bq4(p3, c3, c4);

        score += tq1(p2, c1, c2, c3);
        score += tq2(p2, c2, c3, c4);
        score += tq3(p3, c1, c2, c3);
        score += tq4(p3, c2, c3, c4);

        let mut p = Ctype::O;
        if score > 0 {
            result.push(&input[indices[word_start]..indices[i - 3]]);
            word_start = i - 3;
            p = Ctype::B;
        }
        (p1, p2, p3) = (p2, p3, p);
        word.push_str(seg[i]);
    }
    result.push(&input[indices[word_start]..]);
    result
}

#[cfg(test)]
mod test {
    use crate::util::segmenter::segment;

    #[test]
    fn test() {
        println!("{:?}", segment("今日もいい天気"));
    }
}
