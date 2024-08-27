use std::cmp::Ordering;
use std::collections::BinaryHeap;

use indicatif::{ProgressBar, ProgressStyle};

use crate::game::GameState;

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

pub fn solver(game_state: &GameState) -> Vec<(usize, usize)> {
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
        .expect("This should NEVER fail"),
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

pub fn display_solution(moves: Vec<(usize, usize)>, mut game_state: GameState) {
    for (step, (from, to)) in moves.into_iter().enumerate() {
        println!(
            "Step {}: Pour from tube {} to tube {}",
            step + 1,
            from + 1,
            to + 1
        );
        let mut direction_string = String::new();
        for i in 0..game_state.tube_num() {
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

        game_state
            .make_move_in_place(from, to)
            .expect("The moves input should be correct. If not, the solver is wrong");
    }
}
