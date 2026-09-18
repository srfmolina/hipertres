//! The hyperboard: a 3×3 grid of boards, and the whole game's rules.
//!
//! A hyperboard is an entity whose children are its 9 `Board` entities. It
//! controls the game:
//! - The players (a list of colors) and the `Turn`: which turn it is and
//!   which player plays it. Ending a turn gives it to the next player.
//! - One pressed cell per turn in the **whole** hyperboard: pressing a cell in
//!   another board unpresses the one pressed earlier in the turn.
//! - A turn can only end after the player pressed a cell: no passing.
//! - Which boards can be clicked and show their winner square, through each
//!   board's `BoardControl`. For now: every board except won and full ones.
//! - After every turn, three won boards in a row of the same color win the
//!   hyperboard: a square of that color covers everything, and no board can
//!   be clicked anymore.
//!
//! # Order of a frame
//!
//! Every step runs in a named set, and Bevy guarantees the order between them:
//!
//! 1. `CellSystems::Clicks`: clicks are applied to cells.
//! 2. `BoardSystems::Presses`, then `HyperboardSystems::Presses`: one press
//!    per turn, in each board and then in the whole hyperboard.
//! 3. `HyperboardSystems::EndTurn`: a request to end the turn is accepted or rejected.
//! 4. `BoardSystems::Turns`: boards end the turn and check their win. Every
//!    cell change of the frame has already happened.
//! 5. `HyperboardSystems::Results`: the hyperboard checks its win and decides
//!    each board's `BoardControl`. Every board has already checked its win.
//! 6. `BoardSystems::Control`: boards apply their `BoardControl`.

mod constant;
mod debug;

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

use super::board::{
    BOARD_SIZE, Board, BoardControl, BoardSystems, ClearTurnPress, GRID_SIZE, GridPosition, Turn,
    spawn_board, winning_color,
};
use constant::{BOARD_GAP, BOARD_SCALE, DEFAULT_PLAYERS, END_TURN_KEY, HYPERBOARD_SIZE, OVERLAY_Z};

pub struct HyperboardPlugin;

impl Plugin for HyperboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EndTurnRequested>()
            // Place our steps between the board's (see "Order of a frame" above).
            .configure_sets(
                Update,
                (
                    HyperboardSystems::Presses
                        .after(BoardSystems::Presses)
                        .before(HyperboardSystems::EndTurn),
                    HyperboardSystems::EndTurn.before(BoardSystems::Turns),
                    HyperboardSystems::Results
                        .after(BoardSystems::Turns)
                        .before(BoardSystems::Control),
                ),
            )
            .add_systems(
                Update,
                (
                    keep_one_press_per_turn.in_set(HyperboardSystems::Presses),
                    (
                        // `input_just_pressed` is a run condition from Bevy: the
                        // system only runs on the frame the key goes down.
                        request_end_turn_on_key.run_if(input_just_pressed(END_TURN_KEY)),
                        end_requested_turns,
                    )
                        .chain()
                        .in_set(HyperboardSystems::EndTurn),
                    (check_for_winner, control_boards)
                        .chain()
                        .in_set(HyperboardSystems::Results),
                ),
            );
        debug::register(app);
    }
}

/// The hyperboard's steps in `Update`. See "Order of a frame" above.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum HyperboardSystems {
    /// Keeps one pressed cell per turn in the whole hyperboard.
    Presses,
    /// Handles `EndTurnRequested`.
    EndTurn,
    /// Checks the hyperboard's win and sets each board's `BoardControl`.
    Results,
}

/// A *message*: "end the current turn of this hyperboard". The Space key
/// sends it. It is rejected if no cell was pressed this turn.
#[derive(Message, Debug, Clone, Copy)]
pub struct EndTurnRequested {
    pub hyperboard: Entity,
}

/// The hyperboard's state. Its boards are its children, and it also holds the
/// game's `Turn`.
#[derive(Component, Debug)]
#[require(Transform, Visibility, Name = Name::new("Hyperboard"))]
pub struct Hyperboard {
    /// Players in turn order, identified by their color.
    players: Vec<Color>,
    /// The board with the cell pressed during the current turn, if any.
    active_board: Option<Entity>,
    /// The color that got three boards in a row.
    winner: Option<Color>,
}

impl Hyperboard {
    /// A hyperboard for these players, in turn order. Panics if there are none.
    pub fn new(players: Vec<Color>) -> Self {
        assert!(
            !players.is_empty(),
            "a hyperboard needs at least one player"
        );
        Self {
            players,
            active_board: None,
            winner: None,
        }
    }

    /// The color that won the game, if any.
    pub fn winner(&self) -> Option<Color> {
        self.winner
    }

    /// The player who plays turn `number` (turns start at 1): players take
    /// turns in list order, starting again from the first.
    pub fn player_for_turn(&self, number: u32) -> Color {
        let index = (number as usize).saturating_sub(1) % self.players.len();
        self.players[index]
    }
}

impl Default for Hyperboard {
    fn default() -> Self {
        Self::new(DEFAULT_PLAYERS.to_vec())
    }
}

/// Marks the square that covers a won hyperboard.
#[derive(Component)]
#[require(Name = Name::new("Hyperboard winner overlay"))]
struct WinnerOverlay;

