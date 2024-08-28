use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Color {
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

impl Color {
    pub fn new(name: String, r: u8, g: u8, b: u8) -> Self {
        Self { name, r, g, b }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Tube {
    capacity: usize,
    content: Vec<Color>,
}

impl Tube {
    pub fn new(capacity: usize, content: Vec<Color>) -> Self {
        Self { capacity, content }
    }

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
pub struct GameState {
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
    pub fn new(tubes: Vec<Tube>) -> Self {
        Self { tubes }
    }

    pub fn tube_num(&self) -> usize {
        self.tubes.len()
    }

    pub fn check_win(&self) -> bool {
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

    pub fn available_moves(&self) -> Vec<(usize, usize)> {
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

    pub fn make_move(&self, from: usize, to: usize) -> Result<Self, Error> {
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

    pub fn make_move_in_place(&mut self, from: usize, to: usize) -> Result<(), Error> {
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

    pub fn entropy(&self) -> f64 {
        self.tubes.iter().map(|x| x.entropy()).sum() // TODO: Check if sum or average is better
    }

    pub fn _avg_entropy(&self) -> f64 {
        let total_entropy = self.entropy();
        total_entropy / self.tubes.len() as f64
    }
}
