//! Report export — JSON and CSV serialization for `EvalReport`.
//!
//! # Example
//!
//! ```rust,no_run
//! # fn example() -> traitclaw_core::Result<()> {
//! use traitclaw_eval::{EvalReport, TestResult};
//! use traitclaw_eval::export::EvalReportExport;
//!
//! let report = EvalReport {
//!     suite_name: "my_suite".into(),
//!     results: vec![
//!         TestResult {
//!             case_id: "c1".into(),
//!             actual_output: "hello".into(),
//!             scores: [("kw".to_string(), 1.0)].into_iter().collect(),
//!             passed: true,
//!         }
//!     ],
//!     average_score: 1.0,
//!     passed: 1,
//!     total: 1,
//! };
//!
//! report.export_json("/tmp/report.json")?;
//! report.export_csv("/tmp/report.csv")?;
//! # Ok(())
//! # }
//! ```

use std::io::Write;
use std::path::Path;

use traitclaw_core::{Error, Result};

use crate::EvalReport;

/// Extension trait adding export methods to `EvalReport`.
pub trait EvalReportExport {
    /// Write the report as a JSON file.
    ///
    /// The JSON is pretty-printed for readability.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written.
    fn export_json(&self, path: impl AsRef<Path>) -> Result<()>;

    /// Write the report as a CSV file.
    ///
    /// Columns: `case_id,metric,score,passed`
    ///
    /// One row per (case × metric). If a case has no metrics, one row with empty metric.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written.
    fn export_csv(&self, path: impl AsRef<Path>) -> Result<()>;
}

impl EvalReportExport for EvalReport {
    fn export_json(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Runtime(format!("JSON serialization error: {e}")))?;

        let mut file = std::fs::File::create(path)
            .map_err(|e| Error::Runtime(format!("Cannot create JSON file: {e}")))?;

        file.write_all(json.as_bytes())
            .map_err(|e| Error::Runtime(format!("Cannot write JSON file: {e}")))?;

        Ok(())
    }

    fn export_csv(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut file = std::fs::File::create(path)
            .map_err(|e| Error::Runtime(format!("Cannot create CSV file: {e}")))?;

        // Header
        writeln!(file, "case_id,metric,score,passed")
            .map_err(|e| Error::Runtime(format!("Cannot write CSV header: {e}")))?;

        for result in &self.results {
            if result.scores.is_empty() {
                writeln!(
                    file,
                    "{},{},{},{}",
                    escape_csv(&result.case_id),
                    "",
                    "",
                    result.passed
                )
                .map_err(|e| Error::Runtime(format!("Cannot write CSV row: {e}")))?;
            } else {
                // Sort metric names for deterministic output
                let mut metrics: Vec<_> = result.scores.iter().collect();
                metrics.sort_by_key(|(k, _)| k.as_str());

                for (metric, score) in &metrics {
                    writeln!(
                        file,
                        "{},{},{:.4},{}",
                        escape_csv(&result.case_id),
                        escape_csv(metric),
                        score,
                        result.passed
                    )
                    .map_err(|e| Error::Runtime(format!("Cannot write CSV row: {e}")))?;
                }
            }
        }

        Ok(())
    }
}

/// Escape a field for CSV: wrap in quotes if it contains comma, quote, or newline.
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EvalReport, TestResult};

    fn make_report() -> EvalReport {
        EvalReport {
            suite_name: "test_suite".into(),
            results: vec![
                TestResult {
                    case_id: "case_1".into(),
                    actual_output: "Hello, this is a response.".into(),
                    scores: [("keyword".to_string(), 0.85), ("length".to_string(), 1.0)]
                        .into_iter()
                        .collect(),
                    passed: true,
                },
                TestResult {
                    case_id: "case_2".into(),
                    actual_output: "Short.".into(),
                    scores: [("keyword".to_string(), 0.5)].into_iter().collect(),
                    passed: false,
                },
            ],
            average_score: 0.78,
            passed: 1,
            total: 2,
        }
    }

    #[test]
    fn test_export_json_parseable() {
        // AC #4: export_json → valid JSON parseable back to EvalReport
        let report = make_report();
        let path = std::env::temp_dir().join("traitclaw_eval_test.json");

        report.export_json(&path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: EvalReport = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed.suite_name, "test_suite");
        assert_eq!(parsed.results.len(), 2);
        assert_eq!(parsed.passed, 1);
        assert_eq!(parsed.total, 2);
        assert!((parsed.average_score - 0.78).abs() < 1e-6);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_csv_has_header_and_rows() {
        // AC #5: export_csv → valid CSV with header + one row per case×metric
        let report = make_report();
        let path = std::env::temp_dir().join("traitclaw_eval_test.csv");

        report.export_csv(&path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Header
        assert_eq!(lines[0], "case_id,metric,score,passed");

        // 2 metrics for case_1 + 1 metric for case_2 = 3 rows
        assert_eq!(
            lines.len(),
            4,
            "header + 3 data rows expected, got:\n{content}"
        );

        // Check case_1 is present
        assert!(content.contains("case_1"), "should contain case_1");
        assert!(content.contains("case_2"), "should contain case_2");
        assert!(content.contains("keyword"), "should contain metric name");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_csv_empty_report() {
        let report = EvalReport {
            suite_name: "empty".into(),
            results: vec![],
            average_score: 0.0,
            passed: 0,
            total: 0,
        };
        let path = std::env::temp_dir().join("traitclaw_eval_empty.csv");
        report.export_csv(&path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1); // only header
        assert_eq!(lines[0], "case_id,metric,score,passed");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("plain"), "plain");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
    }
}
