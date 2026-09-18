//! A board: a 3×3 grid of cells where players try to get three in a row.
//!
//! A board is an entity whose children are its 9 `Cell` entities. It follows
//! the turns of its **parent** (the hyperboard), which holds a `Turn`:
//! - During a turn, only one of its cells can be pressed. Pressing another
//!   one unpresses the previous one.
//! - When the turn ends, the pressed cell is locked (it can't be clicked
//!   anymore), and the board checks for three in a row of the same color.
//! - A board with three in a row records its winner and shows a square of the
//!   winning color over its cells.
//!
//! The parent controls each board through its `BoardControl`: whether the
//! board can be clicked, and whether it shows its winner square.

mod constant;
mod debug;

use bevy::prelude::*;

use super::cell::{CELL_SIZE, Cell, CellSystems, PressedColor};
pub use constant::{BOARD_SIZE, GRID_SIZE};
use constant::{CELL_GAP, OVERLAY_Z};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app
            // Sets let other features order their systems relative to ours,
            // e.g. `.after(BoardSystems::Turns)`. `.chain()` runs them in order,
            // and all of them run after the clicks of the frame are applied.
            .configure_sets(
                Update,
                (
                    BoardSystems::Presses,
                    BoardSystems::Turns,
                    BoardSystems::Control,
                )
                    .chain()
                    .after(CellSystems::Clicks),
            )
            .add_systems(
                Update,
                (
                    keep_one_press_per_turn.in_set(BoardSystems::Presses),
                    follow_parent_turn.in_set(BoardSystems::Turns),
                    apply_board_control.in_set(BoardSystems::Control),
                ),
            )
            .add_observer(clear_turn_press);
        debug::register(app);
    }
}

/// The groups of board systems, in the order they run each frame.
/// A `SystemSet` is a label for a group of systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BoardSystems {
    /// Handles this frame's clicks: one pressed cell per board and turn.
    Presses,
    /// Follows the parent's `Turn`: ends turns and checks for wins. Runs
    /// after `CellSystems::Clicks` and `Presses`, so the win check always
    /// sees every cell change of the frame.
    Turns,
    /// Applies `BoardControl` to the cells and the winner square. A parent
    /// that changes `BoardControl` between `Turns` and `Control` sees its
    /// change applied in the same frame.
    Control,
}

/// The game's current turn. It goes on a board's **parent** (the hyperboard),
/// which tells its boards what turn it is by changing it.
///
/// It's defined here, in the board feature, because it is what a board needs
/// from its parent. The hyperboard uses the board feature, not the other way
/// round.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Turn {
    /// Increases by one when a turn ends. Boards compare it to the last value
    /// they saw to notice that a turn ended.
    pub number: u32,
    /// Color of the player whose turn it is. The board's cells get pressed
    /// with it.
    pub color: Color,
}

/// What a board's parent allows it to do. It goes on the board, and only the
/// parent should change it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardControl {
    /// When `false`, none of the board's cells can be clicked. When `true`,
    /// only its cells that aren't locked can be clicked.
    pub clickable: bool,
    /// Whether a won board shows its square of the winning color.
    pub show_winner_overlay: bool,
}

impl Default for BoardControl {
    fn default() -> Self {
        Self {
            clickable: true,
            show_winner_overlay: true,
        }
    }
}

/// A board's state. Its cells are its children (Bevy's `Children` component).
///
/// The fields are private: only the board's own systems change them.
#[derive(Component, Debug, Default)]
#[require(Transform, Visibility, BoardControl, Name = Name::new("Board"))]
pub struct Board {
    /// Last turn number seen from the parent. `None` until the board sees one.
    turn: Option<u32>,
    /// The cell pressed during the current turn, if any.
    current: Option<Entity>,
    /// The color that got three in a row first.
    winner: Option<Color>,
    /// Every cell is pressed. Updated when a turn ends.
    full: bool,
}

impl Board {
    /// The cell pressed during the current turn, if any.
    pub fn current(&self) -> Option<Entity> {
        self.current
    }

    /// The color that won this board, if any.
    pub fn winner(&self) -> Option<Color> {
        self.winner
    }

    /// Whether every cell is pressed, as of the last turn end. A full board
    /// without a winner is a draw: nobody can win it anymore.
    pub fn is_full(&self) -> bool {
        self.full
    }
}

/// Where something sits in a 3×3 grid, with row 0 at the top: a cell in its
/// board, or a board in its hyperboard.
///
/// The board adds it to each of its cells. The cell feature doesn't need to
/// know about grids: an entity can have components from several features.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub col: usize,
    pub row: usize,
}

/// An event a parent sends to a board: "unpress the cell pressed this turn".
///
/// An `EntityEvent` targets one entity (`entity`, the board). Send it with
/// `commands.trigger(ClearTurnPress { entity: board })`.
#[derive(EntityEvent, Debug)]
pub struct ClearTurnPress {
    pub entity: Entity,
}

/// Marks a cell that was pressed in an earlier turn. It can't be clicked
/// anymore, and keeps its color for the rest of the game.
#[derive(Component)]
struct Locked;

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

