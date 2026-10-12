//! 罫線素片を線に直します。
//!
//! ブラウザはフォントの字形で罫線素片を描くので、字形の大きさがセルと違えば
//! 継ぎ目に隙間ができ、行ごとに描いて重ねる分だけ濃くなります。
//! advance が ASCII の 2 倍のフォントでは、格子も崩れます。
//!
//! そこで、`code_font` の字形を空にしたうえで (`code_font::output`)、
//! ここで作った SVG をコードブロックに重ねます。
//! 線はつないで 1 本にするので、継ぎ目が残りません。
//! 塗りは濃さごとに 1 つの path にまとめるので、こちらも継ぎ目が残りません。

use super::code_font::Advances;
use std::collections::BTreeMap;
use unicode_width::UnicodeWidthChar;

/// 腕の太さ。添字は上、下、左、右の順です。
/// 0 は腕なし、1 は細線、2 は太線です。
type Arms = [u8; 4];

/// 描く符号位置と、その腕です。
///
/// U+2500 から U+257F のうち、線を腕で表せるものを入れています。
/// ここに載っていない罫線素片は字形が残るので、フォントのまま描かれます。
/// 二重線、丸角、破線、斜線、ブロック、網掛けは入れていません。
const TABLE: &[(char, Arms)] = &[
    ('─', [0, 0, 1, 1]),
    ('━', [0, 0, 2, 2]),
    ('│', [1, 1, 0, 0]),
    ('┃', [2, 2, 0, 0]),
    ('┌', [0, 1, 0, 1]),
    ('┍', [0, 1, 0, 2]),
    ('┎', [0, 2, 0, 1]),
    ('┏', [0, 2, 0, 2]),
    ('┐', [0, 1, 1, 0]),
    ('┑', [0, 1, 2, 0]),
    ('┒', [0, 2, 1, 0]),
    ('┓', [0, 2, 2, 0]),
    ('└', [1, 0, 0, 1]),
    ('┕', [1, 0, 0, 2]),
    ('┖', [2, 0, 0, 1]),
    ('┗', [2, 0, 0, 2]),
    ('┘', [1, 0, 1, 0]),
    ('┙', [1, 0, 2, 0]),
    ('┚', [2, 0, 1, 0]),
    ('┛', [2, 0, 2, 0]),
    ('├', [1, 1, 0, 1]),
    ('┝', [1, 1, 0, 2]),
    ('┞', [2, 1, 0, 1]),
    ('┟', [1, 2, 0, 1]),
    ('┠', [2, 2, 0, 1]),
    ('┡', [2, 1, 0, 2]),
    ('┢', [1, 2, 0, 2]),
    ('┣', [2, 2, 0, 2]),
    ('┤', [1, 1, 1, 0]),
    ('┥', [1, 1, 2, 0]),
    ('┦', [2, 1, 1, 0]),
    ('┧', [1, 2, 1, 0]),
    ('┨', [2, 2, 1, 0]),
    ('┩', [2, 1, 2, 0]),
    ('┪', [1, 2, 2, 0]),
    ('┫', [2, 2, 2, 0]),
    ('┬', [0, 1, 1, 1]),
    ('┭', [0, 1, 2, 1]),
    ('┮', [0, 1, 1, 2]),
    ('┯', [0, 1, 2, 2]),
    ('┰', [0, 2, 1, 1]),
    ('┱', [0, 2, 2, 1]),
    ('┲', [0, 2, 1, 2]),
    ('┳', [0, 2, 2, 2]),
    ('┴', [1, 0, 1, 1]),
    ('┵', [1, 0, 2, 1]),
    ('┶', [1, 0, 1, 2]),
    ('┷', [1, 0, 2, 2]),
    ('┸', [2, 0, 1, 1]),
    ('┹', [2, 0, 2, 1]),
    ('┺', [2, 0, 1, 2]),
    ('┻', [2, 0, 2, 2]),
    ('┼', [1, 1, 1, 1]),
    ('┽', [1, 1, 2, 1]),
    ('┾', [1, 1, 1, 2]),
    ('┿', [1, 1, 2, 2]),
    ('╀', [2, 1, 1, 1]),
    ('╁', [1, 2, 1, 1]),
    ('╂', [2, 2, 1, 1]),
    ('╃', [2, 1, 2, 1]),
    ('╄', [2, 1, 1, 2]),
    ('╅', [1, 2, 2, 1]),
    ('╆', [1, 2, 1, 2]),
    ('╇', [2, 1, 2, 2]),
    ('╈', [1, 2, 2, 2]),
    ('╉', [2, 2, 2, 1]),
    ('╊', [2, 2, 1, 2]),
    ('╋', [2, 2, 2, 2]),
    ('╴', [0, 0, 1, 0]),
    ('╵', [1, 0, 0, 0]),
    ('╶', [0, 0, 0, 1]),
    ('╷', [0, 1, 0, 0]),
    ('╸', [0, 0, 2, 0]),
    ('╹', [2, 0, 0, 0]),
    ('╺', [0, 0, 0, 2]),
    ('╻', [0, 2, 0, 0]),
    ('╼', [0, 0, 1, 2]),
    ('╽', [1, 2, 0, 0]),
    ('╾', [0, 0, 2, 1]),
    ('╿', [2, 1, 0, 0]),
];

/// 角を丸める符号位置と、その腕です。
///
/// 腕は上下左右のうち 2 つで、太さは細線だけです。Unicode に太線の丸角はありません。
const ROUNDS: &[(char, Arms)] = &[
    ('╭', [0, 1, 0, 1]),
    ('╮', [0, 1, 1, 0]),
    ('╯', [1, 0, 1, 0]),
    ('╰', [1, 0, 0, 1]),
];

