use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{CustomType, Select, Text};
use itertools::Itertools;

#[derive(Debug)]
enum Error {
    MaxCapacity,
    NoContent,
    CantMove,
    DiffColor,
    InvalidMove(&'static str),
}

#[derive(Eq, PartialEq, Debug, Clone)]
struct Color {
    name: String,
    r: u8,
    g: u8,
    b: u8,
}

impl From<&Color> for colored::Color {
    fn from(value: &Color) -> Self {
        colored::Color::TrueColor {
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

impl From<Color> for colored::Color {
    fn from(value: Color) -> Self {
        Self::from(&value)
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.name.color(colored::Color::TrueColor {
                r: self.r,
                g: self.g,
                b: self.b
            })
        )
    }
}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.r.hash(state);
        self.g.hash(state);
        self.b.hash(state);
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
struct Tube {
    capacity: usize,
    content: Vec<Color>,
}

impl Tube {
    fn is_full(&self) -> bool {
        self.content.len() >= self.capacity
    }

    fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn is_complete(&self) -> bool {
        self.is_full() && self.content.iter().all(|x| x == &self.content[0])
    }

    fn is_pour_valid(&self, other: &Tube) -> Result<(), Error> {
        if other.is_full() {
            return Err(Error::MaxCapacity);
        }

        if self.is_empty() {
            return Err(Error::NoContent);
        }

        if self.is_complete() || other.is_complete() {
            return Err(Error::CantMove);
        }

        if !other.is_empty() && self.content.last() != other.content.last() {
            return Err(Error::DiffColor);
        }

        Ok(())
    }

    fn pour(&mut self, other: &mut Tube) -> Result<(), Error> {
        self.is_pour_valid(other)?;

        let other_empty_cap = other.capacity - other.content.len();

        let mut temp_content = Vec::with_capacity(other_empty_cap);
        while let Some(pour_color) = self.content.pop() {
            if temp_content.len() >= other_empty_cap
                || (!temp_content.is_empty() && temp_content[0] != pour_color)
            {
                self.content.push(pour_color);
                break;
            }
            temp_content.push(pour_color);
        }

        other.content.append(&mut temp_content);
        Ok(())
    }

    fn entropy(&self) -> f64 {
        // let color_groups = split_continuous(&self.content);
        // let color_probs = color_groups.iter().map(|x| x.len() as f64 / self.capacity as f64);
        // let color_entropy: f64 = color_probs.map(|x| x * x.log2()).sum();
        //
        // let empty_space = self.capacity - self.content.len();
        // let empty_entropy = if empty_space == 0 {
        //     0.0
        // } else {
        //     let empty_probs = empty_space as f64 / self.capacity as f64;
        //     empty_probs * empty_probs.log2()
        // };
        //
        // -(color_entropy + empty_entropy)

        let mut color_probs = HashMap::new();
        for color in &self.content {
            color_probs
                .entry(color)
                .and_modify(|x| *x += 1)
                .or_insert(1);
        }

        -color_probs
            .values()
            .map(|&x| {
                let prob = x as f64 / self.capacity as f64;
                prob * prob.log2()
            })
            .sum::<f64>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GameState {
    tubes: Vec<Tube>,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tube_cap = self.tubes[0].capacity;
        for level in (0..tube_cap).rev() {
            for tube in &self.tubes {
                let color_block = match tube.content.get(level) {
                    None => " ".on_color("Black"),
                    Some(color) => " ".on_color(color),
                };
                write!(f, "│{color_block}{color_block}│ ")?;
            }
            writeln!(f)?;
        }

        for _ in 0..self.tubes.len() {
            write!(f, "└──┘ ")?;
        }
        writeln!(f)?;

        for i in 0..self.tubes.len() {
            write!(f, " {:02}  ", i + 1)?;
        }
        Ok(())
    }
}

impl GameState {
    fn check_win(&self) -> bool {
        // TODO: Check if this is correct
        for tube in &self.tubes {
            if tube.entropy() > 0.0 {
                return false;
            }
            let tube_len = tube.content.len();
            if tube_len != 0 && tube_len != tube.capacity {
                return false;
            }
        }
        true
    }

    fn available_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = vec![];
        for i in 0..self.tubes.len() {
            for j in i + 1..self.tubes.len() {
                let tube1 = self
                    .tubes
                    .get(i)
                    .expect("Loop over length so shouldn't be out of bound");
                let tube2 = self
                    .tubes
                    .get(j)
                    .expect("Loop over length so shouldn't be out of bound");

                if tube1.is_pour_valid(tube2).is_ok() {
                    moves.push((i, j));
                }
                if tube2.is_pour_valid(tube1).is_ok() {
                    moves.push((j, i));
                }
            }
        }

        moves
    }

    fn make_move(&self, from: usize, to: usize) -> Result<Self, Error> {
        let mut new_game_state = self.clone();

        let tube2 = {
            let mut temp_tube = match new_game_state.tubes.get(to) {
                None => return Err(Error::InvalidMove("To tube doesn't exist")),
                Some(x) => x.clone(),
            };

            let tube1 = match new_game_state.tubes.get_mut(from) {
                None => return Err(Error::InvalidMove("From tube doesn't exist")),
                Some(x) => x,
            };

            tube1.pour(&mut temp_tube)?;
            temp_tube
        };

        new_game_state.tubes[to] = tube2;

        Ok(new_game_state)
    }

    fn make_move_in_place(&mut self, from: usize, to: usize) -> Result<(), Error> {
        let tube2 = {
            let mut temp_tube = match self.tubes.get(to) {
                None => return Err(Error::InvalidMove("To tube doesn't exist")),
                Some(x) => x.clone(),
            };

            let tube1 = match self.tubes.get_mut(from) {
                None => return Err(Error::InvalidMove("From tube doesn't exist")),
                Some(x) => x,
            };

            tube1.pour(&mut temp_tube)?;

            temp_tube
        };

        self.tubes[to] = tube2;
        Ok(())
    }

    fn entropy(&self) -> f64 {
        self.tubes.iter().map(|x| x.entropy()).sum() // TODO: Check if sum or average is better
    }

    fn _avg_entropy(&self) -> f64 {
        let total_entropy = self.entropy();
        total_entropy / self.tubes.len() as f64
    }
}

struct QueueElement {
    moves: Vec<(usize, usize)>,
    game_state: GameState,
}

impl Eq for QueueElement {}

impl PartialEq<Self> for QueueElement {
    fn eq(&self, other: &Self) -> bool {
        self.game_state.entropy() == other.game_state.entropy()
    }
}

impl PartialOrd<Self> for QueueElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueueElement {
    fn cmp(&self, other: &Self) -> Ordering {
        const ALPHA: f64 = 0.65;

        let self_cmp = ALPHA * self.game_state.entropy() + (1.0 - ALPHA) * self.moves.len() as f64;
        let other_cmp =
            ALPHA * other.game_state.entropy() + (1.0 - ALPHA) * other.moves.len() as f64;

        other_cmp
            .partial_cmp(&self_cmp)
            .expect("Should be comparable")
            .then(other.moves.len().cmp(&self.moves.len()))
    }
}

fn solver(game_state: &GameState) -> Vec<(usize, usize)> {
    let mut visited = Vec::new();
    let mut queue = BinaryHeap::new();
    let mut total_state = 1;

    queue.push(QueueElement {
        moves: vec![],
        game_state: game_state.clone(),
    });

    let progress = ProgressBar::new(total_state).with_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [ETA: {eta_precise}] {wide_bar} {percent:<}% [{pos}/{len}] {msg}",
        )
        .unwrap(),
    );

