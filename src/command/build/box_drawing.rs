//! 罫線素片を線に直します。
//!
//! ブラウザはフォントの字形で罫線素片を描くので、字形の大きさがセルと違えば
//! 継ぎ目に隙間ができ、行ごとに描いて重ねる分だけ濃くなります。
//! advance が ASCII の 2 倍のフォントでは、格子も崩れます。
//!
//! そこで、`code_font` の字形を空にしたうえで (`code_font::output`)、
//! ここで作った SVG をコードブロックに重ねます。
//! 線はつないで 1 本にするので、継ぎ目が残りません。

use std::collections::{BTreeMap, BTreeSet};
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

/// 破線の符号位置です。1 セルに引く本数、太さ、横向きかどうかを持ちます。
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

/// 破線の間隔です。両端にはこの半分を空けます。
///
/// 隣のセルと合わせて間隔が揃うので、破線が続いても間延びしません。
/// DejaVu Sans Mono の字形では 0.098 セルでした。ここでは 1/12 セルにしています。
const DASH_GAP: u32 = 2;

/// 塗りで描く符号位置と、その長方形です。
///
/// 座標はセルの左上を (0, 0)、右下を (`CELL`, `CELL`) とする位置です。
/// 網掛け (`░▒▓`) は点の模様なので入れていません。
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

/// 塗る長方形です。左上と右下を持ちます。
type Rect = (u32, u32, u32, u32);

/// SVG で描く文字かどうかを返します。
///
/// SVG で描く文字と、フォントから字形を落とす文字は同じである必要があります。
/// 片方だけ変えると、線が二重に描かれるか、消えたままになります。
pub fn is_covered(c: char) -> bool {
    arms(c).is_some() || blocks(c).is_some() || dashes(c).is_some()
}

fn arms(c: char) -> Option<Arms> {
    TABLE.iter().find(|(x, _)| *x == c).map(|(_, a)| *a)
}

fn blocks(c: char) -> Option<&'static [Rect]> {
    BLOCKS.iter().find(|(x, _)| *x == c).map(|(_, r)| *r)
}

fn dashes(c: char) -> Option<(u8, u8, bool)> {
    DASHES
        .iter()
        .find(|(x, ..)| *x == c)
        .map(|(_, count, weight, horizontal)| (*count, *weight, *horizontal))
}

