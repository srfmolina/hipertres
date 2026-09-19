//! The board's game rules, as plain functions: no ECS, easy to test.

use bevy::prelude::*;

use super::meta::constant::GRID_SIZE;

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
