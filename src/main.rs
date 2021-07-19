use std::collections::HashSet;
use std::io::prelude::*;
mod parse;
use parse::*;

use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{self, style, Color},
    terminal,
    tty::IsTty,
    ExecutableCommand, QueueableCommand,
};
use std::io::{BufRead, Write};

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
    // println!("BOXES {:#?}", bs);
    // println!("EDGES {:#?}", es);

    let dres = ct_draw(&lines, es, bs);
    ct_rst().and(dres).expect("term issues");
}

fn ct_rst() -> crossterm::Result<()> {
    terminal::disable_raw_mode().and(
        std::io::stdout()
            .execute(terminal::LeaveAlternateScreen)
            .map(|_| {}),
    )
}

macro_rules! keyevt {
    ($code:literal, $modifier:ident) => {
        Event::Key(KeyEvent {
            code: KeyCode::Char($code),
            modifiers: KeyModifiers::$modifier,
        })
    };
    ($code:literal) => {
        keyevt!($code, NONE)
    };
}

fn move_inside(inner: TBox, outer: TBox, d: Direction) -> Option<TBox> {
    let border = border_in_dir(outer, d);
    if !inner.intersects(border) {
        return inner.in_dir(d);
    }
    return None;
}

fn ct_draw(lines: &Lines, es: HashSet<Vec<Point>>, bs: Vec<TBox>) -> crossterm::Result<()> {
    let mut stdout = std::io::stdout();
    if !stdout.is_tty() {
        stdout.execute(style::Print("not a tty"))?;
        return Ok(());
    }
    stdout.execute(terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let (mut pr, mut pc): (u16, u16) = (0, 0);
    let (mut vr, mut vc): (usize, usize) = (0, 0);
    let mut dirty = true;
    let (mut cols, mut rows) = terminal::size()?;
    use std::cmp::{max, min};

    // you have
    // - position on screen (pr, pc),
    // - screen dim (w, h)
    //                         pc
    //    ,------. .        ,--->
    //    |   #  |  } h  pr v   #
    //    '------' '
    //    '--.---'
    //       w
    // - position of view (vr, vc)
    // - max lines (n) and cur line len (m[pr+vr])
    //        vc
    //        ->
    //      | +~~~~
    //   vr | |~~
    //      | |~
    //      v |,------.
    //        ||~~~#  |
    //        |'------'
    //        |~
    //        +~
    //
    // relative movement
    // to move in a direction d (v, z), you have 3 options
    //   1. can move local: (pr, pc) += (v, z)
    //   2. can scroll: (vr, vc) += (v, z)
    //   3. can't move (must extend)
    // for each direction
    //   up (-1, 0) => if pr > 0 { 1. } elif vr > 0 { 2. } else 3.
    //   dn (1, 0)  => if pr < h { 1. } elif vr < n { 2. } else 3.
    //   rt (1, 0)  => if pc < w { 1. } elif vc < m[_] { 2. } else 3.
    //   lt (-1, 0) => if pc > 0 { 1. } elif vc > 0 { 2. } else 3.

    let mut move_debug = String::new();
    let mut focus : Option<TBox> = None;
    loop {
        if dirty {
            stdout.queue(cursor::MoveTo(0, 0))?;
            stdout.queue(terminal::Clear(terminal::ClearType::All))?;
            let cursor = Point {
                row: pr as usize + vr,
                col: pc as usize + vc,
            };
            for row in 0..min(rows as usize, lines.0.len()) {
                for col in 0..min(cols as usize, lines.0[vr + row].len()) {
                    let p = Point {
                        row: vr + row,
                        col: vc + col,
                    };
                    if es.iter().any(|e| path_contains(e, p)) {
                        stdout.queue(style::PrintStyledContent(
                            style(lines[p]).with(Color::Yellow),
                        ))?;
                    } else if focus.iter().any(|b| b.contains(p)) {
                        stdout
                            .queue(style::PrintStyledContent(style(lines[p]).with(Color::White)))?;
                    } else if bs.iter().any(|b| b.contains(p)) {
                        stdout.queue(style::PrintStyledContent(
                            style(lines[p]).with(Color::Blue),
                        ))?;
                    } else {
                        stdout.queue(style::Print(lines[p]))?;
                    }
                }
                stdout.queue(cursor::MoveToNextLine(1))?;
            }
            stdout.queue(cursor::MoveTo(pc, pr))?;
            stdout.flush()?;
            dirty = false;
        }

        stdout
            .queue(cursor::MoveTo(0, rows))?
            .queue(style::PrintStyledContent(
                style(format!(
                    "{} {} p: {:?} v: {:?} {:?} | {}",
                    vr + pr as usize,
                    vc + pc as usize,
                    (pr, pc),
                    (vr, vc),
                    (rows, cols),
                    move_debug
                ))
                .on(Color::DarkBlue),
            ))?
            .queue(cursor::MoveTo(pc, pr))?
            .flush()?;

        let mut move_cursor = |d: Direction| {
            move_debug = String::new();
            let p = Point::from((pr as usize, pc as usize));
            let pbox = TBox::from((p, p));
            let vbox = TBox::from((
                Point::from((0, 0)),
                Point::from((rows as usize - 1, cols as usize - 1)),
            ));
            if let Some(b) = move_inside(pbox, vbox, d) {
                pr = b.0.row as u16;
                pc = b.0.col as u16;
                return;
            }

            let vbox = TBox::from((
                Point::from((vr, vc)),
                Point::from((vr + rows as usize - 1, vr + cols as usize - 1)),
            ));
            let windowed_max = lines.0[vr..(vr + rows as usize)]
                .iter()
                .fold(0, |lmax, l| max(lmax, l.len()));
            let lbox = TBox::from((
                Point::from((0, 0)),
                Point::from((lines.0.len() - 1, windowed_max)),
            ));
            move_debug = format!("{} | {:?} {:?} {:?}", move_debug, pbox, vbox, lbox);
            if let Some(b) = move_inside(vbox, lbox, d) {
                vr = b.0.row;
                vc = b.0.col;
                dirty = true;
            }
        };
        match read()? {
            keyevt!('q') | keyevt!('c', CONTROL) => return Ok(()),
            keyevt!('j') => move_cursor(Direction::Dn),
            keyevt!('k') => move_cursor(Direction::Up),
            keyevt!('l') => move_cursor(Direction::Rt),
            keyevt!('h') => move_cursor(Direction::Lt),
            Event::Resize(c, r) => {
                cols = c;
                rows = r;
                pr = min(rows, pr);
                pc = min(cols, pc);
                dirty = true;
            }
            _ => {}
        }

        let nextfocus = bs.iter().find(|b1| b1.contains(Point::from((pr as usize + vr, pc as usize + vc)))).map(|b| *b);
        dirty |= nextfocus != focus;
        focus = nextfocus;
    }
}

fn set_style<'s>(prev: &'s str, next: &'s str) -> &'s str {
    if prev == next {
        return prev;
    }
    print!("{}", next);
    next
}

fn simp_draw(lines: &Lines, es: HashSet<Vec<Point>>, bs: Vec<TBox>) {
    let rst = "\x1b[0m";
    let mut style = rst;
    for r in 0..lines.0.len() {
        for c in 0..lines.0[r].len() {
            let p = Point::from((r, c));
            if es.iter().any(|e| path_contains(e, p)) {
                style = set_style(style, "\x1b[33m");
            } else if bs.iter().any(|b| b.contains(p)) {
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
    #[test]
    fn test_move_inside() {
        let pbox = TBox(Point { row: 0, col: 0 }, Point { row: 0, col: 0 });
        let vbox = TBox(Point { row: 0, col: 0 }, Point { row: 1, col: 1 });

        assert_eq!(move_inside(pbox, vbox, Direction::Up), None);
        assert_eq!(move_inside(pbox, vbox, Direction::Lt), None);
        assert_eq!(
            move_inside(pbox, vbox, Direction::Rt),
            Some(TBox(Point { row: 0, col: 1 }, Point { row: 0, col: 1 }))
        );
        assert_eq!(
            move_inside(pbox, vbox, Direction::Dn),
            Some(TBox(Point { row: 1, col: 0 }, Point { row: 1, col: 0 }))
        );
        let pbox = move_inside(pbox, vbox, Direction::Dn).unwrap();
        assert_eq!(move_inside(pbox, vbox, Direction::Dn), None);
    }
}
