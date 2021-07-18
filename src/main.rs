use std::collections::HashSet;
use std::io::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    row: u32,
    col: u32,
}

struct Lines(Vec<Vec<char>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Dn,
    Lt,
    Rt,
}

impl Direction {
    const VALUES: [Direction; 4] = [Direction::Up, Direction::Dn, Direction::Lt, Direction::Rt];

    #[inline]
    fn neg(self) -> Direction {
        use Direction::*;
        match self {
            Up => Dn,
            Dn => Up,
            Lt => Rt,
            Rt => Lt,
        }
    }
}

#[inline]
fn can_go(c: char, d: Direction) -> bool {
    use Direction::*;
    match (c, d) {
        ('|', Up | Dn) => true,
        ('-', Lt | Rt) => true,
        ('.', Dn | Lt) => true,
        (',', Dn | Rt) => true,
        ('\'', Up | Lt | Rt) => true,
        ('<', Rt) => true,
        ('>', Lt) => true,
        _ => false,
    }
}

impl Point {
    #[inline]
    fn in_dir(self, d: Direction) -> Option<Point> {
        use Direction::*;
        match d {
            Dn => Some(Point {
                row: self.row + 1,
                col: self.col,
            }),
            Up => {
                if self.row == 0 {
                    None
                } else {
                    Some(Point {
                        row: self.row - 1,
                        col: self.col,
                    })
                }
            }
            Rt => Some(Point {
                row: self.row,
                col: self.col + 1,
            }),
            Lt => {
                if self.col == 0 {
                    None
                } else {
                    Some(Point {
                        row: self.row,
                        col: self.col - 1,
                    })
                }
            }
        }
    }
}

impl Lines {
    fn at(&self, p: Point) -> Option<char> {
        if p.row as usize >= self.0.len() {
            return None;
        }
        let line = &self.0[p.row as usize];
        if p.col as usize >= line.len() {
            return None;
        }
        Some(line[p.col as usize])
    }

    fn in_dir(&self, p: Point, d: Direction) -> Option<(Point, char)> {
        p.in_dir(d).and_then(|p| self.at(p).map(|c| (p, c)))
    }

    fn visit(&self, mut pred: impl FnMut(Point, char)) {
        for r in 0..self.0.len() {
            for c in 0..self.0[r].len() {
                pred(
                    Point {
                        row: r as u32,
                        col: c as u32,
                    },
                    self.0[r][c],
                );
            }
        }
    }
}

fn top_lefts(lines: &Lines) -> Vec<(Point, char)> {
    let mut ret = vec![];
    for row in 0..lines.0.len() {
        for col in 0..lines.0[row].len() {
            let c = lines.0[row][col];
            let p = Point {
                row: row as u32,
                col: col as u32,
            };
            if can_go(c, Direction::Dn)
                && can_go(c, Direction::Rt)
                && lines
                    .in_dir(p, Direction::Rt)
                    .map(|(_, c)| can_go(c, Direction::Lt))
                    .unwrap_or(false)
                && lines
                    .in_dir(p, Direction::Dn)
                    .map(|(_, c)| can_go(c, Direction::Up))
                    .unwrap_or(false)
            {
                ret.push((p, c));
            }
        }
    }
    ret
}

fn scan_dir(lines: &Lines, mut p: Point, d: Direction) -> Option<(Point, char)> {
    while let Some((q, c)) = lines.in_dir(p, d) {
        //  p
        // --* < can't connect
        //
        if !can_go(c, d.neg()) {
            return lines.at(p).map(|c| (p, c));
        }
        p = q;
        //   p
        // --. < can connect, can't continue
        //
        if !can_go(c, d) {
            return Some((p, c));
        }
    }
    lines.at(p).map(|c| (p, c))
}

struct PathIter<'l> {
    start: bool,
    lines: &'l Lines,
    p: Point,
    d: Direction,
}

