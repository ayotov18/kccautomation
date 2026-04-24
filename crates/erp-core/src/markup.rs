use serde::{Deserialize, Serialize};

/// Type of markup calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkupType {
    Percentage,
    Fixed,
    PerUnit,
}

/// What base the markup applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyTo {
    /// Markup applies to the original direct cost only
    DirectCost,
    /// Markup applies to the cumulative running total
    Cumulative,
}

/// A single markup definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Markup {
    pub name: String,
    pub markup_type: MarkupType,
    pub percentage: f64,
    pub fixed_amount: f64,
    pub apply_to: ApplyTo,
    pub sort_order: i32,
    pub is_active: bool,
}

/// Result of applying a single markup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkupResult {
    pub name: String,
    pub amount: f64,
    pub running_total: f64,
}

/// Calculate all markups on a direct cost, returning individual results and grand total.
///
/// For each markup (in sort_order):
/// - If Percentage + DirectCost: amount = direct_cost * pct / 100
/// - If Percentage + Cumulative: amount = running_total * pct / 100
/// - If Fixed: amount = fixed_amount
/// - If PerUnit: amount = fixed_amount (treated as a fixed add-on per the BOQ)
///
/// Grand total = running_total after all markups.
pub fn calculate_markups(direct_cost: f64, markups: &[Markup]) -> (Vec<MarkupResult>, f64) {
    let mut active: Vec<&Markup> = markups.iter().filter(|m| m.is_active).collect();
    active.sort_by_key(|m| m.sort_order);

    let mut running_total = direct_cost;
    let mut results = Vec::new();

    for markup in active {
        let amount = match markup.markup_type {
            MarkupType::Percentage => match markup.apply_to {
                ApplyTo::DirectCost => direct_cost * markup.percentage / 100.0,
                ApplyTo::Cumulative => running_total * markup.percentage / 100.0,
            },
            MarkupType::Fixed => markup.fixed_amount,
            MarkupType::PerUnit => markup.fixed_amount,
        };

        running_total += amount;

        results.push(MarkupResult {
            name: markup.name.clone(),
            amount,
            running_total,
        });
    }

    let grand_total = running_total;
    (results, grand_total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage_on_direct_cost() {
        let markups = vec![Markup {
            name: "Overhead".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        }];

        let (results, total) = calculate_markups(1000.0, &markups);
        assert_eq!(results.len(), 1);
        assert!((results[0].amount - 100.0).abs() < 0.01);
        assert!((total - 1100.0).abs() < 0.01);
    }

    #[test]
    fn test_cumulative_markup() {
        let markups = vec![
            Markup {
                name: "Overhead".into(),
                markup_type: MarkupType::Percentage,
                percentage: 10.0,
                fixed_amount: 0.0,
                apply_to: ApplyTo::DirectCost,
                sort_order: 1,
                is_active: true,
            },
            Markup {
                name: "VAT".into(),
                markup_type: MarkupType::Percentage,
                percentage: 20.0,
                fixed_amount: 0.0,
                apply_to: ApplyTo::Cumulative,
                sort_order: 2,
                is_active: true,
            },
        ];

        let (results, total) = calculate_markups(1000.0, &markups);
        // Overhead: 1000 * 10% = 100, running = 1100
        // VAT: 1100 * 20% = 220, running = 1320
        assert!((results[0].amount - 100.0).abs() < 0.01);
        assert!((results[1].amount - 220.0).abs() < 0.01);
        assert!((total - 1320.0).abs() < 0.01);
    }

    #[test]
    fn test_fixed_markup() {
        let markups = vec![Markup {
            name: "Delivery".into(),
            markup_type: MarkupType::Fixed,
            percentage: 0.0,
            fixed_amount: 500.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        }];

        let (results, total) = calculate_markups(1000.0, &markups);
        assert!((results[0].amount - 500.0).abs() < 0.01);
        assert!((total - 1500.0).abs() < 0.01);
    }

    #[test]
    fn test_inactive_markups_skipped() {
        let markups = vec![
            Markup {
                name: "Active".into(),
                markup_type: MarkupType::Percentage,
                percentage: 10.0,
                fixed_amount: 0.0,
                apply_to: ApplyTo::DirectCost,
                sort_order: 1,
                is_active: true,
            },
            Markup {
                name: "Inactive".into(),
                markup_type: MarkupType::Percentage,
                percentage: 50.0,
                fixed_amount: 0.0,
                apply_to: ApplyTo::DirectCost,
                sort_order: 2,
                is_active: false,
            },
        ];

        let (results, total) = calculate_markups(1000.0, &markups);
        assert_eq!(results.len(), 1);
        assert!((total - 1100.0).abs() < 0.01);
    }

    #[test]
    fn test_sort_order_respected() {
        let markups = vec![
            Markup {
                name: "Second".into(),
                markup_type: MarkupType::Percentage,
                percentage: 20.0,
                fixed_amount: 0.0,
                apply_to: ApplyTo::Cumulative,
                sort_order: 2,
                is_active: true,
            },
            Markup {
                name: "First".into(),
                markup_type: MarkupType::Percentage,
                percentage: 10.0,
                fixed_amount: 0.0,
                apply_to: ApplyTo::DirectCost,
                sort_order: 1,
                is_active: true,
            },
        ];

        let (results, _total) = calculate_markups(1000.0, &markups);
        assert_eq!(results[0].name, "First");
        assert_eq!(results[1].name, "Second");
    }

    #[test]
    fn test_zero_direct_cost() {
        let markups = vec![Markup {
            name: "Overhead".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        }];

        let (_results, total) = calculate_markups(0.0, &markups);
        assert!((total - 0.0).abs() < 0.01);
    }
}
