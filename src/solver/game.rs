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
    /// This is converted to an LP:
    /// Maximize: v
    /// Subject to: sum(p_i * a_ij) >= v for all j
    ///            sum(p_i) = 1
    ///            p_i >= 0
    ///
    /// Reformulated with y_i = p_i / v:
    /// Minimize: sum(y_i)
    /// Subject to: sum(y_i * a_ij) >= 1 for all j
    ///            y_i >= 0
    fn solve_row_player(&self, matrix: &[Vec<f64>]) -> Result<Vec<f64>, GameError> {
        // For Row player: minimize sum(y_i) where y_i = p_i / v
        // Subject to: A^T * y >= 1 (transposed constraints)

        // Convert to standard maximization form for Simplex
        // Maximize: -sum(y_i) (equivalent to minimize sum)
        // Subject to: -A^T * y <= -1

        let c: Vec<f64> = vec![-1.0; self.num_rows];

        // Transpose and negate for <= form
        let a: Vec<Vec<f64>> = (0..self.num_cols)
            .map(|j| {
                (0..self.num_rows)
                    .map(|i| -matrix[i][j])
                    .collect()
            })
            .collect();

        let b: Vec<f64> = vec![-1.0; self.num_cols];

        let mut solver = Simplex::new(&c, &a, &b)?;
        let (_, y) = solver.solve()?;

        // Convert back: v = 1 / sum(y_i), p_i = y_i * v
        let sum_y: f64 = y.iter().sum();
        let strategy: Vec<f64> = y.iter().map(|&yi| yi / sum_y).collect();

        Ok(strategy)
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
