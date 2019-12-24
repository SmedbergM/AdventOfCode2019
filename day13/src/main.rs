use std::collections::HashMap;
use std::fmt;
use std::time;
use std::thread;

use intcode;

fn sleep_one_second() {
    let one_second = time::Duration::from_millis(1000/72);
    thread::sleep(one_second)
}

enum Error {
    IllegalStateError
}

enum Tile {
    Empty,
    Wall,
    Block,
    HorizontalPaddle,
    Ball
}

impl Tile {
    fn from_int(id: i64) -> Option<Tile> {
        match id {
            0 => Some(Tile::Empty),
            1 => Some(Tile::Wall),
            2 => Some(Tile::Block),
            3 => Some(Tile::HorizontalPaddle),
            4 => Some(Tile::Ball),
            _ => None
        }
    }

    fn chr(&self) -> char {
        match self {
            Tile::Empty => ' ',
            Tile::Wall => '#',
            Tile::Block => 'K',
            Tile::HorizontalPaddle => '_',
            Tile::Ball => 'o'
        }
    }
}

struct Game {
    tiles: HashMap<(i64, i64), Tile>,
    score: i64,
    game_over: bool
}

impl Game {

    fn empty() -> Game {
        Game {
            tiles: HashMap::new(),
            score: 0,
            game_over: false
        }
    }

    fn ball_and_paddle_pos(&self) -> Option<(i64, i64)> { // returns the x-value of ball and paddle
        let mut ball_pos = None;
        let mut paddle_pos = None;
        for (xy, tile) in &self.tiles {
            match tile {
                Tile::Ball => {
                    ball_pos = Some(xy.0)
                },
                Tile::HorizontalPaddle => {
                    paddle_pos = Some(xy.0)
                },
                _ => ()
            }
        }

        ball_pos.and_then(|ball| paddle_pos.map(|paddle| (ball, paddle)))
    }
}

impl fmt::Display for Game {
    fn fmt(&self, writer: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let (xmin, xmax, ymin, ymax) = self.tiles.iter().fold((i64::max_value(), i64::min_value(), i64::max_value(), i64::min_value()), |(xmin, xmax, ymin, ymax),((x,y), _)| {
            (i64::min(xmin, *x), i64::max(xmax, *x), i64::min(ymin, *y), i64::max(ymax, *y))
        });

        let mut repr = String::new();
        for y in ymin..=ymax {
            for x in xmin..=xmax {
                repr.push(self.tiles.get(&(x,y)).map(|t| t.chr()).unwrap_or(' '));
            }
            repr.push('\n');
        }
        repr.push_str(&format!("Score: {}", &self.score));

        write!(writer, "{}", &repr)
    }
}

fn play_single_move(game: &mut Game, program: &mut intcode::Program) -> Option<Error> {
    loop {
        let state1 = program.await_output();
        match state1 {
            intcode::State::AwaitingInput => return None,
            intcode::State::Done => {
                game.game_over = true;
                return None
            },
            intcode::State::Crashed => {
                game.game_over = true;
                eprintln!("Intcode program crashed!");
                return Some(Error::IllegalStateError)
            },
            intcode::State::Running => {
                eprintln!("await_output() returned State::Running, this should never happen")
            },
            intcode::State::Output(x) | intcode::State::OutputAwaitingInput(x) => {
                let state2 = program.await_output();
                match state2 {
                    intcode::State::AwaitingInput | intcode::State::Crashed | intcode::State::Done => {
                        eprintln!("Program behaved unexpectedly!");
                        game.game_over = true;
                        return Some(Error::IllegalStateError)
                    },
                    intcode::State::Running => {
                        eprintln!("await_output() returned State::Running, this should never happen")
                    },
                    intcode::State::Output(y) | intcode::State::OutputAwaitingInput(y) => {
                        let state3 = program.await_output();
                        match state3 {
                            intcode::State::AwaitingInput | intcode::State::Crashed | intcode::State::Done => {
                                eprintln!("Program behaved unexpectedly!");
                                game.game_over = true;
                                return Some(Error::IllegalStateError)
                            },
                            intcode::State::Running => {
                                eprintln!("await_output() returned State::Running, this should never happen")
                            },
                            intcode::State::Output(tile_code) | intcode::State::OutputAwaitingInput(tile_code) => {
                                match (x,y) {
                                    (-1, 0) => game.score = tile_code,
                                    _ => if let Some(tile) = Tile::from_int(tile_code) {
                                        game.tiles.insert((x,y), tile);
                                    } else {
                                        eprintln!("{} does not code a valid tile type at ({},{})", tile_code, x, y)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

enum PlayerInput {
    Left, Right, Neutral
}

impl PlayerInput {
    fn parse(line: &str) -> Option<PlayerInput> {
        match line {
            "a" => Some(PlayerInput::Left),
            "s" => Some(PlayerInput::Neutral),
            "d" => Some(PlayerInput::Right),
            _ => None
        }
    }

    fn to_int(&self) -> i64 {
        match self {
            PlayerInput::Neutral => 0,
            PlayerInput::Left => -1,
            PlayerInput::Right => 1
        }
    }
}

fn main() {
    let line = util::read_single_line_from_stdin().unwrap();
    let mut program = intcode::Program::from_str(&line);
    let mut game = Game::empty();

    play_single_move(&mut game, &mut program);

    let mut block_count = 0;
    for (_, tile) in game.tiles.iter() {
        if let Tile::Block = tile {
            block_count += 1;
        }
    }
    println!("Block count: {}", &block_count);
    // println!("{}", &game);

    // Put some quarters in the machine
    let pat = regex::Regex::new(r"\d+,").unwrap();    
    let line2 = pat.replace(&line, "2,");
    let mut program2 = intcode::Program::from_str(&line2);
    let mut game2 = Game::empty();
    play_single_move(&mut game2, &mut program2);
    
    while !game2.game_over {
        if let Some((ball_x, paddle_x)) = game2.ball_and_paddle_pos() {
            if ball_x < paddle_x {
                program2.read_input(PlayerInput::Left.to_int());
            } else if ball_x > paddle_x {
                program2.read_input(PlayerInput::Right.to_int());
            } else {
                program2.read_input(PlayerInput::Neutral.to_int());
            }

            play_single_move(&mut game2, &mut program2);
            println!("{}", &game2);
            sleep_one_second()
        } else {
            eprintln!("Unable to read ball/paddle position from game!");
        }
    }
}
