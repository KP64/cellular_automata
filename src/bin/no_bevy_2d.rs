#![warn(
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]
#![allow(unused)]

use itertools::iproduct;
use rand::Rng;
use std::{fmt, thread, time::Duration};

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Generation(usize);

impl Generation {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

type Grid = Vec<Vec<Cell>>;

#[derive(Default, Debug, Clone)]
struct Automaton {
    row_count: usize,
    col_count: usize,
    grid: Grid,
    generation: Generation,
    neighborhood_type: Neighborhood,
}

impl Automaton {
    fn new(row_count: usize, col_count: usize) -> Self {
        let grid = Self::random_population(row_count, col_count);
        Self {
            row_count,
            col_count,
            grid,
            ..Default::default()
        }
    }

    fn random_population(row_count: usize, col_count: usize) -> Grid {
        let mut rng = rand::thread_rng();
        (0..row_count)
            .map(|_| {
                (0..col_count)
                    .map(|_| {
                        if rng.gen_bool(0.3) {
                            Cell::Alive
                        } else {
                            Cell::default()
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

impl Iterator for Automaton {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        self.generation.increment();

        // ~ Is the first Generation considered the "unprocessed" grid or the first "new" generation?
        // TODO: Confirm right behavior
        if self.generation.0 == 1 {
            return Some(self.clone());
        }

        let mut temp_grid = self.grid.clone();

        // ? Stop simulation if all cells are dead
        if !self
            .grid
            .iter()
            .any(|col| col.iter().any(|cell| *cell != Cell::Dead))
        {
            return None;
        }

        for (row, col) in iproduct!(0..self.row_count, 0..self.col_count) {
            let cell = &mut self.grid[row][col];
            match *cell {
                Cell::Dead => {
                    let mut count_alive = 0;

                    for (irow, icol) in iproduct!(
                        row.saturating_sub(1)..=(row + 1).min(self.row_count - 1),
                        col.saturating_sub(1)..=(col + 1).min(self.col_count - 1)
                    ) {
                        if irow == row && icol == col {
                            continue;
                        }

                        let cell = &mut self.grid[irow][icol];
                        match cell {
                            Cell::Dead => {}
                            _ => count_alive += 1,
                        }
                    }

                    if count_alive > 2 && count_alive < 4 {
                        temp_grid[row][col] = Cell::Alive;
                    }
                }
                Cell::Alive => {
                    let mut count_dead = 0;

                    for (irow, icol) in iproduct!(
                        row.saturating_sub(1)..=(row + 1).min(self.row_count - 1),
                        col.saturating_sub(1)..=(col + 1).min(self.col_count - 1)
                    ) {
                        if irow == row && icol == col {
                            continue;
                        }

                        let cell = &mut self.grid[irow][icol];
                        match cell {
                            Cell::Dead => {}
                            _ => count_dead += 1,
                        }
                    }

                    if count_dead > 4 {
                        temp_grid[row][col] = Cell::Dying {
                            ticks_till_death: 3,
                        };
                    }
                }
                Cell::Dying { ticks_till_death } => {
                    let new_ticks = ticks_till_death - 1;
                    temp_grid[row][col] = if new_ticks == 0 {
                        Cell::default()
                    } else {
                        Cell::Dying {
                            ticks_till_death: new_ticks,
                        }
                    };
                }
            }
        }
        self.grid = temp_grid.clone();

        Some(Self {
            grid: temp_grid,
            ..*self
        })
    }
}

impl fmt::Display for Automaton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // ~ PLAIN TEXT
        /*         writeln!(f, "NeighborhoodType: {:?}", self.neighborhood_type)?;
        writeln!(f, "Generation: {}", self.generation.0)?;
        writeln!(f, "Grid:")?;
        for idx in 0..self.col_count {
            write!(f, " {idx:^8} ")?;
        }
        writeln!(f)?;
        for (idx, row) in self.grid.iter().enumerate() {
            write!(f, "{idx:<2}[")?;
            for col in row {
                write!(f, "{:<8}, ", format!("{}", col))?;
            }
            writeln!(f, "]")?;
        }
        Ok(()) */
        // ~ UNICODE
        writeln!(f, "NeighborhoodType: {:?}", self.neighborhood_type)?;
        writeln!(f, "Generation: {}", self.generation.0)?;
        writeln!(f, "Grid:")?;
        for idx in 0..self.col_count {
            write!(f, "  {idx:>2}")?;
        }
        writeln!(f)?;
        for (idx, row) in self.grid.iter().enumerate() {
            write!(f, "{idx}[")?;
            for cell in row {
                match cell {
                    Cell::Dead => write!(f, "\u{2B1B}, "),
                    Cell::Alive => write!(f, "\u{2B1C}, "),
                    Cell::Dying { ticks_till_death } => write!(f, "\u{1F7EB}, "),
                }?;
                /* write!(f, "{:<8}, ", format!("{}", col))?; */
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Neighborhood {
    #[default]
    Moore,
    VonNeumann,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum Cell {
    #[default]
    Dead,
    Alive,
    Dying {
        ticks_till_death: usize,
    },
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dead => write!(f, "Dead"),
            Self::Alive => write!(f, "Alive"),
            Self::Dying { ticks_till_death } => write!(f, "Death {ticks_till_death}"),
        }
    }
}

fn main() {
    let (rows, cols) = (10, 10);
    /* let gen_limit = 40; */
    let automaton = Automaton::new(rows, cols);

    println!("{automaton}");
    for (gen, auto) in automaton.enumerate() {
        println!("{auto}");
        thread::sleep(Duration::from_secs(1));

        /*         if gen == gen_limit {
            break;
        } */
    }
}