impl<'l> PathIter<'l> {
    fn new(lines: &'l Lines, p: Point, d: Direction) -> PathIter<'l> {
        PathIter {
            start: true,
            lines: lines,
            p: p,
            d: d,
        }
    }
}

//       * 4
//   1 2 |
//  |----' 3
//
// 1. start, returns point, begins path-scan
// 2. edge, while current can send, and next can recv, advance cursor
// 3. turn, return point, find next direction (if you can)
// 4. end, current can't send or next can't recv, return final point (if not already returned)
// 5. exit, same as end, but signal end of iteration
//

//
// * > point and direction
//
// 0. test if point exists
// 1. test if you can go that direction
// 2. if so, scan in that direction (returns last point *after* initial, character)
//    2a. mark last point as path point
// 3. if not, pick a direction you haven't tried, go back to 1.
impl<'l> Iterator for PathIter<'l> {
    type Item = Point;
    fn next(&mut self) -> Option<Self::Item> {
        if self.lines.at(self.p).is_none() {
            return None;
        } else if self.start {
            self.start = false;
            return Some(self.p);
        }

        let mut cant_go = vec![self.d.neg()];
        loop {
            println!("PathIter {{ p: {:?}, d: {:?} }}", self.p, self.d);
            if let (Some(true), Some(true)) = (
                self.lines.at(self.p).map(|c| can_go(c, self.d)),
                self.lines
                    .in_dir(self.p, self.d)
                    .map(|(_, c)| can_go(c, self.d.neg())),
            ) {
                if let Some((pnext, c)) = scan_dir(self.lines, self.p, self.d) {
                    println!("scan_dir = Some(({:?}, {:?}))", pnext, c);
                    self.p = pnext;
                    return Some(pnext);
                }
            }

            cant_go.push(self.d);
            if let Some(dnext) = Direction::VALUES
                .into_iter()
                .filter(|d| !cant_go.contains(d))
                .next()
            {
                self.d = dnext;
                continue;
            } else {
                return None;
            }
        }
    }
}

fn scan_path(lines: &Lines, p: Point, d: Direction) -> Vec<Point> {
    if !lines.at(p).map(|c| can_go(c, d)).unwrap_or(false) {
        return vec![];
    }
    let mut ret = vec![];
    let mut it = PathIter::new(&lines, p, d);
    while let Some(next) = it.next() {
        if ret.contains(&next) {
            return ret;
        }
        ret.push(next);
    }
    ret
}

fn boxes(lines: &Lines) -> Vec<(Point, Point)> {
    top_lefts(lines)
        .into_iter()
        .map(|tl| {
            let tr = scan_dir(lines, tl.0, Direction::Rt)?;
            let bl = scan_dir(lines, tl.0, Direction::Dn)?;
            let br = scan_dir(lines, bl.0, Direction::Rt)?;
            let br2 = scan_dir(lines, tr.0, Direction::Dn)?;
            if br2 != br {
                return None;
            }
            Some((tl.0, br.0))
        })
        .fold(vec![], |mut acc, b| match b {
            Some(b) => {
                acc.push(b);
                acc
            }
            _ => acc,
        })
}

fn border(bounds: (Point, Point)) -> Vec<(Point, Direction)> {
    let mut ret = Vec::with_capacity(
        2 * ((bounds.1.row - bounds.0.row) + (bounds.1.col - bounds.0.col)) as usize,
    );
    let (col0, row0) = (bounds.0.col > 0, bounds.0.row > 0);
    if row0 {
        for i in bounds.0.col..=bounds.1.col {
            ret.push((
                Point {
                    row: bounds.0.row - 1,
                    col: i,
                },
                Direction::Up,
            ));
        }
    }
    if col0 {
        for i in bounds.0.row..=bounds.1.row {
            ret.push((
                Point {
                    row: i,
                    col: bounds.0.col - 1,
                },
                Direction::Lt,
            ));
        }
    }
    for i in bounds.0.row..=bounds.1.row {
        ret.push((
            Point {
                row: i,
                col: bounds.1.col + 1,
            },
            Direction::Dn,
        ));
    }
    for i in bounds.0.col..=bounds.1.col {
        ret.push((
            Point {
                row: bounds.1.row + 1,
                col: i,
            },
            Direction::Rt,
        ));
    }
    ret
}

#[inline]
fn norm_box(b: (Point, Point)) -> (Point, Point) {
    use std::cmp::{max, min};
    (
        Point {
            row: min(b.0.row, b.1.row),
            col: min(b.0.col, b.1.col),
        },
        Point {
            row: max(b.0.row, b.1.row),
            col: max(b.0.col, b.1.col),
        },
    )
}

#[inline]
fn box_contains(b: (Point, Point), p: Point) -> bool {
    p.row >= b.0.row && p.row <= b.1.row && p.col >= b.0.col && p.col <= b.1.col
}

#[inline]
fn box_collides_box(b0: (Point, Point), b1: (Point, Point)) -> bool {
    !(b0.1.row < b1.0.row || b0.0.row > b1.1.row || b0.1.col < b1.0.col || b0.0.col > b1.1.col)
}

fn path_contains(pth: &Vec<Point>, p: Point) -> bool {
    let mut it = pth.iter();
    let fst = it.next();
    if !fst.is_some() {
        return false;
    }
    let mut last = fst.unwrap();
    if *last == p {
        return true;
    }
    while let Some(next) = it.next() {
        if box_contains(norm_box((*last, *next)), p) {
            return true;
        }
        last = next;
    }
    false
}

fn edges(lines: &Lines, boxes: &Vec<(Point, Point)>) -> HashSet<Vec<Point>> {
    let mut buf = Vec::with_capacity(lines.0.len());
    for r in 0..lines.0.len() {
        buf.push(Vec::with_capacity(lines.0[r].len()));
        for _ in 0..lines.0[r].len() {
            buf[r].push(' ');
        }
    }
    lines.visit(|p, c| {
        for b in boxes {
            if box_contains(*b, p) {
                buf[p.row as usize][p.col as usize] = '#';
            }
            if buf[p.row as usize][p.col as usize] != ' ' {
                return;
            }
            if can_go(c, Direction::Up)
                && box_contains(
                    (
                        Point {
                            row: b.1.row + 1,
                            col: b.0.col,
                        },
                        Point {
                            row: b.1.row + 1,
                            col: b.1.col,
                        },
                    ),
                    p,
                )
            {
                buf[p.row as usize][p.col as usize] = 'v';
            } else if can_go(c, Direction::Lt)
                && box_contains(
                    (
                        Point {
                            row: b.0.row,
                            col: b.1.col + 1,
                        },
                        Point {
                            row: b.1.row,
                            col: b.1.col + 1,
                        },
                    ),
                    p,
                )
            {
                buf[p.row as usize][p.col as usize] = '>';
            } else if can_go(c, Direction::Dn)
                && b.0
                    .in_dir(Direction::Up)
                    .map(|p0| {
                        box_contains(
                            (
                                p0,
                                Point {
                                    row: p0.row,
                                    col: b.1.col,
                                },
                            ),
                            p,
                        )
                    })
                    .unwrap_or(false)
            {
                buf[p.row as usize][p.col as usize] = '^';
            } else if can_go(c, Direction::Rt)
                && b.0
                    .in_dir(Direction::Lt)
                    .map(|p0| {
                        box_contains(
                            (
                                p0,
                                Point {
                                    row: b.1.row,
                                    col: p0.col,
                                },
                            ),
                            p,
                        )
                    })
                    .unwrap_or(false)
            {
                buf[p.row as usize][p.col as usize] = '<';
            }
        }
    });
    println!(
        "MAP[{},_]\n{}",
        buf.len(),
        buf.iter()
            .map(|l| l.iter().collect::<String>() + "\n")
            .collect::<String>()
    );
    //   ###
    //  ,---. ##
    // #|   |,--.  find all possible starts for edges between boxes
    //  '---''--'
    //   ###  ##
    boxes
        .iter()
        .map(|b| border(*b))
        .flat_map(|v| v.into_iter())
        .filter(|(p, d)| lines.at(*p).map(|c| can_go(c, d.neg())).unwrap_or(false))
        .map(|(p, d)| scan_path(lines, p, dbg!(p, d).1))
        .filter(|pth| pth.len() > 0)
        .fold(HashSet::new(), |mut map, mut pth| {
            if !map.contains(&pth) {
                pth.reverse();
                map.insert(pth);
            }
            map
        })
}

fn set_style<'s>(prev: &'s str, next: &'s str) -> &'s str {
    if prev == next {
        return prev;
    }
    print!("{}", next);
    next
}

fn main() {
    let stdin = std::io::stdin();
    let stdin = stdin.lock();
    let buf: Vec<Vec<char>> = stdin
        .lines()
        .map(|l| l.unwrap().chars().collect())
        .collect();
    let lines = Lines(buf);
    let bs = boxes(&lines);
    let es = edges(&lines, &bs);
    println!("BOXES {:?}", bs);
    println!("EDGES {:?}", es);
    let rst = "\x1b[0m";
    let mut style = rst;
    for r in 0..lines.0.len() {
        for c in 0..lines.0[r].len() {
            let p = Point {
                row: r as u32,
                col: c as u32,
            };
            if es.iter().any(|e| path_contains(e, p)) {
                style = set_style(style, "\x1b[33m");
            } else if bs.iter().any(|b| box_contains(*b, p)) {
                style = set_style(style, "\x1b[34m");
            } else {
                style = set_style(style, rst);
            }
            print!("{}", lines.0[r][c]);
        }
        println!();
    }
    set_style(style, rst);
}

#[cfg(test)]
mod test {
    use super::*;