/// Enforces "one pressed cell per turn" inside each board.
///
/// Cells toggle themselves when clicked. This system notices when a second
/// cell of the same board was pressed during the turn, and unpresses the
/// previous one.
fn keep_one_press_per_turn(
    mut boards: Query<(&mut Board, &Children)>,
    mut cells: Query<&mut Cell, Without<Locked>>,
) {
    for (mut board, children) in &mut boards {
        // Cells pressed during this turn. Locked cells were pressed in earlier
        // turns, and `Without<Locked>` leaves them out of the query.
        let pressed_this_turn: Vec<Entity> = children
            .iter()
            .filter(|&child| cells.get(child).is_ok_and(|cell| cell.pressed.is_some()))
            .collect();

        // A pressed cell that isn't `current` was just pressed: it stays, and
        // the others get unpressed. Otherwise nothing new was pressed, and
        // `current` stays (or becomes `None` if it was clicked again to unpress it).
        let newest = match pressed_this_turn
            .iter()
            .find(|&&c| Some(c) != board.current)
        {
            Some(&newest) => {
                for &other in pressed_this_turn.iter().filter(|&&c| c != newest) {
                    if let Ok(mut cell) = cells.get_mut(other) {
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

/// Observer for `ClearTurnPress`: unpresses the board's current cell.
fn clear_turn_press(
    event: On<ClearTurnPress>,
    mut boards: Query<&mut Board>,
    mut cells: Query<&mut Cell>,
) {
    let Ok(mut board) = boards.get_mut(event.entity) else {
        return;
    };
    if let Some(current) = board.current.take()
        && let Ok(mut cell) = cells.get_mut(current)
    {
        cell.pressed = None;
    }
}

/// Reads the `Turn` of each board's parent and reacts when it changes: ends
/// the previous turn and takes the new turn's color.
fn follow_parent_turn(
    mut commands: Commands,
    mut boards: Query<(Entity, &mut Board, &BoardControl, &ChildOf, &Children)>,
    turns: Query<&Turn>,
    cells: Query<(&Cell, &GridPosition)>,
) {
    for (entity, mut board, control, child_of, children) in &mut boards {
        let Ok(turn) = turns.get(child_of.parent()) else {
            continue; // The parent has no `Turn`.
        };
        if board.turn == Some(turn.number) {
            continue; // Same turn as before.
        }
        // The first turn the board sees doesn't end anything.
        if board.turn.is_some() {
            end_turn(&mut commands, entity, &mut board, control, children, &cells);
        }
        board.turn = Some(turn.number);
        // Cells read `PressedColor` from their parent (this board).
        commands.entity(entity).insert(PressedColor(turn.color));
    }
}

/// Locks the cell pressed this turn, and checks whether the board is full and
/// whether it has three in a row.
fn end_turn(
    commands: &mut Commands,
    entity: Entity,
    board: &mut Board,
    control: &BoardControl,
    children: &Children,
    cells: &Query<(&Cell, &GridPosition)>,
) {
    if let Some(pressed) = board.current.take() {
        commands.entity(pressed).insert(Locked);
    }

    let mut grid = [[None; GRID_SIZE]; GRID_SIZE];
    for child in children.iter() {
        if let Ok((cell, position)) = cells.get(child) {
            grid[position.row][position.col] = cell.pressed;
        }
    }
    let full = grid.iter().flatten().all(Option::is_some);
    if board.full != full {
        board.full = full;
    }
    if board.winner.is_some() {
        return; // The first three in a row is the one that counts.
    }
    let Some(color) = winning_color(&grid) else {
        return;
    };

    board.winner = Some(color);
    commands.entity(entity).with_child((
        WinnerOverlay,
        Sprite::from_color(color, Vec2::splat(BOARD_SIZE)),
        Transform::from_xyz(0.0, 0.0, OVERLAY_Z),
        overlay_visibility(control),
    ));
}

/// Applies each board's `BoardControl` to its children: which cells can be
/// clicked, and whether the winner square is shown.
///
/// A cell is clickable when its board is clickable and the cell isn't locked.
/// "Unclickable" means `Pickable::IGNORE`: picking skips the cell entirely.
fn apply_board_control(
    boards: Query<(&BoardControl, &Children)>,
    mut cells: Query<(&mut Pickable, Has<Locked>), With<Cell>>,
    mut overlays: Query<&mut Visibility, With<WinnerOverlay>>,
) {
    for (control, children) in &boards {
        for child in children.iter() {
            if let Ok((mut pickable, locked)) = cells.get_mut(child) {
                let wanted = if control.clickable && !locked {
                    Pickable::default()
                } else {
                    Pickable::IGNORE
                };
                // `set_if_neq` only writes (and marks as changed) when different.
                pickable.set_if_neq(wanted);
            }
            if let Ok(mut visibility) = overlays.get_mut(child) {
                visibility.set_if_neq(overlay_visibility(control));
            }
        }
    }
}

fn overlay_visibility(control: &BoardControl) -> Visibility {
    if control.show_winner_overlay {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

/// The color with a full row, column or diagonal, if any.
/// `grid[row][col]` is the color at that position, `None` if empty.
///
/// Public so the hyperboard can use the same rule on its grid of boards.
pub fn winning_color(grid: &[[Option<Color>; GRID_SIZE]; GRID_SIZE]) -> Option<Color> {
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
