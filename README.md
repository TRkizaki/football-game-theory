# Football Game Theory

A Rust library for analyzing football (soccer) penalty kicks using game theory. This project applies linear programming and Nash equilibrium concepts to find optimal strategies for both kickers and goalkeepers.

## Overview

Penalty kicks in football represent a classic two-player zero-sum game:
- **Kicker**: Chooses to kick Left, Center, or Right
- **Goalkeeper**: Chooses to dive Left, Center, or Right
- **Payoff**: Goal success probability from the kicker's perspective

This library uses the Simplex method to solve the linear programming formulation and find the mixed strategy Nash equilibrium.

## Features

- **Simplex Solver**: Pure Rust implementation of the Simplex algorithm for linear programming
- **Game Theory Module**: Solves two-player zero-sum games to find optimal mixed strategies
- **PK Analysis**: Football-specific model with real-world success rate data
- **Sensitivity Analysis**: Analyze how changes in success rates affect optimal strategies
- **Simulation**: Monte Carlo simulation to compare different strategies
- **Visualization**: ASCII-based charts, heatmaps, and goal diagrams for strategy display

## Sample Output

```
                          KICKER STRATEGY
    ╔══════════════════╦══════════════════╦══════════════════╗
    ║       LEFT       ║      CENTER      ║      RIGHT       ║
    ║   [███░░░░░░░]   ║   [███░░░░░░░]   ║   [████░░░░░░]   ║
    ║      34.1%       ║      27.7%       ║      38.2%       ║
    ╠══════════════════╩══════════════════╩══════════════════╣
    ║                         ⚽ GOAL ⚽                       ║
    ╚════════════════════════════════════════════════════════╝

╔════════════════════════════════════════════════════════════╗
║                    NASH EQUILIBRIUM                        ║
╠════════════════════════════════════════════════════════════╣
║  Kicker:     Left: 34.1%, Center: 27.7%, Right: 38.2%      ║
║  Goalkeeper: Left: 44.5%, Center: 12.1%, Right: 43.5%      ║
║  Game Value: 78.3% expected goal rate                      ║
╚════════════════════════════════════════════════════════════╝
```

## Project Structure

```
football-game-theory/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── solver/
│   │   ├── simplex.rs       # Simplex method implementation
│   │   ├── game.rs          # Game theory solver (mixed strategies)
│   │   └── nash.rs          # Nash equilibrium detection
│   ├── football/
│   │   ├── penalty.rs       # PK model
│   │   ├── payoff.rs        # Payoff matrix construction
│   │   └── stats.rs         # CSV data loading
│   ├── analysis/
│   │   ├── sensitivity.rs   # Sensitivity analysis
│   │   └── simulation.rs    # Monte Carlo simulation
│   └── visualization/
│       ├── ascii.rs         # Goal diagram visualization
│       ├── heatmap.rs       # Payoff matrix heatmap
│       └── chart.rs         # Bar charts and comparisons
├── data/
│   └── pk_stats.csv         # Sample PK statistics
└── examples/
    └── pk_analysis.rs       # Usage example
```

## Installation

```bash
git clone https://github.com/TRkizaki/football-game-theory.git
cd football-game-theory
cargo build --release
```

## Usage

### Run the CLI

```bash
cargo run
```

### Run the Example

```bash
cargo run --example pk_analysis
```

### Use as a Library

```rust
use football_game_theory::football::penalty::PenaltyKick;

fn main() {
    // Use default real-world data (Palacios-Huerta 2003)
    let pk = PenaltyKick::with_default_data();

    // Find Nash equilibrium
    let analysis = pk.analyze().unwrap();

    println!("Optimal Kicker Strategy: {}", analysis.kicker_strategy_string());
    println!("Optimal GK Strategy: {}", analysis.goalkeeper_strategy_string());
    println!("Goal Probability: {:.1}%", analysis.goal_probability * 100.0);
}
```

### Custom Success Rates

```rust
use football_game_theory::football::penalty::PenaltyKick;

// Define custom success rate matrix
// Rows: Kick direction (Left, Center, Right)
// Columns: GK direction (Left, Center, Right)
let success_rates = vec![
    vec![0.50, 0.85, 0.90], // Kick Left
    vec![0.70, 0.35, 0.70], // Kick Center
    vec![0.98, 0.95, 0.75], // Kick Right
];

let pk = PenaltyKick::new(success_rates).unwrap();
let analysis = pk.analyze().unwrap();
```

## Default Payoff Matrix

Based on empirical data from Palacios-Huerta (2003):

|              | GK Left | GK Center | GK Right |
|--------------|---------|-----------|----------|
| Kick Left    | 0.58    | 0.93      | 0.95     |
| Kick Center  | 0.83    | 0.44      | 0.83     |
| Kick Right   | 0.93    | 0.90      | 0.60     |

Values represent the probability of scoring a goal.

## Theory

### Two-Player Zero-Sum Games

In a zero-sum game, one player's gain equals the other's loss. For penalty kicks:
- Kicker wants to **maximize** goal probability
- Goalkeeper wants to **minimize** goal probability

### Mixed Strategy Nash Equilibrium

The optimal strategy is typically a **mixed strategy** - a probability distribution over actions. At equilibrium:
- Neither player can improve by unilaterally changing their strategy
- The kicker randomizes to make the GK indifferent
- The GK randomizes to make the kicker indifferent

### Linear Programming Formulation

The game is solved by converting it to a linear program:

**For the kicker (maximizer):**
```
Maximize: v
Subject to: Σ p_i * a_ij >= v  for all j
           Σ p_i = 1
           p_i >= 0
```

Where `p_i` is the probability of kicking in direction `i`, and `a_ij` is the success rate.

## Testing

```bash
cargo test
```

## License

MIT

## References

- Palacios-Huerta, I. (2003). Professionals Play Minimax. *Review of Economic Studies*, 70(2), 395-415.
- von Neumann, J. (1928). Zur Theorie der Gesellschaftsspiele. *Mathematische Annalen*, 100(1), 295-320.
