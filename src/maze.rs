// maze.rs
use std::fs::File;
use std::io::{BufRead, BufReader};

pub type Maze = Vec<Vec<char>>;

pub fn load_maze(filename: &str) -> Maze {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| line.unwrap().chars().collect())
        .collect()
}

pub fn find_char(maze: &Maze, target: char) -> Option<(usize, usize)> {
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == target {
                return Some((i, j));
            }
        }
    }
    None
}