/// Spawns a hyperboard with its 9 boards as children, starting at turn 1, and
/// returns the hyperboard entity.
pub fn spawn_hyperboard(
    commands: &mut Commands,
    hyperboard: Hyperboard,
    transform: Transform,
) -> Entity {
    let turn = Turn {
        number: 1,
        color: hyperboard.player_for_turn(1),
    };
    let root = commands.spawn((hyperboard, turn, transform)).id();
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let transform = Transform::from_translation(board_position(col, row).extend(0.0))
                // Scale x and y only. Scaling z too would squash the boards'
                // own layers (their winner square is at z = 1).
                .with_scale(Vec3::new(BOARD_SCALE, BOARD_SCALE, 1.0));
            let board = spawn_board(commands, transform);
            commands
                .entity(board)
                .insert((GridPosition { col, row }, ChildOf(root)));
        }
    }
    root
}

fn request_end_turn_on_key(
    hyperboards: Query<Entity, With<Hyperboard>>,
    mut requests: MessageWriter<EndTurnRequested>,
) {
    for hyperboard in &hyperboards {
        requests.write(EndTurnRequested { hyperboard });
    }
}

/// Ends the turn of each hyperboard that asked for it: the turn number goes up
/// by one, and the next player gets the turn.
///
/// Nothing happens if the game is won, or if no cell was pressed this turn
/// (a player can't pass).
fn end_requested_turns(
    mut requests: MessageReader<EndTurnRequested>,
    mut hyperboards: Query<(&mut Hyperboard, &mut Turn)>,
) {
    for request in requests.read() {
        let Ok((mut hyperboard, mut turn)) = hyperboards.get_mut(request.hyperboard) else {
            continue;
        };
        if hyperboard.winner.is_some() || hyperboard.active_board.is_none() {
            continue;
        }
        turn.number += 1;
        turn.color = hyperboard.player_for_turn(turn.number);
        hyperboard.active_board = None;
    }
}

/// Enforces "one pressed cell per turn" in the whole hyperboard, the same way
/// each board does it for its cells: when a second board gets a pressed cell
/// during the turn, the previous board is told to clear its press.
fn keep_one_press_per_turn(
    mut commands: Commands,
    mut hyperboards: Query<(&mut Hyperboard, &Children)>,
    boards: Query<&Board>,
) {
    // This runs before `EndTurn` (see "Order of a frame"), so the boards'
    // presses always belong to the current turn.
    for (mut hyperboard, children) in &mut hyperboards {
        let boards_with_press: Vec<Entity> = children
            .iter()
            .filter(|&child| boards.get(child).is_ok_and(|b| b.current().is_some()))
            .collect();

        let newest = match boards_with_press
            .iter()
            .find(|&&b| Some(b) != hyperboard.active_board)
        {
            Some(&newest) => {
                for &other in boards_with_press.iter().filter(|&&b| b != newest) {
                    // "Calls go down": we tell the board, and the board
                    // unpresses its own cell.
                    commands.trigger(ClearTurnPress { entity: other });
                }
                Some(newest)
            }
            None => boards_with_press.first().copied(),
        };
        if hyperboard.active_board != newest {
            hyperboard.active_board = newest;
        }
    }
}

/// After every turn, checks for three won boards in a row of the same color.
fn check_for_winner(
    mut commands: Commands,
    mut hyperboards: Query<(Entity, &mut Hyperboard, Ref<Turn>, &Children)>,
    boards: Query<(&Board, &GridPosition)>,
) {
    for (entity, mut hyperboard, turn, children) in &mut hyperboards {
        // `Ref<Turn>` gives read access plus change detection: the turn
        // changed this frame means a turn just ended. The boards already
        // checked their own wins earlier in this frame (`BoardSystems::Turns`).
        if !turn.is_changed() || hyperboard.winner.is_some() {
            continue;
        }
        let mut grid = [[None; GRID_SIZE]; GRID_SIZE];
        for child in children.iter() {
            if let Ok((board, position)) = boards.get(child) {
                grid[position.row][position.col] = board.winner();
            }
        }
        let Some(color) = winning_color(&grid) else {
            continue;
        };
        hyperboard.winner = Some(color);
        commands.entity(entity).with_child((
            WinnerOverlay,
            Sprite::from_color(color, Vec2::splat(HYPERBOARD_SIZE)),
            Transform::from_xyz(0.0, 0.0, OVERLAY_Z),
        ));
    }
}

/// Decides what each board may do, through its `BoardControl`.
///
/// For now every board can be clicked except won and full ones, and nothing
/// can be clicked once the game is won. Won boards always show their winner
/// square; full boards without a winner look the same as before.
fn control_boards(
    hyperboards: Query<(&Hyperboard, &Children)>,
    mut boards: Query<(&Board, &mut BoardControl)>,
) {
    for (hyperboard, children) in &hyperboards {
        for child in children.iter() {
            if let Ok((board, mut control)) = boards.get_mut(child) {
                control.set_if_neq(BoardControl {
                    clickable: hyperboard.winner.is_none()
                        && board.winner().is_none()
                        && !board.is_full(),
                    show_winner_overlay: true,
                });
            }
        }
    }
}

/// Position of the center of the board at (`col`, `row`), relative to the
/// hyperboard's center. Row 0 is at the top.
fn board_position(col: usize, row: usize) -> Vec2 {
    let step = BOARD_SIZE * BOARD_SCALE + BOARD_GAP;
    let offset = (GRID_SIZE as f32 - 1.0) * step / 2.0;
    Vec2::new(col as f32 * step - offset, offset - row as f32 * step)
}

#[cfg(test)]
mod tests;