/// 二重線の符号位置と、その腕です。腕の値は 0 が無し、1 が単線、2 が二重線です。
///
/// 1 つの文字の中では、上下の腕どうし、左右の腕どうしが同じ値になります。
const DOUBLES: &[(char, Arms)] = &[
    ('═', [0, 0, 2, 2]),
    ('║', [2, 2, 0, 0]),
    ('╒', [0, 1, 0, 2]),
    ('╓', [0, 2, 0, 1]),
    ('╔', [0, 2, 0, 2]),
    ('╕', [0, 1, 2, 0]),
    ('╖', [0, 2, 1, 0]),
    ('╗', [0, 2, 2, 0]),
    ('╘', [1, 0, 0, 2]),
    ('╙', [2, 0, 0, 1]),
    ('╚', [2, 0, 0, 2]),
    ('╛', [1, 0, 2, 0]),
    ('╜', [2, 0, 1, 0]),
    ('╝', [2, 0, 2, 0]),
    ('╞', [1, 1, 0, 2]),
    ('╟', [2, 2, 0, 1]),
    ('╠', [2, 2, 0, 2]),
    ('╡', [1, 1, 2, 0]),
    ('╢', [2, 2, 1, 0]),
    ('╣', [2, 2, 2, 0]),
    ('╤', [0, 1, 2, 2]),
    ('╥', [0, 2, 1, 1]),
    ('╦', [0, 2, 2, 2]),
    ('╧', [1, 0, 2, 2]),
    ('╨', [2, 0, 1, 1]),
    ('╩', [2, 0, 2, 2]),
    ('╪', [1, 1, 2, 2]),
    ('╫', [2, 2, 1, 1]),
    ('╬', [2, 2, 2, 2]),
];

/// 二重線の 2 本の間隔です。軸からの距離を、縦方向の単位で持ちます。
///
/// 画面の上で縦横を揃えたいので、横方向の距離は比から出します。
/// 手元の 29 フォントでは、2 本の中心間が 0.08em から 0.176em まで散らばっていました。
/// どれかに合わせる必要はないので、本文の大きさで 2 本に見える値を選んでいます。
const DOUBLE_GAP: u32 = 2;

/// 破線の符号位置です。1 セルに引く本数、太さ、横向きかどうかを持ちます。
///
/// 線と間隔の長さは CSS が決めるので、ここには 1 セルの本数だけを持ちます。
const DASHES: &[(char, u8, u8, bool)] = &[
    ('┄', 3, 1, true),
    ('┅', 3, 2, true),
    ('┆', 3, 1, false),
    ('┇', 3, 2, false),
    ('┈', 4, 1, true),
    ('┉', 4, 2, true),
    ('┊', 4, 1, false),
    ('┋', 4, 2, false),
    ('╌', 2, 1, true),
    ('╍', 2, 2, true),
    ('╎', 2, 1, false),
    ('╏', 2, 2, false),
];

/// 斜線の符号位置と、その向きです。添字は左上から右下、左下から右上の順です。
///
/// セルの角から角へ引くので、画面の上では 45 度になりません。
/// セルが縦長であれば、斜線の傾きもその分だけ急になります。
const DIAGONALS: &[(char, [bool; 2])] = &[
    ('╱', [false, true]),
    ('╲', [true, false]),
    ('╳', [true, true]),
];

/// 塗りで描く符号位置と、その長方形です。
///
/// 座標はセルの左上を (0, 0)、右下を (`CELL`, `CELL`) とする位置です。
const BLOCKS: &[(char, &[Rect])] = &[
    ('▀', &[(0, 0, 24, 12)]),
    ('▁', &[(0, 21, 24, 24)]),
    ('▂', &[(0, 18, 24, 24)]),
    ('▃', &[(0, 15, 24, 24)]),
    ('▄', &[(0, 12, 24, 24)]),
    ('▅', &[(0, 9, 24, 24)]),
    ('▆', &[(0, 6, 24, 24)]),
    ('▇', &[(0, 3, 24, 24)]),
    ('█', &[(0, 0, 24, 24)]),
    ('▉', &[(0, 0, 21, 24)]),
    ('▊', &[(0, 0, 18, 24)]),
    ('▋', &[(0, 0, 15, 24)]),
    ('▌', &[(0, 0, 12, 24)]),
    ('▍', &[(0, 0, 9, 24)]),
    ('▎', &[(0, 0, 6, 24)]),
    ('▏', &[(0, 0, 3, 24)]),
    ('▐', &[(12, 0, 24, 24)]),
    ('▔', &[(0, 0, 24, 3)]),
    ('▕', &[(21, 0, 24, 24)]),
    ('▖', &[(0, 12, 12, 24)]),
    ('▗', &[(12, 12, 24, 24)]),
    ('▘', &[(0, 0, 12, 12)]),
    ('▙', &[(0, 0, 12, 12), (0, 12, 12, 24), (12, 12, 24, 24)]),
    ('▚', &[(0, 0, 12, 12), (12, 12, 24, 24)]),
    ('▛', &[(0, 0, 12, 12), (12, 0, 24, 12), (0, 12, 12, 24)]),
    ('▜', &[(0, 0, 12, 12), (12, 0, 24, 12), (12, 12, 24, 24)]),
    ('▝', &[(12, 0, 24, 12)]),
    ('▞', &[(12, 0, 24, 12), (0, 12, 12, 24)]),
    ('▟', &[(12, 0, 24, 12), (0, 12, 12, 24), (12, 12, 24, 24)]),
];

/// 網掛けの符号位置と、その濃さです。濃さは `DENSITY_STEPS` 分のいくつかで表します。
///
/// もとは CP437 の 176 から 178 で、点の模様でセルを埋めるものです。
/// 模様の繰り返しがセルの境で合うことを当てにした文字なので、
/// フォントの字形に任せると境で合わないことがあります。
/// 端末も模様を使わず、前景色を薄く塗って描くものがあります。
const SHADES: &[(char, u8)] = &[('░', 1), ('▒', 2), ('▓', 3)];