    while !queue.is_empty() && progress.position() <= 5000 {
        progress.set_length(total_state);
        let element = queue.pop().expect("Already checked with while loop");
        let game_state = element.game_state;
        let prev_moves = element.moves;

        visited.push(game_state.clone());

        // println!("{} {}", game_state.entropy(), prev_moves.len());

        if game_state.check_win() {
            progress.abandon();
            return prev_moves.iter().map(|(a, b)| (*a, *b)).collect();
        }

        let all_moves = game_state.available_moves();
        // if all_moves.is_empty() {
        //     eprintln!("Dead end! No more moves");
        // }
        for mov in all_moves {
            match game_state.make_move(mov.0, mov.1) {
                Ok(state) => {
                    if visited.contains(&state) {
                        continue;
                    }

                    let mut cur_moves = prev_moves.clone();
                    // if game_state.entropy() < 3.9 {
                    //     eprintln!("{mov:?}");
                    // }
                    cur_moves.push(mov);
                    queue.push(QueueElement {
                        moves: cur_moves,
                        game_state: state,
                    });
                    total_state += 1;
                }
                Err(_) => continue,
            }
        }
        progress.inc(1);
    }
    progress.abandon();
    vec![]
}

fn display_solution(moves: Vec<(usize, usize)>, mut game_state: GameState) {
    for (step, (from, to)) in moves.into_iter().enumerate() {
        println!(
            "Step {}: Pour from tube {} to tube {}",
            step + 1,
            from + 1,
            to + 1
        );
        let mut direction_string = String::new();
        for i in 0..game_state.tubes.len() {
            if i == from {
                direction_string.push_str(" ↑↑  ");
            } else if i == to {
                direction_string.push_str(" ↓↓  ");
            } else {
                direction_string.push_str("     ");
            }
        }
        println!("{direction_string}");
        println!("{game_state}");

        game_state.make_move_in_place(from, to).unwrap();
    }
}

fn main() {
    let mut colors = vec![];
    let color_prompt: CustomType<(u8, u8, u8)> = CustomType {
        message: "Enter color's RGB values",
        starting_input: None,
        default: None,
        placeholder: Some("255 0 123"),
        help_message: Some("Enter R G B values, separated by spaces, ESC to stop"),
        formatter: &|(r, g, b)| format!("R: {r}, G: {g}, B: {b}"),
        default_value_formatter: &|(r, g, b)| format!("R: {r}, G: {g}, B: {b}"),
        parser: &|input| {
            let temp = input
                .split_ascii_whitespace()
                .filter_map(|x| x.parse().ok())
                .collect_tuple::<(u8, u8, u8)>();
            temp.ok_or(())
        },
        validators: vec![],
        error_message: "Invalid input".to_string(),
        render_config: Default::default(),
    };

    println!("Add color");
    loop {
        let res = color_prompt.clone().prompt_skippable().unwrap();
        if res.is_none() {
            break;
        }
        let (r, g, b) = res.unwrap();

        let color_name = Text::new("What is the color's name?")
            .with_placeholder("Neon Green")
            .with_formatter(&|x| {
                let temp_color = Color {
                    name: String::new(),
                    r,
                    g,
                    b,
                };
                format!("{}", x.color(temp_color))
            })
            .prompt()
            .unwrap();

        let color = Color {
            name: color_name,
            r,
            g,
            b,
        };

        colors.push(color);
    }

    let tube_cap = CustomType::<usize>::new("What is the tube capacity?")
        .prompt()
        .unwrap();
    let tube_num = CustomType::<usize>::new("What is the number of tubes?")
        .prompt()
        .unwrap();
    let mut tubes = Vec::with_capacity(tube_num);

    let mut prev_color = 0;

    for i in 0..tube_num {
        let mut tube_content = Vec::with_capacity(tube_cap);
        for _ in 0..tube_cap {
            let res = Select::new(&format!("Tube {i}: Color to put?"), colors.clone())
                .with_help_message("Select color from the top down, ESC to stop early")
                .with_starting_cursor(prev_color)
                .prompt_skippable()
                .unwrap();
            match res {
                None => break,
                Some(color) => {
                    prev_color = colors.iter().position(|x| x == &color).unwrap();
                    tube_content.push(color)
                }
            };
        }
        tube_content.reverse();
        tubes.push(Tube {
            capacity: tube_cap,
            content: tube_content,
        });
        println!("------------------------");
    }

    let game_state = GameState { tubes };

    println!("{game_state}");

    let solve = solver(&game_state);
    display_solution(solve, game_state);
}
