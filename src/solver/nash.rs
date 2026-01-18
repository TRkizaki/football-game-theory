use super::game::{GameSolution, GameSolver, GameError};

/// Nash equilibrium detector for two-player zero-sum games.
///
/// In a zero-sum game, the minimax solution is always a Nash equilibrium.
#[derive(Debug)]
pub struct NashEquilibrium {
    solution: GameSolution,
}

impl NashEquilibrium {
    /// Finds the Nash equilibrium for a two-player zero-sum game.
    pub fn find(payoff_matrix: Vec<Vec<f64>>) -> Result<Self, GameError> {
        let solver = GameSolver::new(payoff_matrix)?;
        let solution = solver.solve()?;
        Ok(Self { solution })
    }

    /// Returns the equilibrium strategy for the Row player (maximizer).
    pub fn row_strategy(&self) -> &[f64] {
        &self.solution.row_strategy
    }

    /// Returns the equilibrium strategy for the Column player (minimizer).
    pub fn col_strategy(&self) -> &[f64] {
        &self.solution.col_strategy
    }

    /// Returns the value of the game at equilibrium.
    pub fn value(&self) -> f64 {
        self.solution.game_value
    }

    /// Checks if a given strategy pair is an epsilon-Nash equilibrium.
    ///
    /// An epsilon-Nash equilibrium means neither player can improve
    /// their expected payoff by more than epsilon by unilaterally deviating.
    pub fn is_epsilon_nash(
        payoff_matrix: &[Vec<f64>],
        row_strategy: &[f64],
        col_strategy: &[f64],
        epsilon: f64,
    ) -> bool {
        let num_rows = payoff_matrix.len();
        let num_cols = payoff_matrix[0].len();

        // Calculate current expected payoff
        let current_payoff = Self::expected_payoff(payoff_matrix, row_strategy, col_strategy);

        // Check if Row player can improve by deviating to any pure strategy
        for i in 0..num_rows {
            let mut pure_strategy = vec![0.0; num_rows];
            pure_strategy[i] = 1.0;
            let deviation_payoff = Self::expected_payoff(payoff_matrix, &pure_strategy, col_strategy);
            if deviation_payoff > current_payoff + epsilon {
                return false;
            }
        }

        // Check if Column player can improve by deviating to any pure strategy
        for j in 0..num_cols {
            let mut pure_strategy = vec![0.0; num_cols];
            pure_strategy[j] = 1.0;
            let deviation_payoff = Self::expected_payoff(payoff_matrix, row_strategy, &pure_strategy);
            // Column wants to minimize, so check if they can reduce payoff
            if deviation_payoff < current_payoff - epsilon {
                return false;
            }
        }

        true
    }

    /// Calculates expected payoff for given strategies.
    fn expected_payoff(
        payoff_matrix: &[Vec<f64>],
        row_strategy: &[f64],
        col_strategy: &[f64],
    ) -> f64 {
        let mut payoff = 0.0;
        for (i, row) in payoff_matrix.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                payoff += row_strategy[i] * col_strategy[j] * val;
            }
        }
        payoff
    }

    /// Returns the full solution details.
    pub fn solution(&self) -> &GameSolution {
        &self.solution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nash_equilibrium_verification() {
        let matrix = vec![
            vec![1.0, -1.0],
            vec![-1.0, 1.0],
        ];

        let nash = NashEquilibrium::find(matrix.clone()).unwrap();

        // The solution should be an epsilon-Nash equilibrium
        assert!(NashEquilibrium::is_epsilon_nash(
            &matrix,
            nash.row_strategy(),
            nash.col_strategy(),
            0.01
        ));
    }
}