/// 塗りの濃さの段数です。`BLOCKS` の長方形はこの濃さで塗ります。
const DENSITY_STEPS: u8 = 4;

/// 塗りの座標の刻みです。`BLOCKS` の長方形はすべてこの倍数に載ります。
const FILL_UNIT: u32 = 3;

/// 塗る長方形です。左上と右下を持ちます。
type Rect = (u32, u32, u32, u32);

/// SVG で描く文字かどうかを返します。
///
/// SVG で描く文字と、フォントから字形を落とす文字は同じである必要があります。
/// 片方だけ変えると、線が二重に描かれるか、消えたままになります。
pub fn is_covered(c: char) -> bool {
    arms(c).is_some()
        || rounds(c).is_some()
        || doubles(c).is_some()
        || blocks(c).is_some()
        || shades(c).is_some()
        || diagonals(c).is_some()
        || dashes(c).is_some()
}

fn arms(c: char) -> Option<Arms> {
    TABLE.iter().find(|(x, _)| *x == c).map(|(_, a)| *a)
}

fn doubles(c: char) -> Option<Arms> {
    DOUBLES.iter().find(|(x, _)| *x == c).map(|(_, a)| *a)
}

fn rounds(c: char) -> Option<Arms> {
    ROUNDS.iter().find(|(x, _)| *x == c).map(|(_, a)| *a)
}

fn blocks(c: char) -> Option<&'static [Rect]> {
    BLOCKS.iter().find(|(x, _)| *x == c).map(|(_, r)| *r)
}

fn diagonals(c: char) -> Option<[bool; 2]> {
    DIAGONALS.iter().find(|(x, _)| *x == c).map(|(_, d)| *d)
}

fn shades(c: char) -> Option<u8> {
    SHADES.iter().find(|(x, _)| *x == c).map(|(_, d)| *d)
}

fn dashes(c: char) -> Option<(u8, u8, bool)> {
    DASHES
        .iter()
        .find(|(x, ..)| *x == c)
        .map(|(_, count, weight, horizontal)| (*count, *weight, *horizontal))
}

/// 文字がセルをいくつ占めるかを返します。
///
/// フォントが持っている文字は送り幅から決めます。持っていない文字はフォールバックの
/// フォントで描かれるため送り幅を読めないので、East Asian Width で推測します。
/// 幅を持たない文字は 1 と数えます。
/// タブは桁を揃えられないので、罫線素片と同じ行には書けません。
fn cell_width(c: char, advances: Option<&Advances>) -> u32 {
    advances
        .and_then(|a| a.cells(c))
        .unwrap_or_else(|| c.width().unwrap_or(1) as u32)
}

/// コードブロックの行の高さ。SVG の縦横比を出すのに使い、CSS にも同じ値を書き出します。
pub const CODE_LINE_HEIGHT: f32 = 1.25;

/// Rust 側が持っている値を、CSS のカスタムプロパティとして書き出します。
pub fn css_variables() -> String {
    format!("pre {{\n  --code-line-height: {CODE_LINE_HEIGHT};\n}}\n")
}

/// セルの高さが幅の何倍かを返します。
pub fn cell_ratio(advances: &Advances) -> f32 {
    CODE_LINE_HEIGHT / advances.cell_width_em()
}

/// 円弧の縦の半径を返します。横の半径はセルの幅の半分で固定です。
///
/// 画面の上で真円に見えるよう、縦は比で割ります。
/// 座標を整数に保つために丸めるので、半径は 1 単位まで狂います。
fn arc_radius(ratio: f32) -> u32 {
    let ry = (CELL as f32 / 2.0 / ratio).round();
    (ry as u32).clamp(1, CELL / 2)
}

/// 二重線の 2 本の、軸からの距離を返します。縦と横で単位の長さが違うので、横は比を掛けます。
fn double_gap(ratio: f32) -> (u32, u32) {
    let gx = (DOUBLE_GAP as f32 * ratio).round();
    ((gx as u32).clamp(1, CELL / 2 - 1), DOUBLE_GAP)
}

/// 1 セルの長さです。座標はこの単位の整数で持ちます。
///
/// ブロックはセルを 8 つに、破線は 2 つから 4 つに割ります。
/// どちらも割り切れる数にすると、座標に小数が出ません。
const CELL: u32 = 24;

/// 座標です。コードブロックの左上が原点で、セルの中心は `CELL / 2` の奇数倍になります。
type Point = (u32, u32);

/// 線のつながりです。点から、辺でつながる隣の点への対応を持ちます。
///
/// 出力を毎回同じにしたいので、並びの決まる BTree を使います。
#[derive(Default)]
struct Lines(BTreeMap<Point, BTreeMap<Point, Edge>>);

/// 2 点の結び方です。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Edge {
    /// まっすぐ結びます。
    Line,
    /// 四分円で結びます。`sweep` は SVG の sweep-flag です。
    Arc { sweep: bool },
}

impl Edge {
    /// 逆向きに辿るときの結び方を返します。円弧は回る向きが逆になります。
    fn flip(self) -> Self {
        match self {
            Self::Line => Self::Line,
            Self::Arc { sweep } => Self::Arc { sweep: !sweep },
        }
    }
}

impl Lines {
    /// 直線の辺を 1 本足します。
    fn add(&mut self, a: Point, b: Point) {
        self.insert(a, b, Edge::Line);
    }

    /// 円弧の辺を 1 本足します。`sweep` は `a` から `b` へ辿るときの向きです。
    fn add_arc(&mut self, a: Point, b: Point, sweep: bool) {
        self.insert(a, b, Edge::Arc { sweep });
    }

