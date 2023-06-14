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
use std::{
    fmt,
    ops::{ControlFlow, RangeInclusive},
    thread,
    time::Duration,
};

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Generation(usize);

impl Generation {
    const fn is_first_generation(self) -> bool {
        self.0 == 1
    }

    fn increment(&mut self) {
        self.0 += 1;
    }
}

type Grid = Vec<Vec<Cell>>;

#[derive(Debug, Clone)]
struct Automaton {
    row_count: usize,
    col_count: usize,
    grid: Grid,
    generation: Generation,
    neighborhood_type: Neighborhood,
    rule_set: RuleSet,
}

impl Default for Automaton {
    fn default() -> Self {
        let row_count = 20;
        let col_count = 20;
        Self {
            row_count,
            col_count,
            grid: Self::random_population(row_count, col_count),
            generation: Generation::default(),
            neighborhood_type: Neighborhood::default(),
            rule_set: RuleSet::default(),
        }
    }
}

impl Automaton {
    fn new(row_count: usize, col_count: usize, neighborhood_type: Neighborhood) -> Self {
        let grid = Self::random_population(row_count, col_count);
        Self {
            row_count,
            col_count,
            grid,
            neighborhood_type,
            ..Default::default()
        }
    }

    fn random_population(row_count: usize, col_count: usize) -> Grid {
        let mut rng = rand::thread_rng();
        (0..row_count)
            .map(|_| {
                (0..col_count)
                    .map(|_| {
                        if rng.gen_bool(0.5) {
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
        if self.generation.is_first_generation() {
            return Some(self.clone());
        }

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
                        .map(|neighbor| usize::from(*neighbor != Cell::Dead))
                        .sum();

                    let rule_set = if cell.is_dead() {
                        &self.rule_set.dead
                    } else {
                        &self.rule_set.alive
                    };

                    for (rule, action) in rule_set {
                        if rule.check(alive_neighbors, &mut temp_grid[row][col], *action)
                            == ControlFlow::Break(())
                        {
                            break;
                        }
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
        matches!(self, Self::Alive)
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
impl From<Action> for Cell {
    fn from(value: Action) -> Self {
        match value {
            Action::Live => Self::Alive,
            Action::Die => Self::dying_cell(),
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
        match self {
            Self::Singles(v) => {
                if v.contains(&alive_neighbors) {
                    *cell = action.into();
                    return ControlFlow::Break(());
                }
            }
            Self::Range(r) => {
                if r.contains(&alive_neighbors) {
                    *cell = action.into();
                    return ControlFlow::Break(());
                }
            }
        }
        ControlFlow::Continue(())
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
    for auto in Automaton::default() {
        println!("{auto}");
        thread::sleep(Duration::from_secs(1));
    }
}
