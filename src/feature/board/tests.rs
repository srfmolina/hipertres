use super::*;

#[test]
fn center_cell_is_at_origin() {
    assert_eq!(cell_position(1, 1), Vec2::ZERO);
}

#[test]
fn top_left_cell_is_up_and_left() {
    let step = CELL_SIZE + CELL_GAP;
    assert_eq!(cell_position(0, 0), Vec2::new(-step, step));
}
