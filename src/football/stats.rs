use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Raw PK statistics record from CSV.
#[derive(Debug, Clone, Deserialize)]
pub struct PkRecord {
    pub kick_direction: String,
    pub gk_direction: String,
    pub goals: u32,
    pub attempts: u32,
}

impl PkRecord {
    /// Calculates the success rate.
    pub fn success_rate(&self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            self.goals as f64 / self.attempts as f64
        }
    }
}

/// Loads PK statistics from a CSV file.
///
/// Expected CSV format:
/// kick_direction,gk_direction,goals,attempts
/// left,left,58,100
/// left,center,93,100
/// ...
pub fn load_pk_stats(path: impl AsRef<Path>) -> Result<Vec<PkRecord>, StatsError> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: PkRecord = result?;
        records.push(record);
    }

    Ok(records)
}

/// Converts PK records into a 3x3 success rate matrix.
///
/// Matrix layout:
/// - Rows: Kick direction (left=0, center=1, right=2)
/// - Columns: GK direction (left=0, center=1, right=2)
pub fn records_to_matrix(records: &[PkRecord]) -> Result<Vec<Vec<f64>>, StatsError> {
    let mut matrix = vec![vec![0.0; 3]; 3];
    let mut filled = vec![vec![false; 3]; 3];

    for record in records {
        let kick_idx = direction_to_index(&record.kick_direction)?;
        let gk_idx = direction_to_index(&record.gk_direction)?;

        matrix[kick_idx][gk_idx] = record.success_rate();
        filled[kick_idx][gk_idx] = true;
    }

    // Check all cells are filled
    for (i, row) in filled.iter().enumerate() {
        for (j, &is_filled) in row.iter().enumerate() {
            if !is_filled {
                return Err(StatsError::InvalidData(format!(
                    "Missing data for kick={}, gk={}",
                    index_to_direction(i),
                    index_to_direction(j)
                )));
            }
        }
    }

    Ok(matrix)
}

/// Converts direction string to matrix index.
fn direction_to_index(direction: &str) -> Result<usize, StatsError> {
    match direction.to_lowercase().as_str() {
        "left" | "l" => Ok(0),
        "center" | "centre" | "middle" | "c" | "m" => Ok(1),
        "right" | "r" => Ok(2),
        other => Err(StatsError::InvalidData(format!(
            "Unknown direction: {}",
            other
        ))),
    }
}

/// Converts matrix index to direction name.
fn index_to_direction(index: usize) -> &'static str {
    match index {
        0 => "left",
        1 => "center",
        2 => "right",
        _ => "unknown",
    }
}

/// Aggregates multiple records for the same direction combination.
pub fn aggregate_records(records: Vec<PkRecord>) -> Vec<PkRecord> {
    use std::collections::HashMap;

    let mut aggregated: HashMap<(String, String), (u32, u32)> = HashMap::new();

    for record in records {
        let key = (
            record.kick_direction.to_lowercase(),
            record.gk_direction.to_lowercase(),
        );
        let entry = aggregated.entry(key).or_insert((0, 0));
        entry.0 += record.goals;
        entry.1 += record.attempts;
    }

    aggregated
        .into_iter()
        .map(|((kick, gk), (goals, attempts))| PkRecord {
            kick_direction: kick,
            gk_direction: gk,
            goals,
            attempts,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_parsing() {
        assert_eq!(direction_to_index("left").unwrap(), 0);
        assert_eq!(direction_to_index("Left").unwrap(), 0);
        assert_eq!(direction_to_index("CENTER").unwrap(), 1);
        assert_eq!(direction_to_index("r").unwrap(), 2);
    }

    #[test]
    fn test_success_rate() {
        let record = PkRecord {
            kick_direction: "left".into(),
            gk_direction: "left".into(),
            goals: 58,
            attempts: 100,
        };

        assert!((record.success_rate() - 0.58).abs() < 0.001);
    }
}
