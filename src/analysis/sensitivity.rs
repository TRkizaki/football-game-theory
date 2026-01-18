use crate::football::penalty::PenaltyKick;
use crate::solver::game::GameError;

/// Result of a sensitivity analysis.
#[derive(Debug, Clone)]
pub struct SensitivityResult {
    /// Parameter that was varied
    pub parameter: String,
    /// Original value
    pub original_value: f64,
    /// New value tested
    pub new_value: f64,
    /// Change in optimal kicker strategy
    pub kicker_strategy_change: Vec<f64>,
    /// Change in optimal goalkeeper strategy
    pub goalkeeper_strategy_change: Vec<f64>,
    /// Change in equilibrium goal probability
    pub goal_probability_change: f64,
}

/// Performs sensitivity analysis on PK payoff matrices.
pub struct SensitivityAnalyzer {
    base_matrix: Vec<Vec<f64>>,
}

impl SensitivityAnalyzer {
    /// Creates a new analyzer with the given base success rate matrix.
    pub fn new(base_matrix: Vec<Vec<f64>>) -> Self {
        Self { base_matrix }
    }

    /// Creates an analyzer with default PK data.
    pub fn with_default_data() -> Self {
        let base_matrix = vec![
            vec![0.58, 0.93, 0.95],
            vec![0.83, 0.44, 0.83],
            vec![0.93, 0.90, 0.60],
        ];
        Self::new(base_matrix)
    }

    /// Analyzes how changing one success rate affects the optimal strategies.
    ///
    /// # Arguments
    /// * `row` - Kick direction index (0=left, 1=center, 2=right)
    /// * `col` - GK direction index
    /// * `delta` - Amount to change the success rate
    pub fn analyze_single_change(
        &self,
        row: usize,
        col: usize,
        delta: f64,
    ) -> Result<SensitivityResult, GameError> {
        // Get base solution
        let base_pk = PenaltyKick::new(self.base_matrix.clone())
            .map_err(|_| GameError::EmptyMatrix)?;
        let base_analysis = base_pk.analyze()?;

        // Create modified matrix
        let mut modified = self.base_matrix.clone();
        let original_value = modified[row][col];
        modified[row][col] = (modified[row][col] + delta).clamp(0.0, 1.0);
        let new_value = modified[row][col];

        // Get modified solution
        let modified_pk = PenaltyKick::new(modified)
            .map_err(|_| GameError::EmptyMatrix)?;
        let modified_analysis = modified_pk.analyze()?;

        // Calculate changes
        let kicker_strategy_change: Vec<f64> = base_analysis
            .kicker_strategy
            .iter()
            .zip(modified_analysis.kicker_strategy.iter())
            .map(|((_, base), (_, modified))| modified - base)
            .collect();

        let goalkeeper_strategy_change: Vec<f64> = base_analysis
            .goalkeeper_strategy
            .iter()
            .zip(modified_analysis.goalkeeper_strategy.iter())
            .map(|((_, base), (_, modified))| modified - base)
            .collect();

        let goal_probability_change =
            modified_analysis.goal_probability - base_analysis.goal_probability;

        Ok(SensitivityResult {
            parameter: format!("Success rate [{},{}]", row, col),
            original_value,
            new_value,
            kicker_strategy_change,
            goalkeeper_strategy_change,
            goal_probability_change,
        })
    }

    /// Performs a full sensitivity analysis by varying each parameter.
    ///
    /// # Arguments
    /// * `delta` - Amount to change each success rate
    pub fn full_analysis(&self, delta: f64) -> Result<Vec<SensitivityResult>, GameError> {
        let mut results = Vec::new();

        for row in 0..3 {
            for col in 0..3 {
                let result = self.analyze_single_change(row, col, delta)?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Finds which parameters the optimal strategy is most sensitive to.
    pub fn find_critical_parameters(&self, delta: f64) -> Result<Vec<(usize, usize, f64)>, GameError> {
        let results = self.full_analysis(delta)?;

        let mut critical: Vec<(usize, usize, f64)> = results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let row = idx / 3;
                let col = idx % 3;
                let total_change: f64 = result
                    .kicker_strategy_change
                    .iter()
                    .chain(result.goalkeeper_strategy_change.iter())
                    .map(|x| x.abs())
                    .sum();
                (row, col, total_change)
            })
            .collect();

        critical.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        Ok(critical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitivity_analysis() {
        let analyzer = SensitivityAnalyzer::with_default_data();
        let result = analyzer.analyze_single_change(0, 0, 0.1).unwrap();

        assert_eq!(result.original_value, 0.58);
        assert!((result.new_value - 0.68).abs() < 0.001);
    }

    #[test]
    fn test_full_analysis() {
        let analyzer = SensitivityAnalyzer::with_default_data();
        let results = analyzer.full_analysis(0.05).unwrap();

        assert_eq!(results.len(), 9); // 3x3 matrix
    }
}
