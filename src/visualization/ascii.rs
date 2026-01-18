/// ASCII art visualization for penalty kick strategies.

/// Renders a football goal with strategy distribution overlay.
pub struct GoalVisualizer {
    width: usize,
}

impl GoalVisualizer {
    pub fn new() -> Self {
        Self {
            width: 60,
        }
    }

    /// Renders the goal with kicker's strategy probabilities.
    pub fn render_kicker_strategy(&self, left: f64, center: f64, right: f64) -> String {
        self.render_strategy("KICKER STRATEGY", left, center, right)
    }

    /// Renders the goal with goalkeeper's strategy probabilities.
    pub fn render_goalkeeper_strategy(&self, left: f64, center: f64, right: f64) -> String {
        self.render_strategy("GOALKEEPER STRATEGY", left, center, right)
    }

    fn render_strategy(&self, title: &str, left: f64, center: f64, right: f64) -> String {
        let left_pct = format!("{:.1}%", left * 100.0);
        let center_pct = format!("{:.1}%", center * 100.0);
        let right_pct = format!("{:.1}%", right * 100.0);

        let left_bar = self.probability_bar(left);
        let center_bar = self.probability_bar(center);
        let right_bar = self.probability_bar(right);

        let section_width = (self.width - 4) / 3;

        format!(
            r#"
    {title:^width$}
    ╔{bar}╦{bar}╦{bar}╗
    ║{left_sec:^sw$}║{center_sec:^sw$}║{right_sec:^sw$}║
    ║{lb:^sw$}║{cb:^sw$}║{rb:^sw$}║
    ║{lp:^sw$}║{cp:^sw$}║{rp:^sw$}║
    ║{:^sw$}║{:^sw$}║{:^sw$}║
    ╠{bar}╩{bar}╩{bar}╣
    ║{:^fw$}║
    ╚{fbar}╝
"#,
            "", "", "",
            "⚽ GOAL ⚽",
            title = title,
            width = self.width,
            bar = "═".repeat(section_width),
            fbar = "═".repeat(self.width - 2),
            fw = self.width - 2,
            sw = section_width,
            left_sec = "LEFT",
            center_sec = "CENTER",
            right_sec = "RIGHT",
            lb = left_bar,
            cb = center_bar,
            rb = right_bar,
            lp = left_pct,
            cp = center_pct,
            rp = right_pct,
        )
    }

    /// Creates a visual bar representing probability.
    fn probability_bar(&self, prob: f64) -> String {
        let max_blocks = 10;
        let filled = (prob * max_blocks as f64).round() as usize;
        let empty = max_blocks - filled;
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }
}

impl Default for GoalVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders a simple pitch diagram showing the PK scenario.
pub fn render_pitch() -> String {
    r#"
                    ┌─────────────────────────────┐
                    │           GOAL              │
                    └─────────────────────────────┘
                           ┌───────────┐
                           │  PENALTY  │
                           │    BOX    │
                           └───────────┘

                                ⚽

                              KICKER
    "#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_visualizer() {
        let viz = GoalVisualizer::new();
        let output = viz.render_kicker_strategy(0.34, 0.28, 0.38);
        assert!(output.contains("LEFT"));
        assert!(output.contains("CENTER"));
        assert!(output.contains("RIGHT"));
        assert!(output.contains("34.0%"));
    }

    #[test]
    fn test_probability_bar() {
        let viz = GoalVisualizer::new();
        let bar = viz.probability_bar(0.5);
        assert!(bar.contains("█████"));
    }
}
