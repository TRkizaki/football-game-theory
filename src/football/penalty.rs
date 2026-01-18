use super::payoff::PayoffMatrix;
use crate::solver::game::{GameSolver, GameSolution, GameError};

/// Represents the direction of a kick or dive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Center,
    Right,
}

impl Direction {
    /// Returns all possible directions.
    pub fn all() -> &'static [Direction] {
        &[Direction::Left, Direction::Center, Direction::Right]
    }

    /// Returns the direction name.
    pub fn name(&self) -> &'static str {
        match self {
            Direction::Left => "Left",
            Direction::Center => "Center",
            Direction::Right => "Right",
        }
    }

    /// Returns the index for matrix operations.
    pub fn index(&self) -> usize {
        match self {
            Direction::Left => 0,
            Direction::Center => 1,
            Direction::Right => 2,
        }
    }

    /// Creates a direction from an index.
    pub fn from_index(index: usize) -> Option<Direction> {
        match index {
            0 => Some(Direction::Left),
            1 => Some(Direction::Center),
            2 => Some(Direction::Right),
            _ => None,
        }
    }
}

/// Result of analyzing a penalty kick scenario.
#[derive(Debug, Clone)]
pub struct PenaltyAnalysis {
    /// Optimal strategy for the kicker
    pub kicker_strategy: Vec<(Direction, f64)>,
    /// Optimal strategy for the goalkeeper
    pub goalkeeper_strategy: Vec<(Direction, f64)>,
    /// Expected goal probability at equilibrium
    pub goal_probability: f64,
    /// The payoff matrix used
    pub payoff_matrix: PayoffMatrix,
}

impl PenaltyAnalysis {
    /// Formats the kicker's strategy as a readable string.
    pub fn kicker_strategy_string(&self) -> String {
        self.kicker_strategy
            .iter()
            .filter(|(_, prob)| *prob > 0.001)
            .map(|(dir, prob)| format!("{}: {:.1}%", dir.name(), prob * 100.0))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Formats the goalkeeper's strategy as a readable string.
    pub fn goalkeeper_strategy_string(&self) -> String {
        self.goalkeeper_strategy
            .iter()
            .filter(|(_, prob)| *prob > 0.001)
            .map(|(dir, prob)| format!("{}: {:.1}%", dir.name(), prob * 100.0))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Penalty kick game analyzer.
///
/// Models penalty kicks as a two-player zero-sum game and finds
/// the Nash equilibrium strategies using linear programming.
#[derive(Debug)]
pub struct PenaltyKick {
    payoff_matrix: PayoffMatrix,
}

impl PenaltyKick {
    /// Creates a new PK analyzer with the given success rate matrix.
    ///
    /// # Arguments
    /// * `success_rates` - 3x3 matrix of goal success probabilities
    ///   - Rows: Kicker's direction (Left, Center, Right)
    ///   - Columns: Goalkeeper's dive direction (Left, Center, Right)
    ///   - Values: Probability of scoring (0.0 to 1.0)
    pub fn new(success_rates: Vec<Vec<f64>>) -> Result<Self, super::payoff::PayoffError> {
        let row_labels = vec![
            "Kick Left".into(),
            "Kick Center".into(),
            "Kick Right".into(),
        ];
        let col_labels = vec![
            "GK Left".into(),
            "GK Center".into(),
            "GK Right".into(),
        ];

        let payoff_matrix = PayoffMatrix::new(success_rates, row_labels, col_labels)?;

        Ok(Self { payoff_matrix })
    }

    /// Creates a PK analyzer with default success rates based on real data.
    ///
    /// Data source: Palacios-Huerta (2003) empirical PK statistics
    pub fn with_default_data() -> Self {
        // Real-world PK success rates (from kicker's perspective)
        // GK dives: Left, Center, Right
        let success_rates = vec![
            vec![0.58, 0.93, 0.95], // Kick Left
            vec![0.83, 0.44, 0.83], // Kick Center
            vec![0.93, 0.90, 0.60], // Kick Right
        ];

        Self::new(success_rates).expect("Default data should be valid")
    }

    /// Analyzes the penalty kick scenario and returns optimal strategies.
    pub fn analyze(&self) -> Result<PenaltyAnalysis, GameError> {
        // Convert success probabilities to expected payoffs
        // For kicker: goal = +1, save = -1
        let payoff_values = self.payoff_matrix.to_expected_payoff();

        let solver = GameSolver::new(payoff_values)?;
        let solution: GameSolution = solver.solve()?;

        // Convert raw strategies to Direction-probability pairs
        let kicker_strategy: Vec<(Direction, f64)> = solution
            .row_strategy
            .iter()
            .enumerate()
            .filter_map(|(i, &prob)| Direction::from_index(i).map(|d| (d, prob)))
            .collect();

        let goalkeeper_strategy: Vec<(Direction, f64)> = solution
            .col_strategy
            .iter()
            .enumerate()
            .filter_map(|(i, &prob)| Direction::from_index(i).map(|d| (d, prob)))
            .collect();

        // Convert game value back to probability
        // Game value is in [-1, 1], convert back to [0, 1]
        let goal_probability = (solution.game_value + 1.0) / 2.0;

        Ok(PenaltyAnalysis {
            kicker_strategy,
            goalkeeper_strategy,
            goal_probability,
            payoff_matrix: self.payoff_matrix.clone(),
        })
    }

    /// Returns the payoff matrix.
    pub fn payoff_matrix(&self) -> &PayoffMatrix {
        &self.payoff_matrix
    }

    /// Calculates the expected goal probability for given strategies.
    pub fn expected_goal_probability(
        &self,
        kicker_strategy: &[f64],
        goalkeeper_strategy: &[f64],
    ) -> f64 {
        let mut prob = 0.0;
        let matrix = self.payoff_matrix.matrix();

        for (i, &kick_prob) in kicker_strategy.iter().enumerate() {
            for (j, &gk_prob) in goalkeeper_strategy.iter().enumerate() {
                prob += kick_prob * gk_prob * matrix[i][j];
            }
        }

        prob
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_direction_conversion() {
        for dir in Direction::all() {
            let index = dir.index();
            let recovered = Direction::from_index(index).unwrap();
            assert_eq!(*dir, recovered);
        }
    }

    #[test]
    fn test_penalty_analysis() {
        let pk = PenaltyKick::with_default_data();
        let analysis = pk.analyze().unwrap();

        // Verify strategies sum to 1
        let kicker_sum: f64 = analysis.kicker_strategy.iter().map(|(_, p)| p).sum();
        let gk_sum: f64 = analysis.goalkeeper_strategy.iter().map(|(_, p)| p).sum();

        assert_relative_eq!(kicker_sum, 1.0, epsilon = 0.01);
        assert_relative_eq!(gk_sum, 1.0, epsilon = 0.01);

        // Goal probability should be between 0 and 1
        assert!(analysis.goal_probability >= 0.0);
        assert!(analysis.goal_probability <= 1.0);

        // With real data, goal probability is typically around 0.75-0.80
        assert!(analysis.goal_probability > 0.5);
    }

    #[test]
    fn test_expected_goal_probability() {
        let pk = PenaltyKick::with_default_data();

        // Pure strategy: kick left, GK dives left
        let kick = vec![1.0, 0.0, 0.0];
        let gk = vec![1.0, 0.0, 0.0];

        let prob = pk.expected_goal_probability(&kick, &gk);
        assert_relative_eq!(prob, 0.58, epsilon = 0.001);
    }
}
