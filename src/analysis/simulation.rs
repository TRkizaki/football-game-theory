use crate::football::penalty::{Direction, PenaltyKick};
use crate::solver::game::GameError;

/// Result of a single simulated penalty kick.
#[derive(Debug, Clone, Copy)]
pub struct SimulatedKick {
    pub kick_direction: Direction,
    pub gk_direction: Direction,
    pub is_goal: bool,
}

/// Result of a simulation run.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub kicks: Vec<SimulatedKick>,
    pub goals_scored: u32,
    pub total_kicks: u32,
    pub kicker_strategy: Vec<f64>,
    pub goalkeeper_strategy: Vec<f64>,
}

impl SimulationResult {
    /// Returns the goal percentage.
    pub fn goal_percentage(&self) -> f64 {
        if self.total_kicks == 0 {
            0.0
        } else {
            (self.goals_scored as f64 / self.total_kicks as f64) * 100.0
        }
    }

    /// Returns statistics for each direction combination.
    pub fn direction_stats(&self) -> Vec<((Direction, Direction), u32, u32)> {
        let mut stats = vec![];

        for kick_dir in Direction::all() {
            for gk_dir in Direction::all() {
                let (goals, attempts) = self.kicks.iter().fold((0, 0), |(g, a), kick| {
                    if kick.kick_direction == *kick_dir && kick.gk_direction == *gk_dir {
                        (g + if kick.is_goal { 1 } else { 0 }, a + 1)
                    } else {
                        (g, a)
                    }
                });
                if attempts > 0 {
                    stats.push(((*kick_dir, *gk_dir), goals, attempts));
                }
            }
        }

        stats
    }
}

/// Simulates penalty kick scenarios.
pub struct Simulator {
    pk: PenaltyKick,
    rng_seed: u64,
}

impl Simulator {
    /// Creates a new simulator with default PK data.
    pub fn new() -> Self {
        Self {
            pk: PenaltyKick::with_default_data(),
            rng_seed: 12345,
        }
    }

    /// Creates a simulator with custom success rates.
    pub fn with_matrix(success_rates: Vec<Vec<f64>>) -> Result<Self, crate::football::payoff::PayoffError> {
        Ok(Self {
            pk: PenaltyKick::new(success_rates)?,
            rng_seed: 12345,
        })
    }

    /// Sets the random seed for reproducibility.
    pub fn seed(mut self, seed: u64) -> Self {
        self.rng_seed = seed;
        self
    }

    /// Simulates kicks with given strategies.
    ///
    /// # Arguments
    /// * `kicker_strategy` - Probability distribution over kick directions
    /// * `gk_strategy` - Probability distribution over GK directions
    /// * `num_kicks` - Number of kicks to simulate
    pub fn simulate(
        &self,
        kicker_strategy: &[f64],
        gk_strategy: &[f64],
        num_kicks: u32,
    ) -> SimulationResult {
        let mut rng = SimpleRng::new(self.rng_seed);
        let mut kicks = Vec::with_capacity(num_kicks as usize);
        let mut goals_scored = 0;

        let matrix = self.pk.payoff_matrix().matrix();

        for _ in 0..num_kicks {
            // Sample kick direction
            let kick_dir = sample_direction(&mut rng, kicker_strategy);
            // Sample GK direction
            let gk_dir = sample_direction(&mut rng, gk_strategy);

            // Determine if goal based on success rate
            let success_rate = matrix[kick_dir.index()][gk_dir.index()];
            let is_goal = rng.next_f64() < success_rate;

            if is_goal {
                goals_scored += 1;
            }

            kicks.push(SimulatedKick {
                kick_direction: kick_dir,
                gk_direction: gk_dir,
                is_goal,
            });
        }

        SimulationResult {
            kicks,
            goals_scored,
            total_kicks: num_kicks,
            kicker_strategy: kicker_strategy.to_vec(),
            goalkeeper_strategy: gk_strategy.to_vec(),
        }
    }

    /// Compares optimal strategy vs a given strategy.
    pub fn compare_strategies(
        &self,
        alternative_kicker: &[f64],
        alternative_gk: &[f64],
        num_kicks: u32,
    ) -> Result<(SimulationResult, SimulationResult), GameError> {
        let analysis = self.pk.analyze()?;

        // Extract optimal strategies
        let optimal_kicker: Vec<f64> = analysis
            .kicker_strategy
            .iter()
            .map(|(_, p)| *p)
            .collect();
        let optimal_gk: Vec<f64> = analysis
            .goalkeeper_strategy
            .iter()
            .map(|(_, p)| *p)
            .collect();

        // Simulate with optimal strategies
        let optimal_result = self.simulate(&optimal_kicker, &optimal_gk, num_kicks);

        // Simulate with alternative strategies
        let alternative_result = self.simulate(alternative_kicker, alternative_gk, num_kicks);

        Ok((optimal_result, alternative_result))
    }

    /// Returns the underlying PK model.
    pub fn penalty_kick(&self) -> &PenaltyKick {
        &self.pk
    }
}

impl Default for Simulator {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple linear congruential generator for reproducible randomness.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // LCG parameters from Numerical Recipes
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Samples a direction based on the given probability distribution.
fn sample_direction(rng: &mut SimpleRng, probs: &[f64]) -> Direction {
    let r = rng.next_f64();
    let mut cumulative = 0.0;

    for (i, &p) in probs.iter().enumerate() {
        cumulative += p;
        if r < cumulative {
            return Direction::from_index(i).unwrap_or(Direction::Center);
        }
    }

    // Fallback to last direction
    Direction::from_index(probs.len() - 1).unwrap_or(Direction::Right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_basic() {
        let sim = Simulator::new().seed(42);
        let uniform = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];

        let result = sim.simulate(&uniform, &uniform, 1000);

        assert_eq!(result.total_kicks, 1000);
        assert!(result.goals_scored > 0);
        assert!(result.goals_scored < 1000);
    }

    #[test]
    fn test_simulation_reproducibility() {
        let sim1 = Simulator::new().seed(12345);
        let sim2 = Simulator::new().seed(12345);

        let uniform = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];

        let result1 = sim1.simulate(&uniform, &uniform, 100);
        let result2 = sim2.simulate(&uniform, &uniform, 100);

        assert_eq!(result1.goals_scored, result2.goals_scored);
    }

    #[test]
    fn test_strategy_comparison() {
        let sim = Simulator::new().seed(42);
        let uniform = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];

        let (optimal, alternative) = sim.compare_strategies(&uniform, &uniform, 1000).unwrap();

        // Both should complete successfully
        assert_eq!(optimal.total_kicks, 1000);
        assert_eq!(alternative.total_kicks, 1000);
    }
}
