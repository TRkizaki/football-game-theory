use thiserror::Error;

#[derive(Error, Debug)]
pub enum PayoffError {
    #[error("Matrix dimensions mismatch")]
    DimensionMismatch,
    #[error("Invalid probability: {0}")]
    InvalidProbability(f64),
}

/// Represents a payoff matrix for a two-player game.
///
/// In the context of penalty kicks:
/// - Rows represent kicker's strategies
/// - Columns represent goalkeeper's strategies
/// - Values represent success probability (kicker's perspective)
#[derive(Debug, Clone)]
pub struct PayoffMatrix {
    matrix: Vec<Vec<f64>>,
    row_labels: Vec<String>,
    col_labels: Vec<String>,
}

impl PayoffMatrix {
    /// Creates a new payoff matrix with the given data and labels.
    pub fn new(
        matrix: Vec<Vec<f64>>,
        row_labels: Vec<String>,
        col_labels: Vec<String>,
    ) -> Result<Self, PayoffError> {
        if matrix.is_empty() {
            return Ok(Self {
                matrix,
                row_labels,
                col_labels,
            });
        }

        let num_cols = matrix[0].len();
        for row in &matrix {
            if row.len() != num_cols {
                return Err(PayoffError::DimensionMismatch);
            }
        }

        if matrix.len() != row_labels.len() || num_cols != col_labels.len() {
            return Err(PayoffError::DimensionMismatch);
        }

        Ok(Self {
            matrix,
            row_labels,
            col_labels,
        })
    }

    /// Creates a payoff matrix from raw success probabilities.
    ///
    /// # Arguments
    /// * `success_rates` - 2D array of goal success probabilities (0.0 to 1.0)
    pub fn from_success_rates(success_rates: Vec<Vec<f64>>) -> Result<Self, PayoffError> {
        for row in &success_rates {
            for &prob in row {
                if !(0.0..=1.0).contains(&prob) {
                    return Err(PayoffError::InvalidProbability(prob));
                }
            }
        }

        let num_rows = success_rates.len();
        let num_cols = if num_rows > 0 { success_rates[0].len() } else { 0 };

        let row_labels = (0..num_rows).map(|i| format!("Row {}", i)).collect();
        let col_labels = (0..num_cols).map(|j| format!("Col {}", j)).collect();

        Self::new(success_rates, row_labels, col_labels)
    }

    /// Returns the raw payoff matrix.
    pub fn matrix(&self) -> &Vec<Vec<f64>> {
        &self.matrix
    }

    /// Returns the payoff for a specific strategy combination.
    pub fn get(&self, row: usize, col: usize) -> Option<f64> {
        self.matrix.get(row).and_then(|r| r.get(col).copied())
    }

    /// Returns the number of rows (Row player's strategies).
    pub fn num_rows(&self) -> usize {
        self.matrix.len()
    }

    /// Returns the number of columns (Column player's strategies).
    pub fn num_cols(&self) -> usize {
        if self.matrix.is_empty() {
            0
        } else {
            self.matrix[0].len()
        }
    }

    /// Returns the row labels.
    pub fn row_labels(&self) -> &[String] {
        &self.row_labels
    }

    /// Returns the column labels.
    pub fn col_labels(&self) -> &[String] {
        &self.col_labels
    }

    /// Converts success probabilities to expected payoffs.
    ///
    /// For PK: goal = +1, save = -1 (from kicker's perspective)
    pub fn to_expected_payoff(&self) -> Vec<Vec<f64>> {
        self.matrix
            .iter()
            .map(|row| {
                row.iter()
                    .map(|&prob| 2.0 * prob - 1.0) // Maps [0,1] to [-1,1]
                    .collect()
            })
            .collect()
    }

    /// Pretty-prints the payoff matrix.
    pub fn display(&self) -> String {
        let mut output = String::new();

        // Header row
        output.push_str(&format!("{:>12}", ""));
        for label in &self.col_labels {
            output.push_str(&format!("{:>12}", label));
        }
        output.push('\n');

        // Data rows
        for (i, row) in self.matrix.iter().enumerate() {
            output.push_str(&format!("{:>12}", &self.row_labels[i]));
            for val in row {
                output.push_str(&format!("{:>12.3}", val));
            }
            output.push('\n');
        }

        output
    }
}

impl Default for PayoffMatrix {
    fn default() -> Self {
        Self {
            matrix: Vec::new(),
            row_labels: Vec::new(),
            col_labels: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payoff_matrix_creation() {
        let matrix = vec![
            vec![0.58, 0.93, 0.95],
            vec![0.83, 0.44, 0.83],
            vec![0.93, 0.90, 0.60],
        ];

        let row_labels = vec!["Kick Left".into(), "Kick Center".into(), "Kick Right".into()];
        let col_labels = vec!["GK Left".into(), "GK Center".into(), "GK Right".into()];

        let payoff = PayoffMatrix::new(matrix.clone(), row_labels, col_labels).unwrap();

        assert_eq!(payoff.num_rows(), 3);
        assert_eq!(payoff.num_cols(), 3);
        assert_eq!(payoff.get(0, 0), Some(0.58));
        assert_eq!(payoff.get(1, 2), Some(0.83));
    }

    #[test]
    fn test_to_expected_payoff() {
        let matrix = vec![vec![0.5, 1.0], vec![0.0, 0.75]];
        let payoff = PayoffMatrix::from_success_rates(matrix).unwrap();
        let expected = payoff.to_expected_payoff();

        assert!((expected[0][0] - 0.0).abs() < 0.001); // 0.5 -> 0.0
        assert!((expected[0][1] - 1.0).abs() < 0.001); // 1.0 -> 1.0
        assert!((expected[1][0] - (-1.0)).abs() < 0.001); // 0.0 -> -1.0
        assert!((expected[1][1] - 0.5).abs() < 0.001); // 0.75 -> 0.5
    }
}
