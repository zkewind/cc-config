//! 官方供应商种子数据（从 database/dao/providers_seed.rs 迁移）

use crate::app_config::AppType;

pub(crate) struct OfficialProviderSeed {
    pub id: &'static str,
    pub app_type: AppType,
    pub name: &'static str,
    pub website_url: &'static str,
    pub icon: &'static str,
    pub icon_color: &'static str,
    pub settings_config_json: &'static str,
}

pub(crate) const OFFICIAL_SEEDS: &[OfficialProviderSeed] = &[OfficialProviderSeed {
    id: "claude-official",
    app_type: AppType::Claude,
    name: "Claude Official",
    website_url: "https://www.anthropic.com/claude-code",
    icon: "anthropic",
    icon_color: "#D4915D",
    settings_config_json: r#"{"env":{}}"#,
}];

pub(crate) fn is_official_seed_id(id: &str) -> bool {
    OFFICIAL_SEEDS.iter().any(|seed| seed.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_seeds_include_only_claude() {
        assert_eq!(OFFICIAL_SEEDS.len(), 1);
        assert_eq!(OFFICIAL_SEEDS[0].app_type, AppType::Claude);
        assert!(is_official_seed_id("claude-official"));
    }
}
