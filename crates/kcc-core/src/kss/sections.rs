/// Bulgarian KSS section definitions per Образец 9.1.
/// Each section maps to a SEK group and has a canonical Bulgarian title.

#[derive(Debug, Clone)]
pub struct KssSectionDef {
    pub number: &'static str,
    pub title_bg: &'static str,
    pub sek_group: &'static str,
}

/// All 23 standard KSS sections in order per Образец 9.1.
pub const KSS_SECTIONS: &[KssSectionDef] = &[
    KssSectionDef { number: "I",      title_bg: "ЗЕМНИ РАБОТИ",                sek_group: "СЕК01" },
    KssSectionDef { number: "II",     title_bg: "КОФРАЖНИ РАБОТИ",             sek_group: "СЕК02" },
    KssSectionDef { number: "III",    title_bg: "АРМИРОВЪЧНИ РАБОТИ",          sek_group: "СЕК03" },
    KssSectionDef { number: "IV",     title_bg: "БЕТОНОВИ РАБОТИ",             sek_group: "СЕК04" },
    KssSectionDef { number: "V",      title_bg: "ЗИДАРСКИ РАБОТИ",             sek_group: "СЕК05" },
    KssSectionDef { number: "VI",     title_bg: "ПОКРИВНИ РАБОТИ",             sek_group: "СЕК06" },
    KssSectionDef { number: "VII",    title_bg: "ТЕНЕКЕДЖИЙСКИ РАБОТИ",        sek_group: "СЕК07" },
    KssSectionDef { number: "VIII",   title_bg: "ДЪРВОДЕЛСКИ РАБОТИ",          sek_group: "СЕК08" },
    KssSectionDef { number: "IX",     title_bg: "ОБЛИЦОВЪЧНИ РАБОТИ",          sek_group: "СЕК09" },
    KssSectionDef { number: "X",      title_bg: "МАЗАЧЕСКИ РАБОТИ",            sek_group: "СЕК10" },
    KssSectionDef { number: "XI",     title_bg: "НАСТИЛКИ И ЗАМАЗКИ",          sek_group: "СЕК11" },
    KssSectionDef { number: "XII",    title_bg: "СТЪКЛАРСКИ РАБОТИ",           sek_group: "СЕК12" },
    KssSectionDef { number: "XIII",   title_bg: "БОЯДЖИЙСКИ РАБОТИ",           sek_group: "СЕК13" },
    KssSectionDef { number: "XIV",    title_bg: "МЕТАЛНИ КОНСТРУКЦИИ И ДОГРАМА", sek_group: "СЕК14" },
    KssSectionDef { number: "XV",     title_bg: "ХИДРОИЗОЛАЦИИ",              sek_group: "СЕК15" },
    KssSectionDef { number: "XVI",    title_bg: "ТОПЛОИЗОЛАЦИИ",              sek_group: "СЕК16" },
    KssSectionDef { number: "XVII",   title_bg: "СТОЛАРСКИ РАБОТИ",           sek_group: "СЕК17" },
    KssSectionDef { number: "XVIII",  title_bg: "СУХО СТРОИТЕЛСТВО",          sek_group: "СЕК20" },
    KssSectionDef { number: "XIX",    title_bg: "СГРАДНИ ВИК ИНСТАЛАЦИИ",     sek_group: "СЕК22" },
    KssSectionDef { number: "XX",     title_bg: "ВЪНШНИ ВОДОПРОВОДИ И КАНАЛИЗАЦИИ", sek_group: "СЕК23" },
    KssSectionDef { number: "XXI",    title_bg: "ЕЛЕКТРИЧЕСКА ИНСТАЛАЦИЯ",    sek_group: "СЕК34" },
    KssSectionDef { number: "XXII",   title_bg: "ОТОПЛИТЕЛНА ИНСТАЛАЦИЯ",     sek_group: "СЕК18" },
    KssSectionDef { number: "XXIII",  title_bg: "ПЪТИЩА, УЛИЦИ, ТРОТОАРИ",    sek_group: "СЕК26" },
];

/// Find the KSS section definition for a given SEK group code.
pub fn section_for_sek_group(sek_group: &str) -> Option<&'static KssSectionDef> {
    KSS_SECTIONS.iter().find(|s| s.sek_group == sek_group)
}

/// Roman numeral index (0-based) for ordering sections.
pub fn section_index(number: &str) -> usize {
    KSS_SECTIONS.iter().position(|s| s.number == number).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_lookup() {
        let sec = section_for_sek_group("СЕК05").unwrap();
        assert_eq!(sec.number, "V");
        assert_eq!(sec.title_bg, "ЗИДАРСКИ РАБОТИ");
    }

    #[test]
    fn test_section_ordering() {
        assert!(section_index("I") < section_index("V"));
        assert!(section_index("V") < section_index("XXIII"));
    }

    #[test]
    fn test_all_sections_present() {
        assert_eq!(KSS_SECTIONS.len(), 23);
    }
}
