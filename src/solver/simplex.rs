use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimplexError {
    #[error("Problem is unbounded")]
    Unbounded,
    #[error("Problem is infeasible")]
    Infeasible,
    #[error("Invalid tableau dimensions")]
    InvalidDimensions,
    #[error("Maximum iterations exceeded")]
    MaxIterations,
}

/// Simplex method solver for linear programming problems.
///
/// Solves problems in standard form:
/// Maximize: c^T * x
/// Subject to: Ax <= b, x >= 0
#[derive(Debug, Clone)]
pub struct Simplex {
    tableau: Vec<Vec<f64>>,
    num_vars: usize,
    num_constraints: usize,
    max_iterations: usize,
}

impl Simplex {
    /// Creates a new Simplex solver.
    ///
    /// # Arguments
    /// * `c` - Objective function coefficients (to maximize)
    /// * `a` - Constraint matrix (each row is a constraint)
    /// * `b` - Right-hand side values (must be non-negative)
    pub fn new(c: &[f64], a: &[Vec<f64>], b: &[f64]) -> Result<Self, SimplexError> {
        let num_vars = c.len();
        let num_constraints = a.len();

        if b.len() != num_constraints {
            return Err(SimplexError::InvalidDimensions);
        }

        for row in a.iter() {
            if row.len() != num_vars {
                return Err(SimplexError::InvalidDimensions);
            }
        }

        // Build the initial tableau
        // Format: [slack vars | original vars | RHS]
        // Last row is the objective function (negated for maximization)
        let total_cols = num_vars + num_constraints + 1;
        let total_rows = num_constraints + 1;

        let mut tableau = vec![vec![0.0; total_cols]; total_rows];

        // Fill constraint rows
        for i in 0..num_constraints {
            // Original variables
            for j in 0..num_vars {
                tableau[i][j] = a[i][j];
            }
            // Slack variable (identity matrix)
            tableau[i][num_vars + i] = 1.0;
            // RHS
            tableau[i][total_cols - 1] = b[i];
        }

        // Fill objective row (negated for maximization)
        for j in 0..num_vars {
            tableau[num_constraints][j] = -c[j];
        }

        Ok(Self {
            tableau,
            num_vars,
            num_constraints,
            max_iterations: 1000,
        })
    }

    /// Sets the maximum number of iterations.
    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Solves the linear program using the Simplex method.
    ///
    /// Returns the optimal value and the solution vector.
    pub fn solve(&mut self) -> Result<(f64, Vec<f64>), SimplexError> {
        for _ in 0..self.max_iterations {
            // Find the pivot column (most negative in objective row)
            let pivot_col = self.find_pivot_column();

            if pivot_col.is_none() {
                // Optimal solution found
                return Ok(self.extract_solution());
            }

            let pivot_col = pivot_col.unwrap();

            // Find the pivot row (minimum ratio test)
            let pivot_row = self.find_pivot_row(pivot_col)?;

            // Perform pivot operation
            self.pivot(pivot_row, pivot_col);
        }

        Err(SimplexError::MaxIterations)
    }

    /// Finds the pivot column (entering variable).
    fn find_pivot_column(&self) -> Option<usize> {
        let obj_row = &self.tableau[self.num_constraints];
        let num_cols = obj_row.len() - 1; // Exclude RHS

        let mut min_val = 0.0;
        let mut min_col = None;

        for j in 0..num_cols {
            if obj_row[j] < min_val {
                min_val = obj_row[j];
                min_col = Some(j);
            }
        }

        min_col
    }

    /// Finds the pivot row (leaving variable) using minimum ratio test.
    fn find_pivot_row(&self, pivot_col: usize) -> Result<usize, SimplexError> {
        let rhs_col = self.tableau[0].len() - 1;
        let mut min_ratio = f64::INFINITY;
        let mut min_row = None;

        for i in 0..self.num_constraints {
            let coeff = self.tableau[i][pivot_col];
            if coeff > 1e-10 {
                let ratio = self.tableau[i][rhs_col] / coeff;
                if ratio >= 0.0 && ratio < min_ratio {
                    min_ratio = ratio;
                    min_row = Some(i);
                }
            }
        }

        min_row.ok_or(SimplexError::Unbounded)
    }

    /// Performs a pivot operation.
    fn pivot(&mut self, pivot_row: usize, pivot_col: usize) {
        let pivot_val = self.tableau[pivot_row][pivot_col];
        let num_cols = self.tableau[0].len();
        let num_rows = self.tableau.len();

        // Divide pivot row by pivot element
        for j in 0..num_cols {
            self.tableau[pivot_row][j] /= pivot_val;
        }

        // Eliminate pivot column in other rows
        for i in 0..num_rows {
            if i != pivot_row {
                let factor = self.tableau[i][pivot_col];
                for j in 0..num_cols {
                    self.tableau[i][j] -= factor * self.tableau[pivot_row][j];
                }
            }
        }
    }

    /// Extracts the solution from the final tableau.
    fn extract_solution(&self) -> (f64, Vec<f64>) {
        let rhs_col = self.tableau[0].len() - 1;
        let mut solution = vec![0.0; self.num_vars];

        // Find basic variables
        for j in 0..self.num_vars {
            let mut basic_row = None;
            let mut is_basic = true;

            for i in 0..=self.num_constraints {
                let val = self.tableau[i][j];
                if (val - 1.0).abs() < 1e-10 {
                    if basic_row.is_some() {
                        is_basic = false;
                        break;
                    }
                    basic_row = Some(i);
                } else if val.abs() > 1e-10 {
                    is_basic = false;
                    break;
                }
            }

            if is_basic {
                if let Some(row) = basic_row {
                    if row < self.num_constraints {
                        solution[j] = self.tableau[row][rhs_col];
                    }
                }
            }
        }

        // Optimal value is in the bottom-right corner
        let optimal_value = self.tableau[self.num_constraints][rhs_col];

        (optimal_value, solution)
    }

    /// Returns the current tableau (for debugging).
    pub fn tableau(&self) -> &Vec<Vec<f64>> {
        &self.tableau
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_simple_maximization() {
        // Maximize: 3x + 2y
        // Subject to: x + y <= 4, x <= 2, y <= 3
        let c = vec![3.0, 2.0];
        let a = vec![
            vec![1.0, 1.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];
        let b = vec![4.0, 2.0, 3.0];

        let mut solver = Simplex::new(&c, &a, &b).unwrap();
        let (optimal, solution) = solver.solve().unwrap();

        assert_relative_eq!(optimal, 10.0, epsilon = 1e-6);
        assert_relative_eq!(solution[0], 2.0, epsilon = 1e-6);
        assert_relative_eq!(solution[1], 2.0, epsilon = 1e-6);
    }

    #[test]
    fn test_another_lp() {
        // Maximize: 5x + 4y
        // Subject to: x + y <= 5, 10x + 6y <= 45
        let c = vec![5.0, 4.0];
        let a = vec![
            vec![1.0, 1.0],
            vec![10.0, 6.0],
        ];
        let b = vec![5.0, 45.0];

        let mut solver = Simplex::new(&c, &a, &b).unwrap();
        let (optimal, solution) = solver.solve().unwrap();

        // Optimal: x=3.75, y=1.25, value=23.75
        assert_relative_eq!(optimal, 23.75, epsilon = 1e-6);
        assert_relative_eq!(solution[0], 3.75, epsilon = 1e-6);
        assert_relative_eq!(solution[1], 1.25, epsilon = 1e-6);
    }
}