    fn insert(&mut self, a: Point, b: Point, edge: Edge) {
        self.0.entry(a).or_default().insert(b, edge);
        self.0.entry(b).or_default().insert(a, edge.flip());
    }

    /// 辺を 1 本取り除きます。辺のなくなった点は落とします。
    fn remove(&mut self, a: Point, b: Point) {
        for (from, to) in [(a, b), (b, a)] {
            if let Some(next) = self.0.get_mut(&from) {
                next.remove(&to);
                if next.is_empty() {
                    self.0.remove(&from);
                }
            }
        }
    }

    /// 次に進む点を選びます。まっすぐ進める辺があれば、それを選びます。
    ///
    /// まっすぐを選ぶので、同じ向きに続く線は 1 本になり、
    /// 交差点では通り抜ける線が 1 本になります。
    fn next(&self, point: Point, from: Option<Point>) -> Option<(Point, Edge)> {
        let next = self.0.get(&point)?;
        let straight =
            from.and_then(|from| next.iter().find(|(to, _)| is_straight(from, point, **to)));

        straight
            .or_else(|| next.iter().next())
            .map(|(to, edge)| (*to, *edge))
    }

    /// 辺を辿り、通った点と、そこへ来た辺を順に返します。辿った辺は取り除きます。
    fn walk(&mut self, start: Point, from: Option<Point>) -> Vec<(Point, Edge)> {
        let mut out = Vec::new();
        let (mut point, mut from) = (start, from);

        while let Some((next, edge)) = self.next(point, from) {
            self.remove(point, next);
            out.push((next, edge));
            from = Some(point);
            point = next;
        }

        out
    }

    /// `d` 属性を作ります。線が 1 本もなければ `None` を返します。
    ///
    /// `arc_ry` は円弧の縦の半径です。
    fn path_data(mut self, arc_ry: u32) -> Option<String> {
        let mut d = String::new();

        while let Some(start) = self.0.keys().next().copied() {
            // 両方向に伸ばします。先に辿ったほうは逆向きなので、並べ直します。
            let back = self.walk(start, None);
            let forward = self.walk(start, back.first().map(|(p, _)| *p));

            // 逆向きにすると、始点は最後に着いた点になり、辺の向きも入れ替わります。
            let first = back.last().map_or(start, |(p, _)| *p);
            let mut steps: Vec<_> = (0..back.len())
                .rev()
                .map(|i| {
                    let to = if i == 0 { start } else { back[i - 1].0 };
                    (to, back[i].1.flip())
                })
                .collect();
            steps.extend(forward);

            d.push_str(&subpath(first, &steps, arc_ry));
        }

        (!d.is_empty()).then_some(d)
    }
}

/// 3 点が一直線に並び、`from` と `to` が `point` の反対側にあるかどうかを返します。
///
/// `from` から `point` への向きと、`point` から `to` への向きを比べます。
/// 外積が 0 なら一直線に並び、内積が正なら同じ向きに進みます。
fn is_straight(from: Point, point: Point, to: Point) -> bool {
    let delta = |a: Point, b: Point| {
        (
            i64::from(b.0) - i64::from(a.0),
            i64::from(b.1) - i64::from(a.1),
        )
    };
    let (ax, ay) = delta(from, point);
    let (bx, by) = delta(point, to);

    ax * by == ay * bx && ax * bx + ay * by > 0
}

/// 折れ線を 1 つのサブパスにします。
///
/// 向きが変わらない点は書きません。始点に戻る線は `Z` で閉じます。
/// 1 つのサブパスにすると、曲がり角が継ぎ目 (linejoin) として描かれます。
/// 別々に引くと端が並ぶだけになり、角の外側が線の太さの半分だけ欠けます。
fn subpath(first: Point, steps: &[(Point, Edge)], arc_ry: u32) -> String {
    // まっすぐ続く点は書く必要がないので、手前の辺に畳みます。
    let mut corners: Vec<(Point, Edge)> = Vec::new();
    for &(point, edge) in steps {
        let straight = match (corners.len(), corners.last()) {
            (n, Some(&(prev, Edge::Line))) if edge == Edge::Line => {
                let before = if n == 1 { first } else { corners[n - 2].0 };
                is_straight(before, prev, point)
            }
            _ => false,
        };

        match straight {
            true => *corners.last_mut().expect("まっすぐなら手前がある") = (point, edge),
            false => corners.push((point, edge)),
        }
    }

    // 始点に戻る線は閉じます。最後が直線であれば、書かずに `Z` に任せます。
    let closed = corners.len() > 2 && corners.last().map(|(p, _)| *p) == Some(first);
    let body = match closed && corners.last().is_some_and(|(_, e)| *e == Edge::Line) {
        true => &corners[..corners.len() - 1],
        false => &corners[..],
    };

    let mut prev = first;
    let mut d = format!("M{} {}", first.0, first.1);
    for &((x, y), edge) in body {
        match edge {
            Edge::Arc { sweep } => {
                let sweep = u8::from(sweep);
                d.push_str(&format!("A{} {arc_ry} 0 0 {sweep} {x} {y}", CELL / 2));
            }
            Edge::Line if y == prev.1 => d.push_str(&format!("H{x}")),
            Edge::Line if x == prev.0 => d.push_str(&format!("V{y}")),
            Edge::Line => d.push_str(&format!("L{x} {y}")),
        }
        prev = (x, y);
    }
    if closed {
        d.push('Z');
    }

    d
}

/// 破線を描き分ける組です。太さ、横向きかどうか、1 セルの本数を持ちます。
///
/// 組が違うと線と間隔の長さが変わります。CSS で指定するので、組ごとに別の path にします。
type Dash = (u8, bool, u8);

