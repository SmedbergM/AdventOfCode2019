use std::collections::{HashSet};
use intcode;


enum Heading {
    North, South, East, West
}

impl Heading {
    fn turn(&self, left: bool) -> Heading {
        match (self, left) {
            (Heading::North, true) | (Heading::South, false) => Heading::West,
            (Heading::East, true) | (Heading::West, false) => Heading::North,
            (Heading::South, true) | (Heading::North, false) => Heading::East,
            (Heading::West, true) | (Heading::East, false) => Heading::South
        }
    }
}

struct XY {
    x: i32,
    y: i32
}

impl XY {
    fn to_pair(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    fn incr(&mut self, heading: &Heading) {
        match heading {
            Heading::North => self.y += 1,
            Heading::South => self.y -= 1,
            Heading::East => self.x += 1,
            Heading::West => self.x -= 1
        };
    }
}

struct Canvas { // bundle together the surface to be painted and the robot
    program: intcode::Program,
    white: HashSet<(i32, i32)>, // all start black, so white starts empty
    xy: XY,
    heading: Heading
}

impl Canvas {
    fn new(program: intcode::Program) -> Canvas {
        Canvas { program, white: HashSet::new(), xy: XY { x: 0, y: 0 }, heading: Heading::North }
    }

    fn print(&self) {
        let (xmin, xmax, ymin, ymax) = self.white.iter().fold((0,0,0,0), |(xmin, xmax, ymin, ymax), (x, y)| {
            (i32::min(xmin, *x), i32::max(xmax, *x), i32::min(ymin, *y), i32::max(ymax, *y))
        });

        let mut r = String::new();
        for y in ymin..=ymax {
            for x in xmin..=xmax {
                if self.white.contains(&(x,y)) {
                    r.push('#')
                } else {
                    r.push(' ')
                }
            };
            r.push('\n');
        }
        r.pop();

        println!("{}", &r);
    }

    fn count_painted_squares(&mut self) -> usize {
        let mut p: HashSet<(i32, i32)> = HashSet::new();
        loop {
            let current_white = self.white.contains(&self.xy.to_pair());
            self.program.read_input(current_white as i64);

            let output = self.program.await_output(&mut |_| {});
            match output {
                Some(1) => {
                    self.white.insert(self.xy.to_pair());
                    p.insert(self.xy.to_pair());
                },
                Some(0) => {
                    self.white.remove(&self.xy.to_pair());
                    p.insert(self.xy.to_pair());
                },
                None => break,
                Some(x) => eprintln!("Unexpected output {} from intcode!", x)
            };

            let output = self.program.await_output(&mut |_| {});
            match output {
                Some(x) => {
                    let next_heading = self.heading.turn(x == 0);
                    self.xy.incr(&next_heading);
                    self.heading = next_heading;
                },
                None => break
            }
        };
        p.len()
    }
}

fn main() {
    let line = util::read_single_line_from_stdin().unwrap();
    let program = intcode::Program::from_str(&line);
    let mut canvas = Canvas::new(program.clone());

    let painted_squares = canvas.count_painted_squares();

    println!("My robot visited {} squares.", painted_squares);
    canvas.print();

    let mut canvas2 = Canvas::new(program.clone());
    canvas2.white.insert((0,0));
    let painted_squares2 = canvas2.count_painted_squares();
    println!("When started on white, my robot visited {} squares.", painted_squares2);
    canvas2.print();
}

#[cfg(test)]
mod tests {

}