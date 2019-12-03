use std::io;
use std::io::prelude::*;
use std::collections::{HashSet, HashMap};

extern crate regex;
use regex::{Regex};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
struct XY {
    x: i32,
    y: i32
}

impl XY {
    fn xy(x: i32, y: i32) -> XY {
        XY {x, y}
    }

    fn abs(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }

    fn up(&self) -> XY {
        XY::xy(self.x, self.y + 1)
    }
    fn down(&self) -> XY {
        XY::xy(self.x, self.y - 1)
    }
    fn right(&self) -> XY {
        XY::xy(self.x + 1, self.y)
    }
    fn left(&self) -> XY {
        XY::xy(self.x - 1, self.y)
    }
}

struct PathIter<'a> {
    current_segment: Segment, // Segment of moves which have not yet been made
    current_xy: XY, // XY which has already been yielded, or (0,0) initially
    remaining_segments: &'a[Segment]
}

impl Iterator for PathIter<'_> {
    type Item = XY;

    fn next(&mut self) -> Option<XY> {
        if self.current_segment.len() == 0 && self.remaining_segments.is_empty() {
            None
        } else if self.current_segment.len() == 0 {
            self.current_segment = self.remaining_segments[0].clone();
            self.remaining_segments = &self.remaining_segments[1..];
            self.next()
        } else {
            match self.current_segment {
                Segment::Up(k) => {
                    self.current_xy = self.current_xy.up();
                    self.current_segment = Segment::Up(k - 1);
                    Some(self.current_xy.clone())
                },
                Segment::Down(k) => {
                    self.current_xy = self.current_xy.down();
                    self.current_segment = Segment::Down(k - 1);
                    Some(self.current_xy.clone())
                },
                Segment::Left(k) => {
                    self.current_xy = self.current_xy.left();
                    self.current_segment = Segment::Left(k - 1);
                    Some(self.current_xy.clone())
                },
                Segment::Right(k) => {
                    self.current_xy = self.current_xy.right();
                    self.current_segment = Segment::Right(k - 1);
                    Some(self.current_xy.clone())
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Segment {
    Up(u16),
    Down(u16),
    Right(u16),
    Left(u16)
}

impl Segment {
    fn len(&self) -> u16 {
        match self {
            Segment::Up(x) => *x,
            Segment::Down(x) => *x,
            Segment::Right(x) => *x,
            Segment::Left(x) => *x
        }
    }
}

struct Path {
    segments: Vec<Segment>
}

impl Path {
    fn from_str(line: &str) -> Path {

        let pat = Regex::new(r"([UDLR])(\d+)").unwrap();
        let mut segments = Vec::new();
        for s in line.split(",") {
            match pat.captures(&s) {
                None => {
                    eprintln!("No segment parseable for {}", &s)
                },
                Some(cap) => {
                    let opt_segment = cap.get(1).and_then(|d| {
                        let opt_k = cap.get(2).and_then(|m2| u16::from_str_radix(&m2.as_str(), 10).ok());
                        match d.as_str() {
                            "U" => opt_k.map(|k| Segment::Up(k)),
                            "D" => opt_k.map(|k| Segment::Down(k)),
                            "L" => opt_k.map(|k| Segment::Left(k)),
                            "R" => opt_k.map(|k| Segment::Right(k)),
                            _ => {
                                eprintln!("No direction parseable for {}", d.as_str());
                                None
                            }
                        }
                    });
                    for segment in opt_segment {
                        segments.push(segment)
                    }
                }
            }
        };
        Path { segments }
    }

    fn xys(&self) -> PathIter {
        PathIter {
            current_segment: Segment::Up(0),
            current_xy: XY::xy(0,0),
            remaining_segments: &self.segments[..]
        }
    }
}

fn intersect(p1: &Path, p2: &Path) -> Option<XY> {
    let mut xys1 = HashSet::new();
    let mut xys2 = HashSet::new();
    for xy in p1.xys() {
        xys1.insert(xy);
    }
    for xy in p2.xys() {
        xys2.insert(xy);
    }
    let xys_both = xys1.intersection(&xys2);
    xys_both.min_by_key(|xy| xy.abs()).map(|xy| xy.clone())
}

fn intersect_delay(p1: &Path, p2: &Path) -> Option<(XY, usize)> {
    let mut delay_1 = HashMap::new();
    let mut delay_2 = HashMap::new();

    for (idx, xy) in p1.xys().enumerate() {
        if !delay_1.contains_key(&xy) {
            delay_1.insert(xy, idx + 1);
        }
    };

    for (idx, xy) in p2.xys().enumerate() {
        if !delay_2.contains_key(&xy) {
            delay_2.insert(xy, idx + 1);
        }
    };

    let mut best: Option<(XY, usize)> = None;
    for (xy, delay1) in delay_1 {
        if let Some(delay2) = delay_2.get(&xy) {
            match best {
                None => best = Some((xy, delay1 + delay2)),
                Some((_, ref prev_delay)) if delay1 + delay2 < *prev_delay => {
                    best = Some((xy, delay1 + delay2))
                },
                _ => ()
            }
        }
    };
    best
}

fn main() {
    let stdin = io::stdin();
    let mut stdin_lines = stdin.lock().lines();
    let line_p1 = stdin_lines.next().unwrap().unwrap();
    let line_p2 = stdin_lines.next().unwrap().unwrap();
    let path1 = Path::from_str(&line_p1);
    let path2 = Path::from_str(&line_p2);
    let cross_point = intersect(&path1, &path2);
    match cross_point {
        None => eprintln!("No crossing point found!"),
        Some(xy) => {
            println!("Crossing point found at (x,y) = ({},{}), norm: {}", xy.x, xy.y, xy.abs())
        }
    };
    let cross_point2 = intersect_delay(&path1, &path2);
    match cross_point2 {
        None => eprintln!("No crossing point (delay) found"),
        Some((xy, delay)) => {
            println!("Crossing point found at ({},{}), delay: {}", xy.x, xy.y, delay)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterator_spec() {
        let path = Path::from_str("R8,U5,L5,D3");
        let xys: Vec<XY> = path.xys().collect();
        assert_eq!(xys[..], [
            XY::xy(1,0),
            XY::xy(2,0),
            XY::xy(3,0),
            XY::xy(4,0),
            XY::xy(5,0),
            XY::xy(6,0),
            XY::xy(7,0),
            XY::xy(8,0),
            XY::xy(8,1),
            XY::xy(8,2),
            XY::xy(8,3),
            XY::xy(8,4),
            XY::xy(8,5),
            XY::xy(7,5),
            XY::xy(6,5),
            XY::xy(5,5),
            XY::xy(4,5),
            XY::xy(3,5),
            XY::xy(3,4),
            XY::xy(3,3),
            XY::xy(3,2)
        ]);
    }

    #[test]
    fn intersect_spec() {
        let path1 = Path::from_str("R75,D30,R83,U83,L12,D49,R71,U7,L72");
        let path2 = Path::from_str("U62,R66,U55,R34,D71,R55,D58,R83");
        let xy = intersect(&path1, &path2).unwrap();
        assert_eq!(xy.x + xy.y, 159);

        let path1 = Path::from_str("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51");
        let path2 = Path::from_str("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7");
        let xy = intersect(&path1, &path2).unwrap();
        assert_eq!(xy.x + xy.y, 135)
    }

    #[test]
    fn intersect_delay_spec() {
        let path1 = Path::from_str("R75,D30,R83,U83,L12,D49,R71,U7,L72");
        let path2 = Path::from_str("U62,R66,U55,R34,D71,R55,D58,R83");
        let (_, delay) = intersect_delay(&path1, &path2).unwrap();
        assert_eq!(delay, 610);

        let path1 = Path::from_str("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51");
        let path2 = Path::from_str("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7");
        let (_, delay) = intersect_delay(&path1, &path2).unwrap();
        assert_eq!(delay, 410)
    }
}