//! A board: a 3×3 grid of cells where players try to get three in a row.
//!
//! A board is an entity whose children are its 9 `Cell` entities. It follows
//! the turns of its **parent** (the hyperboard), which holds a `Turn`:
//! - During a turn, only one of its cells can be pressed. Pressing another
//!   one unpresses the previous one.
//! - When the turn ends, the pressed cell is locked (it can't be clicked
//!   anymore), and the board checks for three in a row of the same color.
//! - A board with three in a row is won: all its cells are locked, and a
//!   square of the winning color covers them.

mod constant;
mod debug;

use bevy::prelude::*;

use super::cell::{CELL_SIZE, Cell, PressedColor};
use constant::{BOARD_SIZE, CELL_GAP, GRID_SIZE, OVERLAY_Z};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        // `.chain()` runs the systems in this order, one after the other:
        // first handle the clicks of this frame, then check if the turn ended.
        app.add_systems(
            Update,
            (keep_one_press_per_turn, follow_parent_turn).chain(),
        );
        debug::register(app);
    }
}

/// The game's current turn. It goes on a board's **parent** (the hyperboard),
/// which tells its boards what turn it is by changing it.
///
/// It's defined here, in the board feature, because it is what a board needs
/// from its parent. The hyperboard will use the board feature, not the other
/// way round.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Turn {
    /// Increases by one when a turn ends. Boards compare it to the last value
    /// they saw to notice that a turn ended.
    pub number: u32,
    /// Color of the player whose turn it is. The board's cells get pressed
    /// with it.
    pub color: Color,
}

/// A board's state. Its cells are its children (Bevy's `Children` component).
///
/// The fields are private: only the board's own systems change them.
#[derive(Component, Debug, Default)]
#[require(Transform, Visibility, Name = Name::new("Board"))]
pub struct Board {
    /// Last turn number seen from the parent. `None` until the board sees one.
    turn: Option<u32>,
    /// The cell pressed during the current turn, if any.
    current: Option<Entity>,
    /// The color that got three in a row. Once set, the board is disabled.
    winner: Option<Color>,
}

impl Board {
    /// The color that won this board, if any.
    #[allow(dead_code)] // Will be read by the hyperboard.
    pub fn winner(&self) -> Option<Color> {
        self.winner
    }
}

/// Where a cell sits in its board, with row 0 at the top.
///
/// The board adds it to each of its cells. The cell feature doesn't need to
/// know about grids: an entity can have components from several features.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub col: usize,
    pub row: usize,
}

/// Marks the square that covers a won board.
#[derive(Component)]
#[require(Name = Name::new("Winner overlay"))]
struct WinnerOverlay;

/// Spawns a board with its 9 cells as children, and returns the board entity.
///
/// To put it inside a hyperboard, make it a child of the hyperboard:
/// `commands.entity(board).insert(ChildOf(hyperboard))`.
pub fn spawn_board(commands: &mut Commands, transform: Transform) -> Entity {
    commands
        .spawn((Board::default(), transform))
        // Everything spawned inside `with_children` gets `ChildOf(board)`.
        .with_children(|board| {
            for row in 0..GRID_SIZE {
                for col in 0..GRID_SIZE {
                    board.spawn((
                        Cell::default(),
                        GridPosition { col, row },
                        // Relative to the board's center, not to the window.
                        Transform::from_translation(cell_position(col, row).extend(0.0)),
                    ));
                }
            }
        })
        .id()
}

/// Enforces "one pressed cell per turn".
///
/// Cells toggle themselves when clicked. This system notices when a second
/// cell of the same board was pressed during the turn, and unpresses the
/// previous one.
fn keep_one_press_per_turn(
    mut boards: Query<(&mut Board, &Children)>,
    mut cells: Query<(&mut Cell, &Pickable)>,
) {
    for (mut board, children) in &mut boards {
        // Cells pressed during this turn. Cells locked in earlier turns are
        // pressed too, but they are no longer pickable.
        let pressed_this_turn: Vec<Entity> = children
            .iter()
            .filter(|&child| {
                cells
                    .get(child)
                    .is_ok_and(|(cell, pickable)| cell.pressed.is_some() && pickable.is_hoverable)
            })
            .collect();

        // A pressed cell that isn't `current` was just pressed: it wins, and
        // the others get unpressed. Otherwise nothing new was pressed, and
        // `current` stays (or becomes `None` if it was clicked again to unpress it).
        let newest = match pressed_this_turn
            .iter()
            .find(|&&c| Some(c) != board.current)
        {
            Some(&newest) => {
                for &other in pressed_this_turn.iter().filter(|&&c| c != newest) {
                    if let Ok((mut cell, _)) = cells.get_mut(other) {
                        cell.pressed = None;
                    }
                }
                Some(newest)
            }
            None => pressed_this_turn.first().copied(),
        };
        // Only write when it changes: writing marks the component as changed,
        // even with the same value, and the debug log reacts to changes.
        if board.current != newest {
            board.current = newest;
        }
    }
}

