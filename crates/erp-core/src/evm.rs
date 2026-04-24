use serde::{Deserialize, Serialize};

/// A snapshot of earned value data for a single reporting period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmSnapshot {
    /// Period label (e.g., "2026-W14", "2026-03")
    pub period: String,
    /// Budgeted Cost of Work Scheduled (Planned Value)
    pub bcws: f64,
    /// Budgeted Cost of Work Performed (Earned Value)
    pub bcwp: f64,
    /// Actual Cost of Work Performed
    pub acwp: f64,
}

/// Computed Earned Value Management metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmMetrics {
    /// Schedule Performance Index (BCWP / BCWS)
    pub spi: f64,
    /// Cost Performance Index (BCWP / ACWP)
    pub cpi: f64,
    /// Schedule Variance (BCWP - BCWS)
    pub sv: f64,
    /// Cost Variance (BCWP - ACWP)
    pub cv: f64,
    /// Estimate at Completion (BAC / CPI)
    pub eac: f64,
    /// Estimate to Complete (EAC - ACWP)
    pub etc: f64,
    /// Variance at Completion (BAC - EAC)
    pub vac: f64,
    /// To-Complete Performance Index ((BAC - BCWP) / (BAC - ACWP))
    pub tcpi: f64,
}

/// Calculate Earned Value Management metrics from BAC and a period snapshot.
///
/// - `bac`: Budget at Completion (total project budget)
/// - `snapshot`: Current period's BCWS, BCWP, ACWP values
///
/// Returns computed EVM metrics. Division by zero is guarded with f64::INFINITY.
pub fn calculate_evm(bac: f64, snapshot: &EvmSnapshot) -> EvmMetrics {
    let spi = if snapshot.bcws != 0.0 {
        snapshot.bcwp / snapshot.bcws
    } else {
        f64::INFINITY
    };

    let cpi = if snapshot.acwp != 0.0 {
        snapshot.bcwp / snapshot.acwp
    } else {
        f64::INFINITY
    };

    let sv = snapshot.bcwp - snapshot.bcws;
    let cv = snapshot.bcwp - snapshot.acwp;

    let eac = if cpi != 0.0 && cpi.is_finite() {
        bac / cpi
    } else {
        f64::INFINITY
    };

    let etc = if eac.is_finite() {
        eac - snapshot.acwp
    } else {
        f64::INFINITY
    };

    let vac = if eac.is_finite() {
        bac - eac
    } else {
        f64::NEG_INFINITY
    };

    let tcpi_denom = bac - snapshot.acwp;
    let tcpi = if tcpi_denom != 0.0 {
        (bac - snapshot.bcwp) / tcpi_denom
    } else {
        f64::INFINITY
    };

    EvmMetrics {
        spi,
        cpi,
        sv,
        cv,
        eac,
        etc,
        vac,
        tcpi,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_schedule_on_budget() {
        let snapshot = EvmSnapshot {
            period: "2026-W10".into(),
            bcws: 100_000.0,
            bcwp: 100_000.0,
            acwp: 100_000.0,
        };

        let metrics = calculate_evm(500_000.0, &snapshot);
        assert!((metrics.spi - 1.0).abs() < 0.001);
        assert!((metrics.cpi - 1.0).abs() < 0.001);
        assert!((metrics.sv - 0.0).abs() < 0.001);
        assert!((metrics.cv - 0.0).abs() < 0.001);
        assert!((metrics.eac - 500_000.0).abs() < 0.01);
    }

    #[test]
    fn test_behind_schedule_over_budget() {
        let snapshot = EvmSnapshot {
            period: "2026-W10".into(),
            bcws: 100_000.0,
            bcwp: 80_000.0,  // behind schedule
            acwp: 120_000.0, // over budget
        };

        let metrics = calculate_evm(500_000.0, &snapshot);
        assert!(metrics.spi < 1.0); // behind schedule
        assert!(metrics.cpi < 1.0); // over budget
        assert!(metrics.sv < 0.0);  // negative schedule variance
        assert!(metrics.cv < 0.0);  // negative cost variance
        assert!(metrics.eac > 500_000.0); // will cost more than planned
        assert!(metrics.vac < 0.0); // variance at completion is negative
    }

    #[test]
    fn test_ahead_of_schedule_under_budget() {
        let snapshot = EvmSnapshot {
            period: "2026-W10".into(),
            bcws: 100_000.0,
            bcwp: 120_000.0, // ahead of schedule
            acwp: 90_000.0,  // under budget
        };

        let metrics = calculate_evm(500_000.0, &snapshot);
        assert!(metrics.spi > 1.0);
        assert!(metrics.cpi > 1.0);
        assert!(metrics.sv > 0.0);
        assert!(metrics.cv > 0.0);
        assert!(metrics.eac < 500_000.0);
    }

    #[test]
    fn test_zero_bcws() {
        let snapshot = EvmSnapshot {
            period: "2026-W01".into(),
            bcws: 0.0,
            bcwp: 0.0,
            acwp: 0.0,
        };

        let metrics = calculate_evm(500_000.0, &snapshot);
        assert!(metrics.spi.is_infinite());
        assert!(metrics.cpi.is_infinite());
    }
}
