fn solve_n_queens(n: usize) -> Vec<Vec<String>> {
    // Will hold all valid solutions.
    // Each solution is a vector of strings, where each string is a row of the board.
    let mut result = Vec::new();

    // Initialise an n x n board filled with '.' (empty squares).
    let mut board: Vec<Vec<char>> = vec![vec!['.'; n]; n];

    /// Backtracking helper that fills the board row by row.
    ///
    /// * `board` - mutable reference to the current board state
    /// * `row`   - the row we are currently trying to place a queen in
    /// * `n`     - size of the board (passed for convenience)
    /// * `result`- accumulates solutions (each solution is a copy of the board as Vec<String>)
    fn bt(
        board: &mut Vec<Vec<char>>,
        row: usize,
        n: usize,
        result: &mut Vec<Vec<String>>,
    ) {
        // Base case: all rows have a queen -> we have a full valid solution.
        if row == n {
            // Convert each row (Vec<char>) into a String and collect them into a solution.
            let solution: Vec<String> = board
                .iter()
                .map(|r| r.iter().collect()) // char iter to String
                .collect();
            result.push(solution);
            return;
        }

        // Try every column in the current row.
        for col in 0..n {
            // If placing a queen at (row, col) does not attack any previously placed queen
            if is_safe(board, row, col, n) {
                // Place the queen
                board[row][col] = 'Q';

                // Move on to the next row
                bt(board, row + 1, n, result);

                // Backtrack: remove the queen so we can try the next column
                board[row][col] = '.';
            }
        }
    }

    /// Checks whether a queen can be safely placed at (row, col) given the current board.
    ///
    /// We only need to check rows **above** the current row because queens are placed
    /// one row at a time and no rows below exist yet.
    fn is_safe(board: &[Vec<char>], row: usize, col: usize, n: usize) -> bool {
        // Check all previous rows (0..row) for conflicts.
        let no_column_conflict = (0..row).all(|r| board[r][col] != 'Q'); // same column

        // For diagonal conflicts we need to look at two diagonals:
        // - Upper‑left  (row - k, col - k)
        // - Upper‑right (row - k, col + k)
        //
        // We iterate over previous rows with `enumerate` to get both the row index `r`
        // and a sequential index `i` starting at 0. Instead of i we use the row difference
        // `c_offset = row - r` which is the number of rows between the current row and row r.
        let no_diagonal_conflict = (0..row)
            .enumerate()
            .all(|(i, r)| {
                let c_offset = row - r; // distance in rows; also the required column offset

                // Check the upper‑left diagonal: (r, col - c_offset)
                // We must ensure the column index does not underflow (col >= c_offset).
                let upper_left = col < c_offset  // if col is too far left, no square exists
                    || board[r][col - c_offset] != 'Q';

                // Check the upper‑right diagonal: (r, col + c_offset)
                // We must ensure the column index is within bounds (col + c_offset < n).
                let upper_right = col + c_offset >= n  // out of right bound -> safe
                    || board[r][col + c_offset] != 'Q';

                upper_left && upper_right
            });

        // The placement is safe only if there is no conflict in either direction.
        no_column_conflict && no_diagonal_conflict
    }

    // Start backtracking from row 0 with an empty board.
    bt(&mut board, 0, n, &mut result);

    result
}

#[cfg(test)]
mod test {
    use crate::algo::n_queen::solve_n_queens;

    #[test]
    fn test_solve_n_queens() {
        let result = solve_n_queens(9);
        for line in result {
            println!("{:?}", line);
        }
    }
}

