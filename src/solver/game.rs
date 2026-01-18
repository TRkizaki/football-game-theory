use super::simplex::{Simplex, SimplexError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Empty payoff matrix")]
    EmptyMatrix,
    #[error("Inconsistent row lengths in payoff matrix")]
    InconsistentRows,
    #[error("Solver error: {0}")]
    SolverError(#[from] SimplexError),
}

/// Result of solving a two-player zero-sum game.
#[derive(Debug, Clone)]
pub struct GameSolution {
    /// Optimal mixed strategy for Row player (maximizer)
    pub row_strategy: Vec<f64>,
    /// Optimal mixed strategy for Column player (minimizer)
    pub col_strategy: Vec<f64>,
    /// Value of the game
    pub game_value: f64,
}

/// Solver for two-player zero-sum games using linear programming.
///
/// Finds the optimal mixed strategies and game value using the Simplex method.
#[derive(Debug, Clone)]
pub struct GameSolver {
    payoff_matrix: Vec<Vec<f64>>,
    num_rows: usize,
    num_cols: usize,
}

impl GameSolver {
    /// Creates a new game solver with the given payoff matrix.
    ///
    /// The matrix is from Row player's perspective (Row wants to maximize).
    pub fn new(payoff_matrix: Vec<Vec<f64>>) -> Result<Self, GameError> {
        if payoff_matrix.is_empty() {
            return Err(GameError::EmptyMatrix);
        }

        let num_rows = payoff_matrix.len();
        let num_cols = payoff_matrix[0].len();

        if num_cols == 0 {
            return Err(GameError::EmptyMatrix);
        }

        for row in &payoff_matrix {
            if row.len() != num_cols {
                return Err(GameError::InconsistentRows);
            }
        }

        Ok(Self {
            payoff_matrix,
            num_rows,
            num_cols,
        })
    }

    /// Solves the game and returns optimal strategies for both players.
    pub fn solve(&self) -> Result<GameSolution, GameError> {
        // Shift the matrix to ensure all values are positive
        let shift = self.calculate_shift();
        let shifted_matrix = self.shift_matrix(shift);

        // Solve for Row player's strategy
        let row_strategy = self.solve_row_player(&shifted_matrix)?;

        // Solve for Column player's strategy
        let col_strategy = self.solve_col_player(&shifted_matrix)?;

        // Calculate game value
        let game_value = self.calculate_game_value(&row_strategy) ;

        Ok(GameSolution {
            row_strategy,
            col_strategy,
            game_value,
        })
    }

    /// Calculates the shift needed to make all payoffs positive.
    fn calculate_shift(&self) -> f64 {
        let min_val = self.payoff_matrix
            .iter()
            .flat_map(|row| row.iter())
            .cloned()
            .fold(f64::INFINITY, f64::min);

        if min_val <= 0.0 {
            -min_val + 1.0
        } else {
            0.0
        }
    }

    /// Shifts all matrix values by the given amount.
    fn shift_matrix(&self, shift: f64) -> Vec<Vec<f64>> {
        self.payoff_matrix
            .iter()
            .map(|row| row.iter().map(|&v| v + shift).collect())
            .collect()
    }

    /// Solves for Row player's optimal mixed strategy.
    ///
    /// Row player wants to maximize the minimum expected payoff.
    /// We solve the dual problem (Column player's LP) and extract Row's strategy
    /// from the dual variables.
    ///
    /// Column player's primal:
    /// Maximize: sum(z_j)
    /// Subject to: sum(z_j * a_ij) <= 1 for all i
    ///            z_j >= 0
    ///
    /// The dual of this gives us Row player's strategy.
    fn solve_row_player(&self, matrix: &[Vec<f64>]) -> Result<Vec<f64>, GameError> {
        // We solve via the Column player's problem and use duality.
        // For Row player with shifted positive matrix:
        // The value v = 1 / sum(z_j) where z is Column's optimal solution.
        // Row's strategy comes from the shadow prices of the Column LP constraints.

        // Alternative approach: solve Row's problem directly by using
        // the fact that with positive payoffs, we can set up:
        // Maximize: sum(y_i)  [this is 1/v, and we want to maximize the game value]
        // Subject to: sum(y_i * a_ij) >= 1 for all j
        //
        // But Simplex needs <= constraints, so we solve the Column problem
        // and derive Row strategy from complementary slackness.

        // Simpler approach: solve Column's LP, then solve for Row's best response
        // that makes Column indifferent.

        // Actually, let's use the relationship between primal and dual:
        // Solve Column's problem to get game value, then solve Row's directly.

        // For a proper solution, we need to solve:
        // Row: minimize sum(y_i) subject to A^T * y >= 1, y >= 0
        // This is equivalent to: maximize sum(y_i) subject to -A^T * y <= -1
        // But our Simplex requires b >= 0.

        // Solution: Use the Column player's dual.
        // Column's primal: max sum(z) s.t. Az <= 1
        // Column's dual: min sum(w) s.t. A^T w >= 1, w >= 0
        // This dual IS Row's problem!

        // So we get Row's strategy by solving Column's problem and reading dual vars.
        // Since we don't have dual variable extraction, let's solve directly:

        // Transform Row's problem:
        // min sum(y) s.t. A^T y >= 1
        // Let's use: find y such that for each column j: sum_i(y_i * a_ij) >= 1

        // We can solve this by finding the maximum over columns for each row weight.
        // Use iterative approach or solve the equivalent Column problem.

        // Practical solution: solve Column's LP, compute game value,
        // then find Row's strategy that achieves this value.

        let col_solution = self.solve_col_player_internal(matrix)?;
        let sum_z: f64 = col_solution.iter().sum();
        let game_value_shifted = 1.0 / sum_z;

        // Now find Row's strategy by solving:
        // For Row: we want p such that min_j sum_i(p_i * a_ij) = game_value_shifted
        // This means: sum_i(p_i * a_ij) >= v for all j, sum(p_i) = 1

        // Reformulate: let y_i = p_i / v, then sum(y_i * a_ij) >= 1 for all j
        // and sum(y_i) = 1/v

        // We need to find which constraints are tight (active) at optimum.
        // At Nash equilibrium, Column mixes over strategies where Row is indifferent.

        // Find columns where Column plays with positive probability
        let active_cols: Vec<usize> = col_solution
            .iter()
            .enumerate()
            .filter(|&(_, &z)| z > 1e-9)
            .map(|(j, _)| j)
            .collect();

        // Row's strategy must make Column indifferent among active columns.
        // For active columns j: sum_i(p_i * a_ij) = v (all equal)
        // For inactive columns: sum_i(p_i * a_ij) >= v

        // Solve the system of linear equations for active columns.
        let num_active = active_cols.len();

        if num_active == 0 {
            return Err(GameError::SolverError(SimplexError::Infeasible));
        }

        if num_active == 1 {
            // Only one active column, Row plays pure best response
            let j = active_cols[0];
            let best_row = (0..self.num_rows)
                .max_by(|&i1, &i2| matrix[i1][j].partial_cmp(&matrix[i2][j]).unwrap())
                .unwrap();
            let mut strategy = vec![0.0; self.num_rows];
            strategy[best_row] = 1.0;
            return Ok(strategy);
        }

        // For multiple active columns, solve using the constraint that
        // expected payoffs are equal for all active columns.
        // We use: sum_i(p_i * a_ij) = v for active j, and sum(p_i) = 1

        // This is a system of linear equations. Use Gaussian elimination.
        let strategy = self.solve_indifference_system(matrix, &active_cols, game_value_shifted)?;

        Ok(strategy)
    }

    /// Solves the system to find Row's strategy that makes Column indifferent.
    fn solve_indifference_system(
        &self,
        matrix: &[Vec<f64>],
        active_cols: &[usize],
        _game_value: f64,
    ) -> Result<Vec<f64>, GameError> {
        // We need: sum_i(p_i * a_ij) = same for all active j
        //          sum(p_i) = 1
        //          p_i >= 0

        // Set up augmented matrix for: [A_active^T | 1] * [p | -v]^T = 0, sum(p) = 1
        // Simpler: use the fact that at equilibrium, expected payoffs are equal.

        // For 2-player zero-sum, we can compute directly.
        // Equal payoff condition: a_i1 * p + a_i2 * (1-p) for 2x2 case.

        // General case: solve using least squares or direct system.

        let n = self.num_rows;
        let m = active_cols.len();

        // Build system: we want p such that A_active^T * p has all equal entries
        // This means: a_ij1 * p_i = a_ij2 * p_i for all pairs j1, j2
        // => sum_i (a_ij1 - a_ij2) * p_i = 0

        // Plus constraint: sum(p_i) = 1

        // Build augmented matrix
        let mut aug: Vec<Vec<f64>> = Vec::new();

        // Difference equations (m-1 equations)
        for k in 1..m {
            let j0 = active_cols[0];
            let jk = active_cols[k];
            let row: Vec<f64> = (0..n)
                .map(|i| matrix[i][j0] - matrix[i][jk])
                .collect();
            aug.push(row);
        }

        // Sum to 1 constraint
        aug.push(vec![1.0; n]);

        // RHS
        let mut rhs: Vec<f64> = vec![0.0; m - 1];
        rhs.push(1.0);

        // Solve using Gaussian elimination
        let solution = gaussian_elimination(&mut aug, &mut rhs, n)?;

        // Ensure non-negative (clamp small negatives from numerical error)
        let strategy: Vec<f64> = solution.iter().map(|&x| x.max(0.0)).collect();

        // Renormalize
        let sum: f64 = strategy.iter().sum();
        if sum < 1e-10 {
            return Err(GameError::SolverError(SimplexError::Infeasible));
        }

        Ok(strategy.iter().map(|&x| x / sum).collect())
    }

    /// Internal Column player solver that returns raw z values.
    fn solve_col_player_internal(&self, matrix: &[Vec<f64>]) -> Result<Vec<f64>, GameError> {
        let c: Vec<f64> = vec![1.0; self.num_cols];
        let a: Vec<Vec<f64>> = matrix.to_vec();
        let b: Vec<f64> = vec![1.0; self.num_rows];

        let mut solver = Simplex::new(&c, &a, &b)?;
        let (_, z) = solver.solve()?;

        Ok(z)
    }

    /// Solves for Column player's optimal mixed strategy.
    ///
    /// Column player wants to minimize the maximum expected loss.
    /// Maximize: w (from Col player's view, minimize the max)
    /// Subject to: sum(q_j * a_ij) <= w for all i
    ///            sum(q_j) = 1
    ///            q_j >= 0
    ///
    /// Reformulated with z_j = q_j / w:
    /// Maximize: sum(z_j)
    /// Subject to: sum(z_j * a_ij) <= 1 for all i
    ///            z_j >= 0
    fn solve_col_player(&self, matrix: &[Vec<f64>]) -> Result<Vec<f64>, GameError> {
        // For Column player: maximize sum(z_j)
        // Subject to: A * z <= 1

        let c: Vec<f64> = vec![1.0; self.num_cols];

        let a: Vec<Vec<f64>> = matrix.to_vec();

        let b: Vec<f64> = vec![1.0; self.num_rows];

        let mut solver = Simplex::new(&c, &a, &b)?;
        let (_, z) = solver.solve()?;

        // Convert back: w = 1 / sum(z_j), q_j = z_j * w
        let sum_z: f64 = z.iter().sum();
        let strategy: Vec<f64> = z.iter().map(|&zj| zj / sum_z).collect();

        Ok(strategy)
    }

    /// Calculates the game value given Row player's strategy.
    fn calculate_game_value(&self, row_strategy: &[f64]) -> f64 {
        // Game value is the minimum expected payoff over all Column strategies
        (0..self.num_cols)
            .map(|j| {
                (0..self.num_rows)
                    .map(|i| row_strategy[i] * self.payoff_matrix[i][j])
                    .sum::<f64>()
            })
            .fold(f64::INFINITY, f64::min)
    }

    /// Returns the payoff matrix.
    pub fn payoff_matrix(&self) -> &Vec<Vec<f64>> {
        &self.payoff_matrix
    }

    /// Calculates expected payoff for given strategies.
    pub fn expected_payoff(&self, row_strategy: &[f64], col_strategy: &[f64]) -> f64 {
        let mut payoff = 0.0;
        for i in 0..self.num_rows {
            for j in 0..self.num_cols {
                payoff += row_strategy[i] * col_strategy[j] * self.payoff_matrix[i][j];
            }
        }
        payoff
    }
}

/// Solves a system of linear equations using Gaussian elimination with partial pivoting.
fn gaussian_elimination(
    a: &mut [Vec<f64>],
    b: &mut [f64],
    n: usize,
) -> Result<Vec<f64>, GameError> {
    let m = a.len(); // number of equations

    if m == 0 || n == 0 {
        return Err(GameError::SolverError(SimplexError::InvalidDimensions));
    }

    // Forward elimination with partial pivoting
    for col in 0..m.min(n) {
        // Find pivot
        let mut max_row = col;
        let mut max_val = if col < a.len() { a[col][col].abs() } else { 0.0 };

        for row in (col + 1)..m {
            if col < a[row].len() && a[row][col].abs() > max_val {
                max_val = a[row][col].abs();
                max_row = row;
            }
        }

        if max_val < 1e-12 {
            continue; // Skip this column (singular or underdetermined)
        }

        // Swap rows
        a.swap(col, max_row);
        b.swap(col, max_row);

        // Eliminate below
        for row in (col + 1)..m {
            if col < a[row].len() && a[col][col].abs() > 1e-12 {
                let factor = a[row][col] / a[col][col];
                for j in col..n {
                    if j < a[row].len() && j < a[col].len() {
                        a[row][j] -= factor * a[col][j];
                    }
                }
                b[row] -= factor * b[col];
            }
        }
    }

    // Back substitution
    let mut x = vec![0.0; n];

    for i in (0..m.min(n)).rev() {
        if i < a.len() && i < a[i].len() && a[i][i].abs() > 1e-12 {
            let mut sum = b[i];
            for j in (i + 1)..n {
                if j < a[i].len() {
                    sum -= a[i][j] * x[j];
                }
            }
            x[i] = sum / a[i][i];
        }
    }

    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_matching_pennies() {
        // Classic matching pennies game
        // Row wins if both match, Column wins if different
        let matrix = vec![
            vec![1.0, -1.0],
            vec![-1.0, 1.0],
        ];

        let solver = GameSolver::new(matrix).unwrap();
        let solution = solver.solve().unwrap();

        // Both players should play 50-50
        assert_relative_eq!(solution.row_strategy[0], 0.5, epsilon = 0.01);
        assert_relative_eq!(solution.row_strategy[1], 0.5, epsilon = 0.01);
        assert_relative_eq!(solution.col_strategy[0], 0.5, epsilon = 0.01);
        assert_relative_eq!(solution.col_strategy[1], 0.5, epsilon = 0.01);
        assert_relative_eq!(solution.game_value, 0.0, epsilon = 0.01);
    }

    #[test]
    fn test_asymmetric_game() {
        // Asymmetric 2x3 game
        let matrix = vec![
            vec![3.0, -1.0, 2.0],
            vec![-2.0, 4.0, 1.0],
        ];

        let solver = GameSolver::new(matrix).unwrap();
        let solution = solver.solve().unwrap();

        // Verify strategies sum to 1
        let row_sum: f64 = solution.row_strategy.iter().sum();
        let col_sum: f64 = solution.col_strategy.iter().sum();
        assert_relative_eq!(row_sum, 1.0, epsilon = 0.01);
        assert_relative_eq!(col_sum, 1.0, epsilon = 0.01);

        // All probabilities should be non-negative
        for &p in &solution.row_strategy {
            assert!(p >= -0.01);
        }
        for &q in &solution.col_strategy {
            assert!(q >= -0.01);
        }
    }
}