/// 二重線の交点を組み立てます。
///
/// 軸から ±gap の位置に走るレールと、それらをつなぐ枠でできています。
/// 腕はつねに手前のレールで止めます。枠は 4 辺のうち、腕のある向きの辺を外します。
/// `╬` は 4 辺とも外れて中心が開き、`╠` は左の辺だけが残って左の縦線が通ります。
/// 単線が二重線を横切るところには渡りの線を足します。
/// これで `╒` のように、単線が二重線の蓋になる形も出ます。
fn add_double(lines: &mut Lines, arms: Arms, cell: &CellBox, gap: (u32, u32)) {
    let [up, down, left, right] = arms;
    let (gx, gy) = gap;
    let (cx, cy) = (cell.cx, cell.cy);

    // レールの位置です。二重なら 2 本、単線なら 1 本、腕がなければ 0 本です。
    let rails = |kind: u8, center: u32, g: u32| match kind {
        2 => vec![center - g, center + g],
        1 => vec![center],
        _ => Vec::new(),
    };
    let vertical = rails(up.max(down), cx, gx);
    let horizontal = rails(left.max(right), cy, gy);

    // 手前のレールの位置です。相手がいなければセルの中心で止めます。
    let near = |rails: &[u32], center: u32, small: bool| match (rails, small) {
        ([], _) => center,
        (other, true) => other[0],
        (other, false) => other[other.len() - 1],
    };

    for &x in &vertical {
        if up != 0 {
            lines.add((x, cell.top), (x, near(&horizontal, cy, true)));
        }
        if down != 0 {
            lines.add((x, cell.bottom), (x, near(&horizontal, cy, false)));
        }
    }

    for &y in &horizontal {
        if left != 0 {
            lines.add((cell.left, y), (near(&vertical, cx, true), y));
        }
        if right != 0 {
            lines.add((cell.right, y), (near(&vertical, cx, false), y));
        }
    }

    // 単線が二重線を横切るところを渡します。
    if vertical.len() == 1 && horizontal.len() == 2 {
        lines.add((cx, cy - gy), (cx, cy + gy));
    }
    if horizontal.len() == 1 && vertical.len() == 2 {
        lines.add((cx - gx, cy), (cx + gx, cy));
    }

    // 枠は、縦横とも二重のときだけ要ります。腕のない向きの辺を残します。
    if vertical.len() == 2 && horizontal.len() == 2 {
        let (x0, x1, y0, y1) = (cx - gx, cx + gx, cy - gy, cy + gy);
        if up == 0 {
            lines.add((x0, y0), (x1, y0));
        }
        if down == 0 {
            lines.add((x0, y1), (x1, y1));
        }
        if left == 0 {
            lines.add((x0, y0), (x0, y1));
        }
        if right == 0 {
            lines.add((x1, y0), (x1, y1));
        }
    }
}

/// セルの枠の座標です。
struct CellBox {
    left: u32,
    right: u32,
    top: u32,
    bottom: u32,
    cx: u32,
    cy: u32,
}

/// 図から取り出した形です。
#[derive(Default)]
struct Shapes {
    /// 太さごとの線。
    lines: [Lines; 2],
    /// 組ごとの破線。
    dashes: BTreeMap<Dash, Lines>,
    /// 塗る長方形と、その濃さ。座標は線と同じ単位です。
    fills: Vec<(u8, Rect)>,
    cols: u32,
    rows: u32,
}

/// コードブロックの中身から形を集めます。
fn collect(code: &str, advances: Option<&Advances>, arc_ry: u32, gap: (u32, u32)) -> Shapes {
    let mut shapes = Shapes {
        rows: code.lines().count() as u32,
        ..Shapes::default()
    };
    let lines = &mut shapes.lines;

    for (row, text) in code.lines().enumerate() {
        let mut col = 0;
        let (top, bottom) = (row as u32 * CELL, row as u32 * CELL + CELL);

        for c in text.chars() {
            let (left, right) = (col * CELL, col * CELL + CELL);
            let (cx, cy) = (left + CELL / 2, top + CELL / 2);

            for &(x0, y0, x1, y1) in blocks(c).unwrap_or_default() {
                let rect = (left + x0, top + y0, left + x1, top + y1);
                shapes.fills.push((DENSITY_STEPS, rect));
            }

            if let Some(density) = shades(c) {
                shapes.fills.push((density, (left, top, right, bottom)));
            }

            if let Some((count, weight, horizontal)) = dashes(c) {
                // 線はセルの端から端まで 1 本引きます。
                // 線と間隔の繰り返しは CSS の `stroke-dasharray` が決めます。
                // 組ごとに別の `Lines` なので、同じ破線が続くセルだけが
                // 端で一致し、1 本のサブパスになります。
                let (from, to) = if horizontal {
                    ((left, cy), (right, cy))
                } else {
                    ((cx, top), (cx, bottom))
                };
                shapes
                    .dashes
                    .entry((weight, horizontal, count))
                    .or_default()
                    .add(from, to);
            }

            if let Some(arms) = doubles(c) {
                let cell = CellBox {
                    left,
                    right,
                    top,
                    bottom,
                    cx,
                    cy,
                };
                add_double(&mut lines[0], arms, &cell, gap);
            }

            if let Some([up, down, l, r]) = rounds(c) {
                // 円弧は横の腕の端から、縦の腕の側へ回ります。
                let (from, across) = match (l, r) {
                    (0, _) => ((right, cy), (cx, cy)),
                    _ => ((left, cy), (cx, cy)),
                };
                let (to, arm) = match (up, down) {
                    (0, _) => ((across.0, cy + arc_ry), (cx, bottom)),
                    _ => ((across.0, cy - arc_ry), (cx, top)),
                };
                // 進む向きが右回りなら sweep を立てます。
                let turn = (from.0 < to.0) == (from.1 < to.1);
                lines[0].add_arc(from, to, turn);
                lines[0].add(to, arm);
            }

            if let Some([falling, rising]) = diagonals(c) {
                if falling {
                    lines[0].add((left, top), (right, bottom));
                }
                if rising {
                    lines[0].add((left, bottom), (right, top));
                }
            }

            if let Some([up, down, l, r]) = arms(c) {
                if up != 0 {
                    lines[up as usize - 1].add((cx, cy), (cx, top));
                }
                if down != 0 {
                    lines[down as usize - 1].add((cx, cy), (cx, bottom));
                }
                if l != 0 {
                    lines[l as usize - 1].add((cx, cy), (left, cy));
                }
                if r != 0 {
                    lines[r as usize - 1].add((cx, cy), (right, cy));
                }
            }

            col += cell_width(c, advances);
        }

        shapes.cols = shapes.cols.max(col);
    }

    shapes
}

