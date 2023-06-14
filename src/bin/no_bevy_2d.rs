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

        for (row, col) in iproduct!(0..self.row_count, 0..self.col_count) {
            // TODO: Change grid_traverser to the Von Neumann algorithm when it is selected
            let grid_traverser = iproduct!(
                row.saturating_sub(1)..=row.saturating_add(1).min(self.row_count - 1),
                col.saturating_sub(1)..=col.saturating_add(1).min(self.col_count - 1)
            )
            .filter(|&(irow, icol)| irow != row || icol != col)
            .filter_map(|(irow, icol)| self.grid[irow].get(icol));

            //TODO: Add Rules
            let cell = &self.grid[row][col];
            match cell {
                Cell::Dead => {
                    let count_alive: usize = grid_traverser
                        .map(|cell| usize::from(*cell != Cell::Dead))
                        .sum();

                    if count_alive == 3 {
                        temp_grid[row][col] = Cell::Alive;
                    }
                }
                Cell::Alive => {
                    let count_dead: usize = grid_traverser
                        .map(|cell| usize::from(*cell == Cell::Dead))
                        .sum();

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
        /* writeln!(f, "NeighborhoodType: {:?}", self.neighborhood_type)?;
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
        } */
        // ~ UNICODE
        writeln!(f, "NeighborhoodType: {:?}", self.neighborhood_type)?;
        writeln!(f, "Generation: {}", self.generation.0)?;
        writeln!(f, "Grid:")?;
        for row in &self.grid {
            write!(f, "[")?;
            for cell in row {
                match cell {
                    Cell::Dead => write!(f, "⬛"),
                    Cell::Alive => write!(f, "⬜"),
                    Cell::Dying {
                        ticks_till_death: _,
                    } => write!(f, "🟫"),
                }?;
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
    // ~ OPTIMAL: (113, 133) => 1080px
    let (rows, cols) = (50, 20);

    let mut rng = rand::thread_rng();
    let automaton = Automaton::new(rng.gen_range(1..=rows), rng.gen_range(1..=cols));

    for auto in automaton {
        println!("{auto}");
        thread::sleep(Duration::from_secs(1));
    }
}
