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

use itertools::{iproduct, Itertools};
use rand::Rng;
use std::{
    fmt,
    ops::{ControlFlow, RangeInclusive},
    thread,
    time::Duration,
};

type Grid = Vec<Vec<Cell>>;

#[derive(typed_builder::TypedBuilder, Debug, Clone)]
#[builder(field_defaults(default))]
struct Automaton {
    generation: usize,
    row_count: usize,
    col_count: usize,
    grid: Grid,
    neighborhood_type: Neighborhood,
    rule_set: RuleSet,
}

impl Default for Automaton {
    fn default() -> Self {
        const ROW_COUNT: usize = 20;
        const COL_COUNT: usize = 20;
        Self {
            row_count: ROW_COUNT,
            col_count: COL_COUNT,
            grid: Self::random_population(ROW_COUNT, COL_COUNT),
            generation: Default::default(),
            neighborhood_type: Neighborhood::default(),
            rule_set: RuleSet::default(),
        }
    }
}

impl Automaton {
    fn random_population(row_count: usize, col_count: usize) -> Grid {
        (0..row_count)
            .map(|_| (0..col_count).map(|_| Self::random_cell()).collect())
            .collect()
    }

    fn random_cell() -> Cell {
        if rand::thread_rng().gen_bool(0.5) {
            Cell::Alive
        } else {
            Cell::default()
        }
    }
}

impl Iterator for Automaton {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        self.generation += 1;

        let mut temp_grid = self.grid.clone();