/// 同じ濃さで隣り合う長方形を、1 つの大きな長方形にまとめます。
///
/// 継ぎ目のためではなく、path のデータを短くするために行います。
/// 60 列 12 行を塗りつぶす図では、まとめると 15190 バイトが 39 バイトになりました。
/// `FILL_UNIT` の格子に敷き直したうえで、右と下に伸ばせるだけ伸ばした長方形を切り出します。
fn merge_fills(fills: &[(u8, Rect)], cols: u32, rows: u32) -> Vec<(u8, Rect)> {
    let w = (cols * CELL / FILL_UNIT) as usize;
    let h = (rows * CELL / FILL_UNIT) as usize;
    let mut grid = vec![0u8; w * h];
    for &(density, (x0, y0, x1, y1)) in fills {
        for y in y0 / FILL_UNIT..y1 / FILL_UNIT {
            for x in x0 / FILL_UNIT..x1 / FILL_UNIT {
                grid[y as usize * w + x as usize] = density;
            }
        }
    }

    let mut merged = Vec::new();
    let mut taken = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            let density = grid[y * w + x];
            if density == 0 || taken[y * w + x] {
                continue;
            }
            let free = |grid: &[u8], taken: &[bool], i: usize| grid[i] == density && !taken[i];
            let mut right = x + 1;
            while right < w && free(&grid, &taken, y * w + right) {
                right += 1;
            }
            let mut bottom = y + 1;
            while bottom < h && (x..right).all(|i| free(&grid, &taken, bottom * w + i)) {
                bottom += 1;
            }
            for row in y..bottom {
                taken[row * w + x..row * w + right].fill(true);
            }
            let unit = |v: usize| v as u32 * FILL_UNIT;
            merged.push((density, (unit(x), unit(y), unit(right), unit(bottom))));
        }
    }
    merged
}

/// 濃さに対応する `fill-opacity` です。塗りつぶしのときは属性を出しません。
fn fill_opacity(density: u8) -> String {
    if density >= DENSITY_STEPS {
        return String::new();
    }
    format!(
        r#" fill-opacity="{:.2}""#,
        f32::from(density) / f32::from(DENSITY_STEPS)
    )
}

/// 太さに対応する class です。
fn weight_class(weight: u8) -> &'static str {
    if weight == 1 {
        "box-light"
    } else {
        "box-heavy"
    }
}

