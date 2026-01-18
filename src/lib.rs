pub mod solver;
pub mod football;
pub mod analysis;

pub use solver::simplex::Simplex;
pub use solver::game::GameSolver;
pub use football::penalty::PenaltyKick;
pub use football::payoff::PayoffMatrix;