/// Reads the `Turn` of each board's parent and reacts when it changes: ends
/// the previous turn and takes the new turn's color.
fn follow_parent_turn(
    mut commands: Commands,
    mut boards: Query<(Entity, &mut Board, &ChildOf, &Children)>,
    turns: Query<&Turn>,
    mut cells: Query<(&Cell, &GridPosition, &mut Pickable)>,
) {
    for (entity, mut board, child_of, children) in &mut boards {
        let Ok(turn) = turns.get(child_of.parent()) else {
            continue; // The parent has no `Turn`.
        };
        if board.turn == Some(turn.number) || board.winner.is_some() {
            continue; // Same turn as before, or the board is disabled.
        }
        // The first turn the board sees doesn't end anything.
        if board.turn.is_some() {
            end_turn(&mut commands, entity, &mut board, children, &mut cells);
        }
        board.turn = Some(turn.number);
        if board.winner.is_none() {
            // Cells read `PressedColor` from their parent (this board).
            commands.entity(entity).insert(PressedColor(turn.color));
        }
    }
}

/// Locks the cell pressed this turn, and checks for three in a row.
fn end_turn(
    commands: &mut Commands,
    entity: Entity,
    board: &mut Board,
    children: &Children,
    cells: &mut Query<(&Cell, &GridPosition, &mut Pickable)>,
) {
    if let Some(pressed) = board.current.take()
        && let Ok((_, _, mut pickable)) = cells.get_mut(pressed)
    {
        *pickable = Pickable::IGNORE;
    }

    let mut grid = [[None; GRID_SIZE]; GRID_SIZE];
    for child in children.iter() {
        if let Ok((cell, position, _)) = cells.get(child) {
            grid[position.row][position.col] = cell.pressed;
        }
    }
    let Some(color) = winning_color(&grid) else {
        return;
    };

    board.winner = Some(color);
    for child in children.iter() {
        if let Ok((_, _, mut pickable)) = cells.get_mut(child) {
            *pickable = Pickable::IGNORE;
        }
    }
    commands.entity(entity).with_child((
        WinnerOverlay,
        Sprite::from_color(color, Vec2::splat(BOARD_SIZE)),
        Transform::from_xyz(0.0, 0.0, OVERLAY_Z),
    ));
}

/// The color with a full row, column or diagonal, if any.
/// `grid[row][col]` is the color a cell was pressed with.
fn winning_color(grid: &[[Option<Color>; GRID_SIZE]; GRID_SIZE]) -> Option<Color> {
    let n = GRID_SIZE;
    let rows = (0..n).map(|row| (0..n).map(|col| grid[row][col]).collect::<Vec<_>>());
    let columns = (0..n).map(|col| (0..n).map(|row| grid[row][col]).collect());
    let diagonal = (0..n).map(|i| grid[i][i]).collect();
    let anti_diagonal = (0..n).map(|i| grid[i][n - 1 - i]).collect();

    rows.chain(columns)
        .chain([diagonal, anti_diagonal])
        .find_map(|line: Vec<Option<Color>>| {
            // `?` returns `None` from this closure when the first cell is empty.
            let first = line[0]?;
            line.iter().all(|&c| c == Some(first)).then_some(first)
        })
}

/// Position of the center of the cell at (`col`, `row`), relative to the
/// board's center.
///
/// In Bevy, +X points right and +Y points **up**, so row 0 (the top) gets the
/// largest Y.
fn cell_position(col: usize, row: usize) -> Vec2 {
    let step = CELL_SIZE + CELL_GAP;
    // Distance from the grid's center to the center of the outermost cells.
    let offset = (GRID_SIZE as f32 - 1.0) * step / 2.0;
    Vec2::new(col as f32 * step - offset, offset - row as f32 * step)
}

// Unit tests live in `tests.rs`, next to this file. `#[cfg(test)]` means they
// are only compiled for `cargo test`, never into the game.
#[cfg(test)]
mod tests;
