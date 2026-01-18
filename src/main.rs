use football_game_theory::football::penalty::PenaltyKick;
use football_game_theory::analysis::simulation::Simulator;
use football_game_theory::analysis::sensitivity::SensitivityAnalyzer;
use football_game_theory::visualization::ascii::GoalVisualizer;
use football_game_theory::visualization::heatmap::HeatmapRenderer;
use football_game_theory::visualization::chart::BarChart;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║       FOOTBALL GAME THEORY: PK ANALYSIS                    ║");
    println!("║       Nash Equilibrium Strategy Finder                     ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Analyze PK with default real-world data
    let pk = PenaltyKick::with_default_data();

    // Visualize payoff matrix as heatmap
    let heatmap = HeatmapRenderer::new();
    let matrix = pk.payoff_matrix().matrix();
    let rows = vec!["Kick Left", "Kick Center", "Kick Right"];
    let cols = vec!["GK Left", "GK Center", "GK Right"];

    println!("{}", heatmap.render(matrix, &rows, &cols, "PAYOFF MATRIX (Goal Success Rates)"));

    match pk.analyze() {
        Ok(analysis) => {
            // Extract strategy values
            let kicker_strat: Vec<f64> = analysis.kicker_strategy.iter().map(|(_, p)| *p).collect();
            let gk_strat: Vec<f64> = analysis.goalkeeper_strategy.iter().map(|(_, p)| *p).collect();

            // Visualize strategies with goal diagram
            let goal_viz = GoalVisualizer::new();
            println!("{}", goal_viz.render_kicker_strategy(
                kicker_strat[0], kicker_strat[1], kicker_strat[2]
            ));
            println!("{}", goal_viz.render_goalkeeper_strategy(
                gk_strat[0], gk_strat[1], gk_strat[2]
            ));

            // Bar chart for strategies
            let chart = BarChart::new();
            let kicker_data: Vec<(&str, f64)> = vec![
                ("Left", kicker_strat[0]),
                ("Center", kicker_strat[1]),
                ("Right", kicker_strat[2]),
            ];
            println!("{}", chart.render("KICKER OPTIMAL STRATEGY", &kicker_data, 1.0));

            let gk_data: Vec<(&str, f64)> = vec![
                ("Left", gk_strat[0]),
                ("Center", gk_strat[1]),
                ("Right", gk_strat[2]),
            ];
            println!("{}", chart.render("GOALKEEPER OPTIMAL STRATEGY", &gk_data, 1.0));

            // Summary statistics
            println!("╔════════════════════════════════════════════════════════════╗");
            println!("║                    NASH EQUILIBRIUM                        ║");
            println!("╠════════════════════════════════════════════════════════════╣");
            println!("║  Kicker:     {}",
                format!("{:<49}║", analysis.kicker_strategy_string()));
            println!("║  Goalkeeper: {}",
                format!("{:<49}║", analysis.goalkeeper_strategy_string()));
            println!("║  Game Value: {}",
                format!("{:<49}║", format!("{:.1}% expected goal rate", analysis.goal_probability * 100.0)));
            println!("╚════════════════════════════════════════════════════════════╝\n");

            // Run simulation
            println!("═══════════════════════════════════════════════════════════════");
            println!("                    MONTE CARLO SIMULATION                      ");
            println!("═══════════════════════════════════════════════════════════════\n");

            let sim = Simulator::new().seed(42);
            let result = sim.simulate(&kicker_strat, &gk_strat, 10000);

            println!("  Simulated {} penalty kicks with optimal strategies", result.total_kicks);
            println!("  Results: {} goals ({:.1}%)\n",
                result.goals_scored,
                result.goal_percentage()
            );

            // Compare with uniform strategy
            let uniform = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
            let uniform_result = sim.simulate(&uniform, &uniform, 10000);

            // Comparison chart
            println!("{}", chart.render_comparison(
                "STRATEGY COMPARISON: Optimal vs Uniform",
                &["Left", "Center", "Right"],
                ("Optimal", &kicker_strat),
                ("Uniform", &uniform),
            ));

            println!("  Optimal strategy result: {:.1}% goals", result.goal_percentage());
            println!("  Uniform strategy result: {:.1}% goals", uniform_result.goal_percentage());
            println!("  Difference: {:+.1}%\n",
                result.goal_percentage() - uniform_result.goal_percentage());
        }
        Err(e) => {
            eprintln!("Analysis failed: {}", e);
        }
    }

    // Sensitivity analysis
    println!("═══════════════════════════════════════════════════════════════");
    println!("                    SENSITIVITY ANALYSIS                        ");
    println!("═══════════════════════════════════════════════════════════════\n");

    let analyzer = SensitivityAnalyzer::with_default_data();
    match analyzer.find_critical_parameters(0.05) {
        Ok(critical) => {
            println!("  Most sensitive parameters (when changed by +5%):\n");

            let chart = BarChart::new();
            let sensitivity_data: Vec<(&str, f64)> = critical
                .iter()
                .take(5)
                .map(|(row, col, sens)| {
                    let label = match (*row, *col) {
                        (0, 0) => "L vs L",
                        (0, 1) => "L vs C",
                        (0, 2) => "L vs R",
                        (1, 0) => "C vs L",
                        (1, 1) => "C vs C",
                        (1, 2) => "C vs R",
                        (2, 0) => "R vs L",
                        (2, 1) => "R vs C",
                        (2, 2) => "R vs R",
                        _ => "?",
                    };
                    (label, *sens)
                })
                .collect();

            let max_sens = sensitivity_data.iter().map(|(_, v)| *v).fold(0.0, f64::max);
            println!("{}", chart.render("PARAMETER SENSITIVITY (Kick vs GK)", &sensitivity_data, max_sens * 1.2));

            println!("  Key insight: Changes to diagonal elements (same direction)");
            println!("  have the highest impact on optimal strategies.\n");
        }
        Err(e) => {
            eprintln!("  Sensitivity analysis failed: {}", e);
        }
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("                         COMPLETE                               ");
    println!("═══════════════════════════════════════════════════════════════");
}
