/// Maps DXF layer names to KSS SEK groups for quantity extraction.
///
/// Bulgarian architectural DWG files use layer naming conventions that
/// directly correspond to construction work categories (СМР).

/// A mapping result: layer name matched to a SEK group + description.
#[derive(Debug, Clone)]
pub struct LayerMapping {
    pub sek_group: &'static str,
    pub work_description_bg: &'static str,
    pub unit: &'static str,
    pub quantity_method: QuantityMethod,
}

/// How to calculate quantity from entities on this layer.
#[derive(Debug, Clone, Copy)]
pub enum QuantityMethod {
    /// Sum line lengths × wall height → М2
    WallArea,
    /// Sum line lengths × wall height × thickness → М3
    WallVolume,
    /// Sum closed polyline areas → М2
    FloorArea,
    /// Count INSERT block references → бр.
    BlockCount,
    /// Sum line lengths → м
    LinearLength,
    /// Sum entity areas directly → М2
    DirectArea,
    /// Steel member weight → кг
    SteelWeight,
}

/// Pattern-based layer name rules. Checked in order; first match wins.
static LAYER_RULES: &[(&str, LayerMapping)] = &[
    // Masonry layers
    ("steni-gazobeton", LayerMapping {
        sek_group: "СЕК05", work_description_bg: "Зидария от газобетонни блокчета",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    ("steni-tuhla", LayerMapping {
        sek_group: "СЕК05", work_description_bg: "Тухлена зидария",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    // Concrete layers
    ("steni-beton", LayerMapping {
        sek_group: "СЕК04", work_description_bg: "Бетонови стени",
        unit: "М3", quantity_method: QuantityMethod::WallVolume,
    }),
    // Drywall
    ("gipskarton", LayerMapping {
        sek_group: "СЕК20", work_description_bg: "Преградни стени от гипсокартон",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    // Steel / metal
    ("metal", LayerMapping {
        sek_group: "СЕК14", work_description_bg: "Стоманени конструкции",
        unit: "кг", quantity_method: QuantityMethod::SteelWeight,
    }),
    // Reinforcement
    ("arm", LayerMapping {
        sek_group: "СЕК03", work_description_bg: "Армировъчни работи",
        unit: "кг", quantity_method: QuantityMethod::SteelWeight,
    }),
    // Insulation
    ("isolation", LayerMapping {
        sek_group: "СЕК16", work_description_bg: "Топ��оизолация по външни стени",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    // Roofing
    ("pokriv-", LayerMapping {
        sek_group: "СЕК06", work_description_bg: "Покривни работи",
        unit: "М2", quantity_method: QuantityMethod::DirectArea,
    }),
    // Facade
    ("fasad-", LayerMapping {
        sek_group: "СЕК10", work_description_bg: "Външна мазилка по фасада",
        unit: "М2", quantity_method: QuantityMethod::DirectArea,
    }),
    // Plumbing
    ("vik", LayerMapping {
        sek_group: "СЕК22", work_description_bg: "ВиК инсталации",
        unit: "м", quantity_method: QuantityMethod::LinearLength,
    }),
    // Electrical
    ("elektr", LayerMapping {
        sek_group: "СЕК34", work_description_bg: "Електрическа инсталация",
        unit: "м", quantity_method: QuantityMethod::LinearLength,
    }),
    // Bolts (steel connections)
    ("bolt", LayerMapping {
        sek_group: "СЕК14", work_description_bg: "Болтови съединения",
        unit: "бр.", quantity_method: QuantityMethod::BlockCount,
    }),
    // Structural layer (generic — needs context to split between concrete/steel)
    ("kon", LayerMapping {
        sek_group: "СЕК04", work_description_bg: "Конструктивни елементи",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),

    // === КАБ Графичен стандарт (Chamber of Architects, 2019) ===
    // Layer names per the Bulgarian architectural AutoCAD template.
    ("a-wall-load", LayerMapping {
        sek_group: "СЕК05", work_description_bg: "Носещи стени (зидария)",
        unit: "М3", quantity_method: QuantityMethod::WallVolume,
    }),
    ("a-wall-part", LayerMapping {
        sek_group: "СЕК05", work_description_bg: "Преградни стени (зидария)",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    ("стени-носещи", LayerMapping {
        sek_group: "СЕК05", work_description_bg: "Носещи стени (зидария)",
        unit: "М3", quantity_method: QuantityMethod::WallVolume,
    }),
    ("стени-преградни", LayerMapping {
        sek_group: "СЕК05", work_description_bg: "Преградни стени (зидария)",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    ("zid-25", LayerMapping {
        sek_group: "СЕК05", work_description_bg: "Тухлена зидария 25 см",
        unit: "М3", quantity_method: QuantityMethod::WallVolume,
    }),
    ("zid-12", LayerMapping {
        sek_group: "СЕК05", work_description_bg: "Тухлена зидария 12 см",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    ("a-glaz", LayerMapping {
        sek_group: "СЕК17", work_description_bg: "PVC/алуминиева дограма — прозорци",
        unit: "М2", quantity_method: QuantityMethod::DirectArea,
    }),
    ("прозорци", LayerMapping {
        sek_group: "СЕК17", work_description_bg: "Прозорци",
        unit: "М2", quantity_method: QuantityMethod::DirectArea,
    }),
    ("a-door", LayerMapping {
        sek_group: "СЕК17", work_description_bg: "Врати",
        unit: "бр.", quantity_method: QuantityMethod::BlockCount,
    }),
    ("врати", LayerMapping {
        sek_group: "СЕК17", work_description_bg: "Врати",
        unit: "бр.", quantity_method: QuantityMethod::BlockCount,
    }),
    ("a-stair", LayerMapping {
        sek_group: "СЕК11", work_description_bg: "Стълбищни настилки",
        unit: "М2", quantity_method: QuantityMethod::DirectArea,
    }),
    ("стълби", LayerMapping {
        sek_group: "СЕК11", work_description_bg: "Стълбищни настилки",
        unit: "М2", quantity_method: QuantityMethod::DirectArea,
    }),
    ("обзавеждане", LayerMapping {
        // Furniture layer — no SEK category, but we still map it so that the
        // classifier recognises it as non-structural and skips it.
        sek_group: "СЕК49", work_description_bg: "Обзавеждане (информативно)",
        unit: "бр.", quantity_method: QuantityMethod::BlockCount,
    }),
    // Renovation / demolition patterns
    ("demont", LayerMapping {
        sek_group: "СЕК49", work_description_bg: "Демонтажни работи",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    ("razkart", LayerMapping {
        sek_group: "СЕК49", work_description_bg: "Разкъртване",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    ("стар", LayerMapping {
        sek_group: "СЕК49", work_description_bg: "Съществуващи елементи (демонтаж)",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),

    // === Generic / English patterns ===
    ("wall", LayerMapping {
        sek_group: "СЕК05", work_description_bg: "Стени (зидария)",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    ("ceiling", LayerMapping {
        sek_group: "СЕК11", work_description_bg: "Тавани",
        unit: "М2", quantity_method: QuantityMethod::DirectArea,
    }),
    ("floor", LayerMapping {
        sek_group: "СЕК11", work_description_bg: "Подови настилки",
        unit: "М2", quantity_method: QuantityMethod::FloorArea,
    }),
    ("roof", LayerMapping {
        sek_group: "СЕК06", work_description_bg: "Покривни работи",
        unit: "М2", quantity_method: QuantityMethod::DirectArea,
    }),
    ("door", LayerMapping {
        sek_group: "СЕК17", work_description_bg: "Врати",
        unit: "бр.", quantity_method: QuantityMethod::BlockCount,
    }),
    ("window", LayerMapping {
        sek_group: "СЕК17", work_description_bg: "Прозорци",
        unit: "бр.", quantity_method: QuantityMethod::BlockCount,
    }),
    ("plumb", LayerMapping {
        sek_group: "СЕК22", work_description_bg: "ВиК инсталации",
        unit: "м", quantity_method: QuantityMethod::LinearLength,
    }),
    ("electr", LayerMapping {
        sek_group: "СЕК34", work_description_bg: "Електрическа инсталация",
        unit: "м", quantity_method: QuantityMethod::LinearLength,
    }),
    ("insul", LayerMapping {
        sek_group: "СЕК16", work_description_bg: "Топлоизолация",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    ("drywall", LayerMapping {
        sek_group: "СЕК20", work_description_bg: "Сухо строителство",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),

    // === Portuguese patterns ===
    ("vista", LayerMapping {
        sek_group: "СЕК05", work_description_bg: "Стени (архитектурен план)",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    ("alvenaria", LayerMapping {
        sek_group: "СЕК05", work_description_bg: "Зидария",
        unit: "М2", quantity_method: QuantityMethod::WallArea,
    }),
    ("cobertura", LayerMapping {
        sek_group: "СЕК06", work_description_bg: "Покривни работи",
        unit: "М2", quantity_method: QuantityMethod::DirectArea,
    }),
    ("piso", LayerMapping {
        sek_group: "СЕК11", work_description_bg: "Подови настилки",
        unit: "М2", quantity_method: QuantityMethod::FloorArea,
    }),
    ("forro", LayerMapping {
        sek_group: "СЕК11", work_description_bg: "Тавани",
        unit: "М2", quantity_method: QuantityMethod::DirectArea,
    }),
    ("hidra", LayerMapping {
        sek_group: "СЕК22", work_description_bg: "ВиК инсталации",
        unit: "м", quantity_method: QuantityMethod::LinearLength,
    }),
    ("elet", LayerMapping {
        sek_group: "СЕК34", work_description_bg: "Електрическа инсталация",
        unit: "м", quantity_method: QuantityMethod::LinearLength,
    }),

    // === Layers to SKIP (not construction work) ===
    ("texto", LayerMapping {
        sek_group: "", work_description_bg: "",
        unit: "", quantity_method: QuantityMethod::BlockCount, // will produce 0 from text
    }),
    ("carimbo", LayerMapping {
        sek_group: "", work_description_bg: "",
        unit: "", quantity_method: QuantityMethod::BlockCount,
    }),
    ("defpoint", LayerMapping {
        sek_group: "", work_description_bg: "",
        unit: "", quantity_method: QuantityMethod::BlockCount,
    }),
];

/// Block name patterns that map to fixture counts.
static BLOCK_RULES: &[(&str, &str, &str)] = &[
    // Bulgarian
    ("door",        "СЕК17", "Доставка и монтаж на врата"),
    ("vrati",       "СЕК17", "Доставка и монтаж на врата"),
    ("prozorci",    "СЕК17", "Доставка и монтаж на прозорец"),
    ("wc",          "СЕК22", "Доставка и монтаж на тоалетна"),
    ("bas",         "СЕК22", "Доставка и монтаж на мивка"),
    ("van",         "СЕК22", "Доставка и монтаж на вана"),
    ("bid",         "СЕК22", "Доставка и монтаж на биде"),
    ("duk",         "СЕК22", "Доставка и монтаж на душ кабина"),
    // Portuguese
    ("bacia",       "СЕК22", "Тоалетна (bacia sanitária)"),
    ("chuveiro",    "СЕК22", "Душ (chuveiro)"),
    ("cuba",        "СЕК22", "Мивка (cuba/lavatório)"),
    ("misturador",  "СЕК22", "Смесител (misturador)"),
    ("lavatorio",   "СЕК22", "Умивалник (lavatório)"),
    ("porta",       "СЕК17", "Врата (porta)"),
    ("janela",      "СЕК17", "Прозорец (janela)"),
    // English
    ("toilet",      "СЕК22", "Тоалетна (toilet)"),
    ("shower",      "СЕК22", "Душ (shower)"),
    ("sink",        "СЕК22", "Мивка (sink)"),
    ("basin",       "СЕК22", "Умивалник (basin)"),
    ("bath",        "СЕК22", "Вана (bath)"),
    ("faucet",      "СЕК22", "Смесител (faucet)"),
    ("window",      "СЕК17", "Прозорец (window)"),
];

/// Match a layer name (case-insensitive) to a KSS mapping.
pub fn map_layer(layer_name: &str) -> Option<&'static LayerMapping> {
    let lower = layer_name.to_lowercase();
    for (pattern, mapping) in LAYER_RULES {
        if lower.contains(pattern) {
            return Some(mapping);
        }
    }
    None
}

/// Match a block name to a fixture mapping: (sek_group, description_bg).
pub fn map_block(block_name: &str) -> Option<(&'static str, &'static str)> {
    let lower = block_name.to_lowercase();
    for (pattern, sek_group, desc) in BLOCK_RULES {
        if lower.contains(pattern) {
            return Some((sek_group, desc));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_mapping() {
        let m = map_layer("0-steni-gazobeton").unwrap();
        assert_eq!(m.sek_group, "СЕК05");
        assert_eq!(m.unit, "М2");
    }

    #[test]
    fn test_layer_case_insensitive() {
        assert!(map_layer("GIPSKARTON").is_some());
        assert!(map_layer("Metal").is_some());
    }

    #[test]
    fn test_block_mapping() {
        let (group, _desc) = map_block("Door").unwrap();
        assert_eq!(group, "СЕК17");

        let (group, _desc) = map_block("WC1").unwrap();
        assert_eq!(group, "СЕК22");
    }

    #[test]
    fn test_unknown_layer() {
        assert!(map_layer("0-mebeli").is_none());
        // Defpoints now matches the "defpoint" skip pattern (empty sek_group)
        let dp = map_layer("Defpoints").unwrap();
        assert!(dp.sek_group.is_empty());
    }

    #[test]
    fn test_portuguese_layer() {
        let m = map_layer("Vista 1").unwrap();
        assert_eq!(m.sek_group, "СЕК05");
    }

    #[test]
    fn test_english_layer() {
        let m = map_layer("Walls-Exterior").unwrap();
        assert_eq!(m.sek_group, "СЕК05");
    }

    #[test]
    fn test_portuguese_block() {
        let (group, _) = map_block("LOUÇA - bacia com caixa deca").unwrap();
        assert_eq!(group, "СЕК22");
        let (group, _) = map_block("METAL - chuveiro deca").unwrap();
        assert_eq!(group, "СЕК22");
    }
}