        for (row, col) in iproduct!(0..self.row_count, 0..self.col_count) {
            let grid_traverser = iproduct!(
                row.saturating_sub(1)..=row.saturating_add(1).min(self.row_count - 1),
                col.saturating_sub(1)..=col.saturating_add(1).min(self.col_count - 1)
            )
            .filter(|&(irow, icol)| irow != row || icol != col);

            // ? Casting to Box<Iterator> Necessary to remove unnecessary collecting into a vector for each match arm.
            let grid_traverser = match self.neighborhood_type {
                Neighborhood::Moore => Box::new(grid_traverser),
                Neighborhood::VonNeumann => {
                    Box::new(grid_traverser.filter(|&(irow, icol)| irow == row || icol == col))
                        as Box<dyn Iterator<Item = (usize, usize)>>
                }
            }
            .filter_map(|(irow, icol)| self.grid[irow].get(icol));

            let cell = &self.grid[row][col];
            match cell {
                Cell::Dead | Cell::Alive => {
                    let alive_neighbors: usize = grid_traverser
                        .map(|neighbor| usize::from(neighbor.is_alive()))
                        .sum();

                    let rule_set = if cell.is_dead() {
                        &self.rule_set.dead
                    } else {
                        &self.rule_set.alive
                    };

                    rule_set.iter().any(|(rule, action)| {
                        rule.check(alive_neighbors, &mut temp_grid[row][col], *action)
                            .is_break()
                    });
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
        std::mem::swap(&mut self.grid, &mut temp_grid);

        Some(Self {
            grid: temp_grid,
            rule_set: self.rule_set.clone(),
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
        writeln!(f, "Generation: {}", self.generation)?;
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

/// Represents the Neighborhood checking type
/// - `Moore` => Checks all neighbors including the diagonal neighbors
/// - `VonNeumann` => Checks all neighbors excluding the diagonal neighbors
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Neighborhood {
    #[default]
    Moore,
    VonNeumann,
}

/// Represents The current State of the Cell
/// - `Dead` => The Cell is dead
/// - `Alive` => The Cell is alive
/// - `Dying` => The Cell is currently dying with the state counter `ticks_till_death`
/// representing the remaining generations until the Cell is dead
/// i.e. Changes to the `Dead` state
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum Cell {
    #[default]
    Dead,
    Alive,
    Dying {
        ticks_till_death: usize,
    },
}

impl Cell {
    const fn is_dead(&self) -> bool {
        matches!(self, Self::Dead)
    }
    const fn is_alive(&self) -> bool {
        !self.is_dead()
    }
    const fn is_dying(&self) -> bool {
        matches!(
            self,
            Self::Dying {
                ticks_till_death: _
            }
        )
    }

    const fn dying_cell() -> Self {
        const TICKS_TILL_DEATH: usize = 3;
        Self::Dying {
            ticks_till_death: TICKS_TILL_DEATH,
        }
    }
}

// TODO: Replace "dying cells" with Dead in order to exactly imitate conways game of life when needed.
impl From<Action> for Cell {
    fn from(value: Action) -> Self {
        match value {
            Action::Live => Self::Alive,
            Action::Die => Self::Dead,
        }
    }
}
impl From<&Action> for Cell {
    fn from(value: &Action) -> Self {
        Self::from(*value)
    }
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

/// `RuleSets` for the Automata
///
/// It is combined
/// Defaults to the Rules of Conway's Game of Life
#[derive(Debug, PartialEq, Eq, Clone)]
struct RuleSet {
    /// Rules for an `Cell::Alive`
    alive: Vec<(Rules, Action)>,
    /// Rules for an `Cell::Dead`
    dead: Vec<(Rules, Action)>,
}
impl Default for RuleSet {
    fn default() -> Self {
        Self {
            alive: vec![
                (Rules::Range(0..=1), Action::Die),
                (Rules::Range(2..=3), Action::Live),
                (Rules::Range(4..=9), Action::Die),
            ],
            dead: vec![(Rules::Singles(vec![3]), Action::Live)],
        }
    }
}

/// Subset of `RuleSet`
///
/// - `Range` Determines an Inclusive range in which a rule Applies
/// - `Singles` Determines multiple values in which a rule Applies
#[derive(Debug, PartialEq, Eq, Clone)]
enum Rules {
    Range(RangeInclusive<usize>),
    Singles(Vec<usize>),
}

impl Rules {
    fn check(&self, alive_neighbors: usize, cell: &mut Cell, action: Action) -> ControlFlow<()> {
        let mut iterable: Box<dyn Iterator<Item = usize>> = match self {
            Self::Range(r) => Box::new(r.clone()),
            Self::Singles(s) => Box::new(s.iter().copied()),
        };

        if iterable.contains(&alive_neighbors) {
            *cell = action.into();
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

/// The action to perform when Operating on a Cell
///
/// - `Live` => transforms the Cell to `Cell::Alive`
/// - `Die`  => transforms the Cell to `Cell::Dying`
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Action {
    #[default]
    Live,
    Die,
}

fn main() {
    let grid = vec![vec![Cell::Dead, Cell::Alive, Cell::Dead]; 3];
    let automaton = Automaton::builder()
        .row_count(3)
        .col_count(3)
        .grid(grid)
        .build();

    for auto in automaton {
        println!("{auto}");
        thread::sleep(Duration::from_secs(1));
    }
    /* for auto in Automaton::default() {
        println!("{auto}");
        thread::sleep(Duration::from_secs(1));
    } */
}

// ! THESE TESTS ONLY WORK WHEN THE DYING LOGIC IS SET TO Cell::Dead
// ! Instead of Cell::dying_cell()
// ! i.e. WHEN THE AUTOMATON EXACTLY REPRESENTS THE LOGIC OF CONWAYS GAME OF LIFE
#[cfg(test)]
mod tests {
    use crate::{Automaton, Cell, Neighborhood};
    use std::{thread, time::Duration};

    #[test]
    fn primitive_test_1() {
        let grid = vec![vec![Cell::Dead, Cell::Alive, Cell::Dead]; 3];
        let mut automaton = Automaton::builder()
            .row_count(3)
            .col_count(3)
            .grid(grid.clone())
            .build();

        assert_eq!(automaton.next().unwrap().grid, grid);
        assert_ne!(automaton.next().unwrap().grid, grid);
        assert_eq!(automaton.next().unwrap().grid, grid);
    }
    #[test]
    #[should_panic]
    fn primitive_test_2() {
        let grid = vec![vec![Cell::Dead, Cell::Alive, Cell::Dead]; 3];
        let mut automaton = Automaton::builder()
            .row_count(3)
            .col_count(3)
            .grid(grid.clone())
            .build();

        assert_eq!(automaton.next().unwrap().grid, grid);
        assert_eq!(automaton.next().unwrap().grid, grid);
    }
}