/// コードブロックに重ねる SVG を作ります。
///
/// 描く罫線素片が 1 つもなければ `None` を返します。
/// 座標の単位は 1 セルの `CELL` 分の 1 なので、viewBox は列数と行数の `CELL` 倍になります。
/// 実際の大きさは CSS が `--cols` と `--rows` から決めます。
pub fn svg(code: &str, advances: Option<&Advances>, ratio: f32) -> Option<String> {
    let arc_ry = arc_radius(ratio);
    let gap = double_gap(ratio);
    let Shapes {
        lines: [light, heavy],
        dashes,
        fills,
        cols,
        rows,
    } = collect(code, advances, arc_ry, gap);

    let mut paths = String::new();
    for (lines, weight) in [(light, 1), (heavy, 2)] {
        if let Some(d) = lines.path_data(arc_ry) {
            let class = weight_class(weight);
            paths.push_str(&format!(r#"<path class="{class}" d="{d}"/>"#));
        }
    }

    for ((weight, horizontal, count), lines) in dashes {
        let weight = weight_class(weight);
        let dir = if horizontal {
            "box-dash-x"
        } else {
            "box-dash-y"
        };
        if let Some(d) = lines.path_data(arc_ry) {
            paths.push_str(&format!(
                r#"<path class="{weight} {dir}" style="--n:{count}" d="{d}"/>"#
            ));
        }
    }

    // 濃さごとに 1 つの path にします。path 要素を分けると、
    // 境の列が下地と混ざって縞に見えます。1 つの path の中であれば、
    // サブパスが辺を共有しても継ぎ目は出ません。
    let mut by_density: BTreeMap<u8, String> = BTreeMap::new();
    for (density, (x0, y0, x1, y1)) in merge_fills(&fills, cols, rows) {
        let d = by_density.entry(density).or_default();
        d.push_str(&format!("M{x0} {y0}H{x1}V{y1}H{x0}Z"));
    }
    for (density, d) in by_density {
        let opacity = fill_opacity(density);
        paths.push_str(&format!(r#"<path class="box-fill"{opacity} d="{d}"/>"#));
    }

    (!paths.is_empty()).then(|| {
        format!(
            r#"<svg class="box-drawing" style="--cols:{cols};--rows:{rows}" viewBox="0 0 {} {}" preserveAspectRatio="none" aria-hidden="true">{paths}</svg>"#,
            cols * CELL,
            rows * CELL,
        )
    })
}

#[cfg(test)]
mod test {
    use super::is_covered;

    /// フォントを渡さずに描きます。列は East Asian Width で数えます。
    /// 比はセルが縦 2 倍のものとします。
    fn svg(code: &str) -> Option<String> {
        super::svg(code, None, 2.0)
    }

    /// 1 つの箱は 1 つのサブパスになります。文字ごとに描けば 12 本のところです。
    /// 端は角の中心までなので、4 列の箱の横線は 12 から 84 までです。
    #[test]
    fn joins_a_box_into_one_subpath() {
        let d = svg("┌──┐\n│  │\n└──┘\n").unwrap();
        assert!(d.contains(r#"d="M12 12H84V60H12Z""#), "{d}");
    }

    /// 全角の文字は 2 セルと数えます。
    #[test]
    fn counts_full_width_characters_as_two_cells() {
        let d = svg("┌──┐\n│図│\n└──┘\n").unwrap();
        assert!(d.contains(r#"--cols:4;--rows:3"#), "{d}");
    }

    /// 角で曲がる線は、続けて 1 つのサブパスに書きます。
    /// 別々に引くと端が並ぶだけになり、曲がり角の外側が線の太さの半分だけ欠けます。
    #[test]
    fn turns_a_corner_within_one_subpath() {
        let d = svg("┌─\n│ \n").unwrap();
        assert!(d.contains(r#"d="M12 48V12H48""#), "{d}");
    }

    /// 線が通り抜ける文字では、端が一致しないので別のサブパスになります。
    /// 交点は通り抜ける線が覆うので、つなぐ必要がありません。
    #[test]
    fn keeps_crossings_apart() {
        let d = svg("┼\n").unwrap();
        assert!(d.contains(r#"d="M24 12H0M12 24V0""#), "{d}");
    }

    /// ブロックは線ではなく長方形の塗りです。
    #[test]
    fn fills_a_block() {
        let d = svg("\u{2588}").unwrap();
        assert!(
            d.contains(r#"<path class="box-fill" d="M0 0H24V24H0Z"/>"#),
            "{d}"
        );
    }

    /// 四分割のブロックは長方形 2 つ以上になります。
    #[test]
    fn fills_each_quadrant() {
        let d = svg("\u{259e}").unwrap();
        assert!(d.contains(r#"d="M12 0H24V12H12ZM0 12H12V24H0Z""#), "{d}");
    }

    /// 斜線はセルの角から角へ引きます。
    #[test]
    fn draws_a_diagonal() {
        let d = svg("\u{2572}").unwrap();
        assert!(d.contains(r#"d="M24 24L0 0""#), "{d}");
    }

    /// `╳` は 2 本です。中心で交わりますが、端は一致しないので別のサブパスになります。
    #[test]
    fn crosses_two_diagonals() {
        let d = svg("\u{2573}").unwrap();
        assert!(d.contains(r#"d="M24 24L0 0M24 0L0 24""#), "{d}");
    }

    /// 斜めに続く斜線は端が一致するので、1 本のサブパスになります。
    #[test]
    fn joins_diagonals_across_rows() {
        let d = svg("\u{2572} \n \u{2572}\n").unwrap();
        assert!(d.contains(r#"d="M48 48L0 0""#), "{d}");
    }

    /// 網掛けはセル全体を薄く塗ります。
    #[test]
    fn fills_a_shade_with_a_lighter_paint() {
        let d = svg("\u{2591}").unwrap();
        assert!(
            d.contains(r#"<path class="box-fill" fill-opacity="0.25" d="M0 0H24V24H0Z"/>"#),
            "{d}"
        );
    }

    /// 同じ濃さで隣り合う塗りは、1 つの長方形になります。
    #[test]
    fn merges_fills_that_touch() {
        let d = svg("\u{2592}\u{2592}\u{2592}").unwrap();
        assert!(
            d.contains(r#"fill-opacity="0.50" d="M0 0H72V24H0Z""#),
            "{d}"
        );
    }

    /// 行をまたいでもまとめます。
    #[test]
    fn merges_fills_across_rows() {
        let d = svg("\u{2588}\u{2588}\n\u{2588}\u{2588}\n").unwrap();
        assert!(
            d.contains(r#"<path class="box-fill" d="M0 0H48V48H0Z"/>"#),
            "{d}"
        );
    }

    /// 濃さが違えば別の長方形になります。濃さごとに path も分かれます。
    #[test]
    fn keeps_fills_of_different_densities_apart() {
        let d = svg("\u{2593}\u{2588}").unwrap();
        assert!(
            d.contains(r#"<path class="box-fill" fill-opacity="0.75" d="M0 0H24V24H0Z"/>"#),
            "{d}"
        );
        assert!(
            d.contains(r#"<path class="box-fill" d="M24 0H48V24H24Z"/>"#),
            "{d}"
        );
    }

    /// 隣り合う半分のブロックもまとめます。長方形の境はセルの中にもあります。
    #[test]
    fn merges_fills_inside_a_cell() {
        let d = svg("\u{2590}\u{258c}").unwrap();
        assert!(
            d.contains(r#"<path class="box-fill" d="M12 0H36V24H12Z"/>"#),
            "{d}"
        );
    }

    /// 腕ごとに太さが違う文字では、太さごとに別の path に分かれます。
    #[test]
    fn splits_a_junction_of_two_weights() {
        let d = svg("┍━\n│ \n").unwrap();
        assert!(
            d.contains(r#"<path class="box-light" d="M12 48V12"/>"#),
            "{d}"
        );
        assert!(
            d.contains(r#"<path class="box-heavy" d="M48 12H12"/>"#),
            "{d}"
        );
    }

    /// 片側だけに腕のある文字も描きます。
    #[test]
    fn draws_a_half_line() {
        let d = svg("╴\n").unwrap();
        assert!(d.contains(r#"d="M12 12H0""#), "{d}");
    }

    /// 破線はセルの端から端までの 1 本として書きます。
    /// 線と間隔の繰り返しは CSS が決めます。同じ破線が続くセルは、1 本にまとまります。
    #[test]
    fn draws_a_run_of_dashes_as_one_line() {
        let d = svg("╌╌╌").unwrap();
        assert!(
            d.contains(r#"<path class="box-light box-dash-x" style="--n:2" d="M72 12H0"/>"#),
            "{d}"
        );
    }

    /// 破線は実線とつなぎません。つなぐと間隔が消えます。
    #[test]
    fn keeps_dashes_apart_from_solid_lines() {
        let d = svg("─╌").unwrap();
        assert!(
            d.contains(r#"<path class="box-light" d="M24 12H0"/>"#),
            "{d}"
        );
        assert!(
            d.contains(r#"box-dash-x" style="--n:2" d="M48 12H24""#),
            "{d}"
        );
    }

    /// 本数や向きが違う破線は、間隔が違うので別の path にします。
    #[test]
    fn splits_dashes_by_count_and_direction() {
        let d = svg("╌┄╎").unwrap();
        assert!(
            d.contains(r#"box-dash-x" style="--n:2" d="M24 12H0""#),
            "{d}"
        );
        assert!(
            d.contains(r#"box-dash-x" style="--n:3" d="M48 12H24""#),
            "{d}"
        );
        assert!(
            d.contains(r#"box-dash-y" style="--n:2" d="M60 24V0""#),
            "{d}"
        );
    }

    /// 太さごとに別の path にします。
    #[test]
    fn separates_heavy_lines() {
        let d = svg("─\n━\n").unwrap();
        assert!(
            d.contains(r#"<path class="box-light" d="M24 12H0"/>"#),
            "{d}"
        );
        assert!(
            d.contains(r#"<path class="box-heavy" d="M24 36H0"/>"#),
            "{d}"
        );
    }

    /// 丸角の箱は、角丸でない箱と同じく 1 つの閉じたサブパスになります。
    /// 比が 2 なので、縦の半径は横の半分の 6 です。
    #[test]
    fn rounds_a_corner_within_one_subpath() {
        let d = svg("╭─╮\n│ │\n╰─╯\n").unwrap();
        let want = "M12 18A12 6 0 0 1 24 12H48A12 6 0 0 1 60 18\
                    V54A12 6 0 0 1 48 60H24A12 6 0 0 1 12 54Z";
        assert!(d.contains(&format!(r#"d="{want}""#)), "{d}");
    }

    /// 縦の半径は、横の半径をセルの縦横比で割ったものです。
    #[test]
    fn shrinks_the_arc_to_the_cell_shape() {
        use super::arc_radius;
        assert_eq!(arc_radius(1.0), 12);
        assert_eq!(arc_radius(2.0), 6);
        assert_eq!(arc_radius(2.5), 5);
        assert_eq!(arc_radius(2.076), 6);
    }

    /// 縦に長いセルでも、半径は 1 を下回りません。
    #[test]
    fn keeps_the_arc_visible() {
        use super::arc_radius;
        assert_eq!(arc_radius(100.0), 1);
    }

    /// 二重線は軸から等しく離れた 2 本になります。比が 2 なので、縦は 2、横は 4 です。
    #[test]
    fn draws_two_rails_for_a_double_line() {
        assert!(svg("═").unwrap().contains(r#"d="M24 10H0M24 14H0""#));
        assert!(svg("║").unwrap().contains(r#"d="M8 24V0M16 24V0""#));
    }

    /// 二重線どうしの交点は中心が開き、4 つの L になります。
    #[test]
    fn opens_the_centre_of_a_double_crossing() {
        let d = svg("╬").unwrap();
        let want = "M8 0V10H0M8 24V14H0M24 10H16V0M16 24V14H24";
        assert!(d.contains(&format!(r#"d="{want}""#)), "{d}");
    }

    /// 腕のない側の線は通します。`╠` の左は 1 本の縦線になります。
    #[test]
    fn keeps_the_armless_side_closed() {
        let d = svg("╠").unwrap();
        assert!(d.contains(r#"d="M8 24V0M24 10H16V0M16 24V14H24""#), "{d}");
    }

    /// 片方が単線なら、その線は交点を突き抜けます。
    #[test]
    fn lets_a_single_line_pass_through() {
        assert!(svg("╪").unwrap().contains("M12 24V0"), "");
        assert!(svg("╫").unwrap().contains("M24 12H0"), "");
    }

    /// 単線と二重線が出会う角では、単線が二重線の蓋になります。
    #[test]
    fn caps_a_double_line_with_the_single_arm() {
        let d = svg("╒").unwrap();
        assert!(d.contains(r#"d="M12 24V10H24M24 14H12""#), "{d}");
    }

    /// 2 本の間隔は、縦を決めてから横を比で出します。
    #[test]
    fn widens_the_double_gap_to_the_cell_shape() {
        use super::double_gap;
        assert_eq!(double_gap(1.0), (2, 2));
        assert_eq!(double_gap(2.0), (4, 2));
        assert_eq!(double_gap(2.5), (5, 2));
    }

    #[test]
    fn ignores_code_without_box_drawing() {
        assert!(svg("let x = 1;\n").is_none());
    }

    /// 字形を落とす文字と、描く文字が一致することを確かめます。
    ///
    /// 片方だけに入っていると、線が二重に描かれるか、消えたままになります。
    #[test]
    fn draws_every_covered_character() {
        for c in '\u{2500}'..='\u{259f}' {
            assert_eq!(is_covered(c), svg(&c.to_string()).is_some(), "{c}");
        }
    }
}
