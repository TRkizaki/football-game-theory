/// ASCII heatmap visualization for payoff matrices.

/// Color/shade levels for heatmap cells.
const HEAT_LEVELS: [&str; 10] = [
    "░░░", "░░▒", "░▒▒", "▒▒▒", "▒▒▓",
    "▒▓▓", "▓▓▓", "▓▓█", "▓██", "███"
];

/// Renders a payoff matrix as an ASCII heatmap.
pub struct HeatmapRenderer {
    cell_width: usize,
}

impl HeatmapRenderer {
    pub fn new() -> Self {
        Self { cell_width: 12 }
    }

    /// Renders the payoff matrix as a heatmap with color gradient.
    pub fn render(
        &self,
        matrix: &[Vec<f64>],
        row_labels: &[&str],
        col_labels: &[&str],
        title: &str,
    ) -> String {
        let mut output = String::new();

        // Find min/max for normalization
        let (min_val, max_val) = self.find_range(matrix);

        // Title
        let total_width = self.cell_width * (col_labels.len() + 1) + col_labels.len() + 2;
        output.push_str(&format!("\n{:^width$}\n", title, width = total_width));
        output.push_str(&format!("{:^width$}\n\n", self.render_legend(min_val, max_val), width = total_width));

        // Header row
        output.push_str(&format!("{:>width$}", "", width = self.cell_width));
        for label in col_labels {
            output.push_str(&format!(" {:^width$}", label, width = self.cell_width));
        }
        output.push('\n');

        // Separator
        output.push_str(&format!("{:>width$}", "", width = self.cell_width));
        for _ in col_labels {
            output.push_str(&format!(" {:─^width$}", "", width = self.cell_width));
        }
        output.push('\n');

        // Data rows
        for (i, row) in matrix.iter().enumerate() {
            let row_label = row_labels.get(i).unwrap_or(&"");
            output.push_str(&format!("{:>width$}", row_label, width = self.cell_width));

            for &val in row {
                let heat = self.value_to_heat(val, min_val, max_val);
                let cell = format!("{} {:.2}", heat, val);
                output.push_str(&format!(" {:^width$}", cell, width = self.cell_width));
            }
            output.push('\n');
        }

        output
    }

    /// Renders a compact heatmap for quick display.
    pub fn render_compact(
        &self,
        matrix: &[Vec<f64>],
        row_labels: &[&str],
        col_labels: &[&str],
    ) -> String {
        let mut output = String::new();

        let (min_val, max_val) = self.find_range(matrix);

        // Header
        output.push_str("         ");
        for label in col_labels {
            output.push_str(&format!(" {:^7}", label));
        }
        output.push('\n');

        // Rows
        for (i, row) in matrix.iter().enumerate() {
            let row_label = row_labels.get(i).unwrap_or(&"");
            output.push_str(&format!("{:>8} ", row_label));

            for &val in row {
                let heat = self.value_to_heat(val, min_val, max_val);
                output.push_str(&format!(" {} ", heat));
            }
            output.push_str(&format!("  <- {}\n", row_label));
        }

        output
    }

    fn find_range(&self, matrix: &[Vec<f64>]) -> (f64, f64) {
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;

        for row in matrix {
            for &val in row {
                if val < min_val {
                    min_val = val;
                }
                if val > max_val {
                    max_val = val;
                }
            }
        }

        (min_val, max_val)
    }

    fn value_to_heat(&self, val: f64, min_val: f64, max_val: f64) -> &'static str {
        if (max_val - min_val).abs() < 1e-10 {
            return HEAT_LEVELS[5];
        }

        let normalized = (val - min_val) / (max_val - min_val);
        let index = (normalized * 9.0).round() as usize;
        HEAT_LEVELS[index.min(9)]
    }

    fn render_legend(&self, min_val: f64, max_val: f64) -> String {
        format!(
            "Low ({:.2}) {} {} {} {} {} High ({:.2})",
            min_val,
            HEAT_LEVELS[0],
            HEAT_LEVELS[2],
            HEAT_LEVELS[5],
            HEAT_LEVELS[7],
            HEAT_LEVELS[9],
            max_val
        )
    }
}

impl Default for HeatmapRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heatmap_render() {
        let renderer = HeatmapRenderer::new();
        let matrix = vec![
            vec![0.58, 0.93, 0.95],
            vec![0.83, 0.44, 0.83],
            vec![0.93, 0.90, 0.60],
        ];
        let rows = vec!["Kick L", "Kick C", "Kick R"];
        let cols = vec!["GK Left", "GK Center", "GK Right"];

        let output = renderer.render(&matrix, &rows, &cols, "PK Success Rates");
        assert!(output.contains("PK Success Rates"));
        assert!(output.contains("Kick L"));
    }

    #[test]
    fn test_heat_levels() {
        let renderer = HeatmapRenderer::new();
        let low = renderer.value_to_heat(0.0, 0.0, 1.0);
        let high = renderer.value_to_heat(1.0, 0.0, 1.0);
        assert_eq!(low, HEAT_LEVELS[0]);
        assert_eq!(high, HEAT_LEVELS[9]);
    }
}
