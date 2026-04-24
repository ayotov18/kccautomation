use crate::markup::{ApplyTo, Markup, MarkupType};

/// Available regional markup templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// Bulgaria
    BG,
    /// Germany, Austria, Switzerland
    DACH,
    /// United Kingdom
    UK,
    /// United States
    US,
    /// France
    FR,
    /// Gulf Cooperation Council (UAE, Saudi, Qatar, etc.)
    GULF,
    /// India
    IN,
    /// Australia
    AU,
    /// Japan
    JP,
    /// Russia
    RU,
    /// Brazil
    BR,
    /// China
    CN,
}

impl Region {
    /// Parse a region code string (case-insensitive).
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_uppercase().as_str() {
            "BG" => Some(Region::BG),
            "DACH" => Some(Region::DACH),
            "UK" => Some(Region::UK),
            "US" => Some(Region::US),
            "FR" => Some(Region::FR),
            "GULF" => Some(Region::GULF),
            "IN" => Some(Region::IN),
            "AU" => Some(Region::AU),
            "JP" => Some(Region::JP),
            "RU" => Some(Region::RU),
            "BR" => Some(Region::BR),
            "CN" => Some(Region::CN),
            _ => None,
        }
    }

    /// Get the default markup template for this region.
    pub fn default_markups(&self) -> Vec<Markup> {
        match self {
            Region::BG => bg_markups(),
            Region::DACH => dach_markups(),
            Region::UK => uk_markups(),
            Region::US => us_markups(),
            Region::FR => fr_markups(),
            Region::GULF => gulf_markups(),
            Region::IN => in_markups(),
            Region::AU => au_markups(),
            Region::JP => jp_markups(),
            Region::RU => ru_markups(),
            Region::BR => br_markups(),
            Region::CN => cn_markups(),
        }
    }
}

fn bg_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "\u{0414}\u{043e}\u{043f}\u{044a}\u{043b}\u{043d}\u{0438}\u{0442}\u{0435}\u{043b}\u{043d}\u{0438} \u{0440}\u{0430}\u{0437}\u{0445}\u{043e}\u{0434}\u{0438}".into(), // Допълнителни разходи
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "\u{041f}\u{0435}\u{0447}\u{0430}\u{043b}\u{0431}\u{0430}".into(), // Печалба
            markup_type: MarkupType::Percentage,
            percentage: 8.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "\u{041d}\u{0435}\u{043f}\u{0440}\u{0435}\u{0434}\u{0432}\u{0438}\u{0434}\u{0435}\u{043d}\u{0438}".into(), // Непредвидени
            markup_type: MarkupType::Percentage,
            percentage: 5.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 3,
            is_active: true,
        },
        Markup {
            name: "\u{0414}\u{0414}\u{0421}".into(), // ДДС
            markup_type: MarkupType::Percentage,
            percentage: 20.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::Cumulative,
            sort_order: 4,
            is_active: true,
        },
    ]
}

fn dach_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "Baustellengemeinkosten (BGK)".into(),
            markup_type: MarkupType::Percentage,
            percentage: 12.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "Allgemeine Gesch\u{00e4}ftskosten (AGK)".into(),
            markup_type: MarkupType::Percentage,
            percentage: 8.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "Wagnis und Gewinn (W+G)".into(),
            markup_type: MarkupType::Percentage,
            percentage: 6.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 3,
            is_active: true,
        },
        Markup {
            name: "MwSt.".into(),
            markup_type: MarkupType::Percentage,
            percentage: 19.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::Cumulative,
            sort_order: 4,
            is_active: true,
        },
    ]
}

fn uk_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "Preliminaries".into(),
            markup_type: MarkupType::Percentage,
            percentage: 12.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "Overheads & Profit".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "Contingency".into(),
            markup_type: MarkupType::Percentage,
            percentage: 5.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 3,
            is_active: true,
        },
        Markup {
            name: "VAT".into(),
            markup_type: MarkupType::Percentage,
            percentage: 20.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::Cumulative,
            sort_order: 4,
            is_active: true,
        },
    ]
}

fn us_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "General Conditions".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "Overhead".into(),
            markup_type: MarkupType::Percentage,
            percentage: 8.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "Profit".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 3,
            is_active: true,
        },
        Markup {
            name: "Contingency".into(),
            markup_type: MarkupType::Percentage,
            percentage: 5.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 4,
            is_active: true,
        },
    ]
}

fn fr_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "Frais de chantier".into(),
            markup_type: MarkupType::Percentage,
            percentage: 12.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "Frais g\u{00e9}n\u{00e9}raux".into(),
            markup_type: MarkupType::Percentage,
            percentage: 8.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "B\u{00e9}n\u{00e9}fice".into(),
            markup_type: MarkupType::Percentage,
            percentage: 6.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 3,
            is_active: true,
        },
        Markup {
            name: "TVA".into(),
            markup_type: MarkupType::Percentage,
            percentage: 20.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::Cumulative,
            sort_order: 4,
            is_active: true,
        },
    ]
}

