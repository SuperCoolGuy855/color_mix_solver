use std::fs::File;

use colored::Colorize;
use inquire::{CustomType, Select, Text};
use itertools::Itertools;

use game::GameState;

use crate::game::{Color, Tube};

mod game;
mod solver;

#[derive(Debug)]
enum Error {
    MaxCapacity,
    NoContent,
    CantMove,
    DiffColor,
    InvalidMove(&'static str),
}

fn load_color() -> Option<Vec<Color>> {
    let color_file = match File::open("colors.json") {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Unable to read colors file: {e}");
            return None;
        }
    };

    let colors: Vec<Color> = match serde_json::from_reader(color_file) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Unable to parse file, possible corruption: {e}");
            return None;
        }
    };

    Some(colors)
}

fn color_menu(mut colors: Vec<Color>) -> Vec<Color> {
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

    println!("Current colors:");
    for color in &colors {
        println!("{color}");
    }

    loop {
        let res = color_prompt.clone().prompt_skippable().unwrap(); // This is deliberate, until better error message
        if res.is_none() {
            break;
        }
        let (r, g, b) = res.expect("This IS checked above");

        let color_name = Text::new("What is the color's name?")
            .with_placeholder("Neon Green")
            .with_formatter(&|x| {
                let temp_color = Color::new(String::new(), r, g, b);
                format!("{}", x.color(temp_color))
            })
            .with_help_message("Enter name of existing color to override")
            .prompt()
            .unwrap(); // This is deliberate, until better error message

        let color = Color::new(color_name, r, g, b);

        let exist_res = colors.iter().position(|x| x.get_name() == color.get_name());
        match exist_res {
            None => colors.push(color),
            Some(x) => {
                colors[x] = color;
            }
        }
    }

    let color_file = File::create("colors.json")
        .unwrap_or_else(|e| panic!("Unable to create colors file: {e}. Exiting"));
    serde_json::to_writer(color_file, &colors)
        .unwrap_or_else(|e| panic!("Unable to write json to colors file: {e}. Exiting")); // This is deliberate, until better error message

    colors
}

fn main() {
    // TODO: If found, show main menu

    let mut colors = load_color().unwrap_or_else(|| color_menu(vec![]));

    let options = vec!["Add colors", "Solver"];

    loop {
        let select = Select::new("What would you like to do?", options.clone()).prompt().unwrap(); // This is deliberate, until better error message
        if select == "Solver" {
            break;
        }
        colors = color_menu(colors);
    }

    let tube_cap = CustomType::<usize>::new("What is the tube capacity?")
        .prompt()
        .unwrap(); // This is deliberate, until better error message
    let tube_num = CustomType::<usize>::new("What is the number of tubes?")
        .prompt()
        .unwrap(); // This is deliberate, until better error message
    let mut tubes = Vec::with_capacity(tube_num);

    let mut prev_color = 0;

    for i in 0..tube_num {
        let mut tube_content = Vec::with_capacity(tube_cap);
        for _ in 0..tube_cap {
            let res = Select::new(&format!("Tube {i}: Color to put?"), colors.clone())
                .with_help_message("Select color from the top down, ESC to stop early")
                .with_starting_cursor(prev_color)
                .prompt_skippable()
                .unwrap(); // This is deliberate, until better error message
            match res {
                None => break,
                Some(color) => {
                    prev_color = colors.iter().position(|x| x == &color).expect(
                        "The color should be in list, as user can only select from that list",
                    );
                    tube_content.push(color)
                }
            };
        }
        tube_content.reverse();
        tubes.push(Tube::new(tube_cap, tube_content));
        println!("------------------------");
    }

    let game_state = GameState::new(tubes);

    println!("{game_state}");

    let solve = solver::solver(&game_state);
    solver::display_solution(solve, game_state);
}
