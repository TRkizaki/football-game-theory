//! Example: Penalty Kick Analysis using Game Theory
//!
//! This example demonstrates how to:
//! 1. Create a PK game model with custom success rates
//! 2. Find the Nash equilibrium (optimal mixed strategies)
//! 3. Run simulations to compare strategies
//! 4. Perform sensitivity analysis

use football_game_theory::football::penalty::PenaltyKick;
use football_game_theory::analysis::simulation::Simulator;

fn main() {
    println!("=== Custom PK Analysis Example ===\n");

    // Example 1: Using default real-world data
    println!("--- Using Default Data (Palacios-Huerta 2003) ---");
    analyze_default_data();

    // Example 2: Custom player-specific data
    println!("\n--- Custom Player Analysis ---");
    analyze_custom_player();

    // Example 3: What if GK always stays in center?
    println!("\n--- Exploiting Predictable GK ---");
    analyze_predictable_gk();
}

fn analyze_default_data() {
    let pk = PenaltyKick::with_default_data();

    println!("Success rate matrix:");
    println!("{}", pk.payoff_matrix().display());

    let analysis = pk.analyze().expect("Analysis should succeed");

    println!("Nash Equilibrium:");
    println!("  Kicker strategy:     {}", analysis.kicker_strategy_string());
    println!("  Goalkeeper strategy: {}", analysis.goalkeeper_strategy_string());
    println!("  Goal probability:    {:.1}%", analysis.goal_probability * 100.0);
}

fn analyze_custom_player() {
    // Hypothetical player who is very strong kicking right
    let success_rates = vec![
        vec![0.50, 0.85, 0.90], // Kick Left: weaker than average
        vec![0.70, 0.35, 0.70], // Kick Center: normal
        vec![0.98, 0.95, 0.75], // Kick Right: very strong
    ];

    let pk = PenaltyKick::new(success_rates).expect("Valid matrix");
    let analysis = pk.analyze().expect("Analysis should succeed");

    println!("Right-footed specialist:");
    println!("  Optimal kick:   {}", analysis.kicker_strategy_string());
    println!("  GK should dive: {}", analysis.goalkeeper_strategy_string());
    println!("  Goal probability: {:.1}%", analysis.goal_probability * 100.0);
}

fn analyze_predictable_gk() {
    let pk = PenaltyKick::with_default_data();

    // GK always stays in center
    let gk_center = vec![0.0, 1.0, 0.0];

    // Find best response for kicker
    let matrix = pk.payoff_matrix().matrix();

    // Against GK staying center, kicker should maximize vs center column
    let best_kick_idx = (0..3)
        .max_by(|&i, &j| {
            matrix[i][1].partial_cmp(&matrix[j][1]).unwrap()
        })
        .unwrap();

    let directions = ["Left", "Center", "Right"];
    println!(
        "If GK always stays center, kicker should kick: {} ({:.0}% success)",
        directions[best_kick_idx],
        matrix[best_kick_idx][1] * 100.0
    );

    // Simulate this scenario
    let sim = Simulator::new().seed(42);
    let kicker_best = match best_kick_idx {
        0 => vec![1.0, 0.0, 0.0],
        1 => vec![0.0, 1.0, 0.0],
        _ => vec![0.0, 0.0, 1.0],
    };

    let result = sim.simulate(&kicker_best, &gk_center, 1000);
    println!(
        "Simulation (1000 kicks): {:.1}% goals",
        result.goal_percentage()
    );
}