fn gulf_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "General Requirements".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "Overhead".into(),
            markup_type: MarkupType::Percentage,
            percentage: 8.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "Profit".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 3,
            is_active: true,
        },
        Markup {
            name: "VAT".into(),
            markup_type: MarkupType::Percentage,
            percentage: 5.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::Cumulative,
            sort_order: 4,
            is_active: true,
        },
    ]
}

fn in_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "Site Overheads".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "Head Office Overheads".into(),
            markup_type: MarkupType::Percentage,
            percentage: 5.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "Profit".into(),
            markup_type: MarkupType::Percentage,
            percentage: 8.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 3,
            is_active: true,
        },
        Markup {
            name: "GST".into(),
            markup_type: MarkupType::Percentage,
            percentage: 18.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::Cumulative,
            sort_order: 4,
            is_active: true,
        },
    ]
}

fn au_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "Preliminaries".into(),
            markup_type: MarkupType::Percentage,
            percentage: 15.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "Margin".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "Contingency".into(),
            markup_type: MarkupType::Percentage,
            percentage: 5.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 3,
            is_active: true,
        },
        Markup {
            name: "GST".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::Cumulative,
            sort_order: 4,
            is_active: true,
        },
    ]
}

fn jp_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "General Expenses".into(),
            markup_type: MarkupType::Percentage,
            percentage: 15.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "Profit".into(),
            markup_type: MarkupType::Percentage,
            percentage: 8.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "Consumption Tax".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::Cumulative,
            sort_order: 3,
            is_active: true,
        },
    ]
}

fn ru_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "Overhead Costs".into(),
            markup_type: MarkupType::Percentage,
            percentage: 12.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "Estimated Profit".into(),
            markup_type: MarkupType::Percentage,
            percentage: 8.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "NDS (VAT)".into(),
            markup_type: MarkupType::Percentage,
            percentage: 20.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::Cumulative,
            sort_order: 3,
            is_active: true,
        },
    ]
}

fn br_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "BDI (Overhead & Profit)".into(),
            markup_type: MarkupType::Percentage,
            percentage: 25.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "Contingency".into(),
            markup_type: MarkupType::Percentage,
            percentage: 5.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "ISS + PIS/COFINS".into(),
            markup_type: MarkupType::Percentage,
            percentage: 11.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::Cumulative,
            sort_order: 3,
            is_active: true,
        },
    ]
}

fn cn_markups() -> Vec<Markup> {
    vec![
        Markup {
            name: "Indirect Costs".into(),
            markup_type: MarkupType::Percentage,
            percentage: 10.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 1,
            is_active: true,
        },
        Markup {
            name: "Profit".into(),
            markup_type: MarkupType::Percentage,
            percentage: 7.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 2,
            is_active: true,
        },
        Markup {
            name: "Contingency".into(),
            markup_type: MarkupType::Percentage,
            percentage: 5.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::DirectCost,
            sort_order: 3,
            is_active: true,
        },
        Markup {
            name: "VAT".into(),
            markup_type: MarkupType::Percentage,
            percentage: 9.0,
            fixed_amount: 0.0,
            apply_to: ApplyTo::Cumulative,
            sort_order: 4,
            is_active: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::calculate_markups;

    #[test]
    fn test_bg_markups() {
        let markups = Region::BG.default_markups();
        assert_eq!(markups.len(), 4);

        let (results, total) = calculate_markups(100_000.0, &markups);
        // Direct: 100,000
        // Overhead: 10,000 (10% of 100k) -> 110,000
        // Profit: 8,000 (8% of 100k) -> 118,000
        // Contingency: 5,000 (5% of 100k) -> 123,000
        // VAT: 24,600 (20% of 123k) -> 147,600
        assert_eq!(results.len(), 4);
        assert!((results[0].amount - 10_000.0).abs() < 0.01);
        assert!((results[1].amount - 8_000.0).abs() < 0.01);
        assert!((results[2].amount - 5_000.0).abs() < 0.01);
        assert!((results[3].amount - 24_600.0).abs() < 0.01);
        assert!((total - 147_600.0).abs() < 0.01);
    }

    #[test]
    fn test_all_regions_produce_markups() {
        let regions = [
            Region::BG,
            Region::DACH,
            Region::UK,
            Region::US,
            Region::FR,
            Region::GULF,
            Region::IN,
            Region::AU,
            Region::JP,
            Region::RU,
            Region::BR,
            Region::CN,
        ];
        for region in regions {
            let markups = region.default_markups();
            assert!(!markups.is_empty(), "Region {:?} has no markups", region);
            // Every markup should be active
            for m in &markups {
                assert!(m.is_active);
            }
        }
    }

    #[test]
    fn test_region_from_code() {
        assert_eq!(Region::from_code("bg"), Some(Region::BG));
        assert_eq!(Region::from_code("DACH"), Some(Region::DACH));
        assert_eq!(Region::from_code("XX"), None);
    }
}