/// 文字がセルをいくつ占めるかを返します。
///
/// ターミナルと同じく East Asian Width で決めます。幅を持たない文字は 1 と数えます。
/// タブは桁を揃えられないので、罫線素片と同じ行には書けません。
fn cell_width(c: char) -> u32 {
    c.width().unwrap_or(1) as u32
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
struct Lines(BTreeMap<Point, BTreeSet<Point>>);

impl Lines {
    /// 辺を 1 本足します。
    fn add(&mut self, a: Point, b: Point) {
        self.0.entry(a).or_default().insert(b);
        self.0.entry(b).or_default().insert(a);
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
    fn next(&self, point: Point, from: Option<Point>) -> Option<Point> {
        let next = self.0.get(&point)?;
        let straight = from.and_then(|from| next.iter().find(|to| is_straight(from, point, **to)));

        straight.or_else(|| next.iter().next()).copied()
    }

    /// 辺を辿り、通った点を順に返します。辿った辺は取り除きます。
    fn walk(&mut self, start: Point, from: Option<Point>) -> Vec<Point> {
        let mut out = Vec::new();
        let (mut point, mut from) = (start, from);

        while let Some(next) = self.next(point, from) {
            self.remove(point, next);
            out.push(next);
            from = Some(point);
            point = next;
        }

        out
    }

    /// `d` 属性を作ります。線が 1 本もなければ `None` を返します。
    fn path_data(mut self) -> Option<String> {
        let mut d = String::new();

        while let Some(start) = self.0.keys().next().copied() {
            // 両方向に伸ばします。先に辿ったほうは逆向きなので、並べ直します。
            let back = self.walk(start, None);
            let forward = self.walk(start, back.first().copied());

            let mut points = back;
            points.reverse();
            points.push(start);
            points.extend(forward);

            d.push_str(&subpath(&points));
        }

        (!d.is_empty()).then_some(d)
    }
}

/// 3 点が一直線に並び、`from` と `to` が `point` の反対側にあるかどうかを返します。
fn is_straight(from: Point, point: Point, to: Point) -> bool {
    let vertical = from.0 == point.0 && to.0 == point.0 && (from.1 < point.1) != (to.1 < point.1);
    let horizontal = from.1 == point.1 && to.1 == point.1 && (from.0 < point.0) != (to.0 < point.0);

    vertical || horizontal
}

/// 折れ線を 1 つのサブパスにします。
///
/// 向きが変わらない点は書きません。始点に戻る線は `Z` で閉じます。
/// 1 つのサブパスにすると、曲がり角が継ぎ目 (linejoin) として描かれます。
/// 別々に引くと端が並ぶだけになり、角の外側が線の太さの半分だけ欠けます。
fn subpath(points: &[Point]) -> String {
    let mut corners = vec![points[0]];
    for p in points.windows(3) {
        if !is_straight(p[0], p[1], p[2]) {
            corners.push(p[1]);
        }
    }
    corners.extend(points.last().filter(|_| points.len() > 1));

    let closed = corners.len() > 2 && corners.first() == corners.last();
    let body = if closed {
        &corners[1..corners.len() - 1]
    } else {
        &corners[1..]
    };

    let mut prev = corners[0];
    let mut d = format!("M{} {}", prev.0, prev.1);
    for &(x, y) in body {
        if y == prev.1 {
            d.push_str(&format!("H{x}"));
        } else {
            d.push_str(&format!("V{y}"));
        }
        prev = (x, y);
    }
    if closed {
        d.push('Z');
    }

    d
}

/// 図から取り出した形です。
#[derive(Default)]
struct Shapes {
    /// 太さごとの線。
    lines: [Lines; 2],
    /// 塗る長方形。座標は線と同じ単位です。
    fills: Vec<Rect>,
    cols: u32,
    rows: u32,
}

/// コードブロックの中身から形を集めます。
fn collect(code: &str) -> Shapes {
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
                shapes
                    .fills
                    .push((left + x0, top + y0, left + x1, top + y1));
            }

            if let Some((count, weight, horizontal)) = dashes(c) {
                let (start, across) = if horizontal { (left, cy) } else { (top, cx) };
                // セルを本数で割り、各区画の中ほどに引きます。
                // 端は奇数になるので、実線や隣のセルとは端が一致せず、つながりません。
                let step = CELL / u32::from(count);
                for i in 0..u32::from(count) {
                    let a = start + step * i + DASH_GAP / 2;
                    let b = a + step - DASH_GAP;
                    let (from, to) = if horizontal {
                        ((a, across), (b, across))
                    } else {
                        ((across, a), (across, b))
                    };
                    lines[weight as usize - 1].add(from, to);
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

            col += cell_width(c);
        }

        shapes.cols = shapes.cols.max(col);
    }

    shapes
}

/// コードブロックに重ねる SVG を作ります。
///
/// 描く罫線素片が 1 つもなければ `None` を返します。
/// 座標の単位は 1 セルの `CELL` 分の 1 なので、viewBox は列数と行数の `CELL` 倍になります。
/// 実際の大きさは CSS が `--cols` と `--rows` から決めます。
pub fn svg(code: &str) -> Option<String> {
    let Shapes {
        lines: [light, heavy],
        fills,
        cols,
        rows,
    } = collect(code);

    let mut paths = String::new();
    for (lines, class) in [(light, "box-light"), (heavy, "box-heavy")] {
        if let Some(d) = lines.path_data() {
            paths.push_str(&format!(r#"<path class="{class}" d="{d}"/>"#));
        }
    }

    if !fills.is_empty() {
        let d: String = fills
            .iter()
            .map(|(x0, y0, x1, y1)| format!("M{x0} {y0}H{x1}V{y1}H{x0}Z"))
            .collect();
        paths.push_str(&format!(r#"<path class="box-fill" d="{d}"/>"#));
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
    use super::{is_covered, svg};

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

    /// 四分円は長方形 2 つ以上になります。
    #[test]
    fn fills_each_quadrant() {
        let d = svg("\u{259e}").unwrap();
        assert!(d.contains(r#"d="M12 0H24V12H12ZM0 12H12V24H0Z""#), "{d}");
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

    /// 破線はセルを本数で割り、間隔の分だけ縮めて引きます。
    #[test]
    fn breaks_a_dashed_line_into_segments() {
        let d = svg("╌").unwrap();
        assert!(d.contains(r#"d="M11 12H1M23 12H13""#), "{d}");
    }

    /// 破線の端は奇数なので、実線とも隣のセルともつながりません。
    #[test]
    fn keeps_dashes_apart_from_their_neighbours() {
        let d = svg("─╌").unwrap();
        assert!(d.contains(r#"d="M24 12H0M35 12H25M47 12H37""#), "{d}");
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
