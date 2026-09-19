//! The board's game rules, as plain functions: no ECS, easy to test.

use super::meta::constant::GRID_SIZE;

/// The value with a full row, column or diagonal, if any.
/// `grid[row][col]` is the value at that position, `None` if empty.
///
/// It is *generic*: `T` can be any type that can be copied and compared.
/// The game uses player entities (`T = Entity`), but the rule doesn't need to
/// know what a player is, and tests can use plain numbers.
///
/// Public so the hyperboard can use the same rule on its grid of boards.
pub fn three_in_a_row<T: Copy + PartialEq>(
    grid: &[[Option<T>; GRID_SIZE]; GRID_SIZE],
) -> Option<T> {
    let n = GRID_SIZE;
    let rows = (0..n).map(|row| (0..n).map(|col| grid[row][col]).collect::<Vec<_>>());
    let columns = (0..n).map(|col| (0..n).map(|row| grid[row][col]).collect());
    let diagonal = (0..n).map(|i| grid[i][i]).collect();
    let anti_diagonal = (0..n).map(|i| grid[i][n - 1 - i]).collect();

    rows.chain(columns)
        .chain([diagonal, anti_diagonal])
        .find_map(|line: Vec<Option<T>>| {
            // `?` returns `None` from this closure when the first cell is empty.
            let first = line[0]?;
            line.iter().all(|&c| c == Some(first)).then_some(first)
        })
}