    fn lines() -> Lines {
        let lines: Vec<Vec<char>> = r#"
 ,---.,-----------.
 |   |',-.        |
 |   | | |  ,-----'
 '---' | |  |
       | |--'
       '-'
"#
        .lines()
        .map(|l| l.chars().collect())
        .collect();
        Lines(lines)
    }

    #[test]
    fn test_top_lefts() {
        let lines = lines();
        assert_eq!(
            vec![
                (Point { row: 1, col: 1 }, ','),
                (Point { row: 1, col: 6 }, ','),
                (Point { row: 2, col: 7 }, ','),
                (Point { row: 3, col: 12 }, ','),
            ],
            top_lefts(&lines)
        );
    }

    #[test]
    fn test_scan_dir() {
        let lines = lines();

        let tl = Point { row: 1, col: 1 };
        let tr = Point { row: 1, col: 5 };
        let bl = Point { row: 4, col: 1 };
        let br = Point { row: 4, col: 5 };
        assert_eq!(Some((tr, '.')), scan_dir(&lines, tl, Direction::Rt),);
        assert_eq!(Some((bl, '\'')), scan_dir(&lines, tl, Direction::Dn),);
        assert_eq!(Some((br, '\'')), scan_dir(&lines, bl, Direction::Rt),);

        assert_eq!(
            Some((Point { row: 1, col: 18 }, '.')),
            scan_dir(&lines, Point { row: 1, col: 6 }, Direction::Rt),
        );
        assert_eq!(
            Some((Point { row: 2, col: 6 }, '\'')),
            scan_dir(&lines, Point { row: 1, col: 6 }, Direction::Dn),
        );
        assert_eq!(
            Some((Point { row: 1, col: 6 }, ',')),
            scan_dir(&lines, Point { row: 1, col: 6 }, Direction::Lt),
        );
    }

