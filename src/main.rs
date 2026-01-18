use football_game_theory::football::penalty::PenaltyKick;
use football_game_theory::analysis::simulation::Simulator;
use football_game_theory::analysis::sensitivity::SensitivityAnalyzer;

fn main() {
    println!("=== Football Game Theory: PK Analysis ===\n");

    // Analyze PK with default real-world data
    let pk = PenaltyKick::with_default_data();

    println!("Payoff Matrix (Goal Success Rates):");
    println!("{}", pk.payoff_matrix().display());

    match pk.analyze() {
        Ok(analysis) => {
            println!("Optimal Strategies (Nash Equilibrium):");
            println!("  Kicker:     {}", analysis.kicker_strategy_string());
            println!("  Goalkeeper: {}", analysis.goalkeeper_strategy_string());
            println!(
                "  Expected Goal Probability: {:.1}%\n",
                analysis.goal_probability * 100.0
            );

            // Run simulation
            println!("=== Simulation: 10000 kicks ===");
            let sim = Simulator::new().seed(42);

            let kicker_strat: Vec<f64> = analysis.kicker_strategy.iter().map(|(_, p)| *p).collect();
            let gk_strat: Vec<f64> = analysis.goalkeeper_strategy.iter().map(|(_, p)| *p).collect();

            let result = sim.simulate(&kicker_strat, &gk_strat, 10000);
            println!(
                "Results: {} goals / {} kicks ({:.1}%)\n",
                result.goals_scored,
                result.total_kicks,
                result.goal_percentage()
            );

            // Compare with naive uniform strategy
            println!("=== Strategy Comparison ===");
            let uniform = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
            let uniform_result = sim.simulate(&uniform, &uniform, 10000);
            println!(
                "Uniform strategy:  {:.1}% goals",
                uniform_result.goal_percentage()
            );
            println!(
                "Optimal strategy:  {:.1}% goals",
                result.goal_percentage()
            );
        }
        Err(e) => {
            eprintln!("Analysis failed: {}", e);
        }
    }

    // Sensitivity analysis
    println!("\n=== Sensitivity Analysis ===");
    let analyzer = SensitivityAnalyzer::with_default_data();
    match analyzer.find_critical_parameters(0.05) {
        Ok(critical) => {
            println!("Most sensitive parameters (delta=0.05):");
            for (i, (row, col, sensitivity)) in critical.iter().take(3).enumerate() {
                let kick_dir = ["Left", "Center", "Right"][*row];
                let gk_dir = ["Left", "Center", "Right"][*col];
                println!(
                    "  {}. Kick {} vs GK {}: sensitivity = {:.4}",
                    i + 1,
                    kick_dir,
                    gk_dir,
                    sensitivity
                );
            }
        }
        Err(e) => {
            eprintln!("Sensitivity analysis failed: {}", e);
        }
    }
}
