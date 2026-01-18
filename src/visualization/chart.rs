/// ASCII bar chart visualization for strategy comparison.

/// Horizontal bar chart renderer.
pub struct BarChart {
    max_bar_width: usize,
    label_width: usize,
}

impl BarChart {
    pub fn new() -> Self {
        Self {
            max_bar_width: 40,
            label_width: 15,
        }
    }

    /// Renders a horizontal bar chart comparing values.
    pub fn render(&self, title: &str, data: &[(&str, f64)], max_value: f64) -> String {
        let mut output = String::new();

        // Title
        output.push_str(&format!("\n{}\n", title));
        output.push_str(&format!("{}\n\n", "─".repeat(title.len())));

        for (label, value) in data {
            let bar_len = ((value / max_value) * self.max_bar_width as f64).round() as usize;
            let bar = "█".repeat(bar_len);
            let percentage = format!("{:.1}%", value * 100.0);

            output.push_str(&format!(
                "{:>width$} │{:<bar_width$}│ {}\n",
                label,
                bar,
                percentage,
                width = self.label_width,
                bar_width = self.max_bar_width
            ));
        }

        // Scale
        output.push_str(&format!(
            "{:>width$} └{}┘\n",
            "",
            "─".repeat(self.max_bar_width),
            width = self.label_width
        ));
        output.push_str(&format!(
            "{:>width$}  0%{:^center$}100%\n",
            "",
            "",
            width = self.label_width,
            center = self.max_bar_width - 6
        ));

        output
    }

    /// Renders a comparison chart with two series side by side.
    pub fn render_comparison(
        &self,
        title: &str,
        labels: &[&str],
        series1: (&str, &[f64]),
        series2: (&str, &[f64]),
    ) -> String {
        let mut output = String::new();

        // Title
        output.push_str(&format!("\n{}\n", title));
        output.push_str(&format!("{}\n", "─".repeat(title.len())));
        output.push_str(&format!("  {} = ████  {} = ░░░░\n\n", series1.0, series2.0));

        let bar_width = self.max_bar_width / 2;

        for (i, label) in labels.iter().enumerate() {
            let val1 = series1.1.get(i).copied().unwrap_or(0.0);
            let val2 = series2.1.get(i).copied().unwrap_or(0.0);

            let bar1_len = (val1 * bar_width as f64).round() as usize;
            let bar2_len = (val2 * bar_width as f64).round() as usize;

            let bar1 = "█".repeat(bar1_len);
            let bar2 = "░".repeat(bar2_len);

            output.push_str(&format!(
                "{:>width$} │{:<bw$}│{:<bw$}│ {:.1}% vs {:.1}%\n",
                label,
                bar1,
                bar2,
                val1 * 100.0,
                val2 * 100.0,
                width = self.label_width,
                bw = bar_width
            ));
        }

        output
    }

    /// Renders a stacked probability distribution chart.
    pub fn render_distribution(&self, title: &str, labels: &[&str], values: &[f64]) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n{}\n", title));
        output.push_str(&format!("{}\n\n", "─".repeat(title.len())));

        // Stacked bar
        let total: f64 = values.iter().sum();
        let chars = ['█', '▓', '░'];

        output.push_str("  [");
        for (i, &val) in values.iter().enumerate() {
            let width = ((val / total) * self.max_bar_width as f64).round() as usize;
            let c = chars[i % chars.len()];
            output.push_str(&c.to_string().repeat(width));
        }
        output.push_str("]\n\n");

        // Legend
        for (i, (label, &val)) in labels.iter().zip(values.iter()).enumerate() {
            let c = chars[i % chars.len()];
            output.push_str(&format!(
                "  {} {} = {:.1}%\n",
                c,
                label,
                (val / total) * 100.0
            ));
        }

        output
    }
}

impl Default for BarChart {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a sparkline-style mini chart.
pub fn sparkline(values: &[f64]) -> String {
    const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    if values.is_empty() {
        return String::new();
    }

    let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max_val - min_val;

    values
        .iter()
        .map(|&v| {
            if range < 1e-10 {
                BLOCKS[4]
            } else {
                let normalized = (v - min_val) / range;
                let index = (normalized * 7.0).round() as usize;
                BLOCKS[index.min(7)]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_chart() {
        let chart = BarChart::new();
        let data = vec![
            ("Left", 0.34),
            ("Center", 0.28),
            ("Right", 0.38),
        ];

        let output = chart.render("Kicker Strategy", &data, 1.0);
        assert!(output.contains("Left"));
        assert!(output.contains("34.0%"));
    }

    #[test]
    fn test_sparkline() {
        let values = vec![0.1, 0.5, 0.3, 0.9, 0.2];
        let spark = sparkline(&values);
        assert_eq!(spark.chars().count(), 5);
    }

    #[test]
    fn test_distribution() {
        let chart = BarChart::new();
        let labels = vec!["A", "B", "C"];
        let values = vec![0.3, 0.5, 0.2];

        let output = chart.render_distribution("Test", &labels, &values);
        assert!(output.contains("A"));
        assert!(output.contains("B"));
        assert!(output.contains("C"));
    }
}