    #[test]
    fn test_boxes() {
        let lines = lines();
        assert_eq!(
            vec![
                (Point { row: 1, col: 1 }, Point { row: 4, col: 5 }),
                (Point { row: 2, col: 7 }, Point { row: 6, col: 9 }),
            ],
            boxes(&lines),
        );
    }

    #[test]
    fn test_scan_path() {
        let lines = lines();

        let mut pth = vec![
            Point { row: 2, col: 6 },
            Point { row: 1, col: 6 },
            Point { row: 1, col: 18 },
            Point { row: 3, col: 18 },
            Point { row: 3, col: 12 },
            Point { row: 5, col: 12 },
            Point { row: 5, col: 10 },
        ];

        assert_eq!(pth, scan_path(&lines, pth[0], Direction::Rt),);
        // should work in reverse
        pth.reverse();
        assert_eq!(pth, scan_path(&lines, pth[0], Direction::Rt),);

        // |--' |--'
        //  ^     ^
        // instead of the beginning, start a little aways
        pth[0].col += 1;
        assert_eq!(pth, scan_path(&lines, pth[0], Direction::Rt),);
    }

    #[test]
    fn test_box_contains() {
        let lb = (Point { row: 1, col: 1 }, Point { row: 4, col: 5 });

        assert_eq!(true, box_contains(lb, lb.0) && box_contains(lb, lb.1));
        assert_eq!(false, box_contains(lb, Point { row: 5, col: 4 }),);
    }

    #[test]
    fn test_border() {
        let b = (Point { row: 1, col: 1 }, Point { row: 3, col: 4 });
        use Direction::*;
        assert_eq!(
            vec![
                (Point { row: 0, col: 1 }, Up),
                (Point { row: 0, col: 2 }, Up),
                (Point { row: 0, col: 3 }, Up),
                (Point { row: 0, col: 4 }, Up),
                (Point { row: 1, col: 0 }, Lt),
                (Point { row: 2, col: 0 }, Lt),
                (Point { row: 3, col: 0 }, Lt),
                (Point { row: 1, col: 5 }, Dn),
                (Point { row: 2, col: 5 }, Dn),
                (Point { row: 3, col: 5 }, Dn),
                (Point { row: 4, col: 1 }, Rt),
                (Point { row: 4, col: 2 }, Rt),
                (Point { row: 4, col: 3 }, Rt),
                (Point { row: 4, col: 4 }, Rt)
            ],
            border(b)
        )
    }
}
