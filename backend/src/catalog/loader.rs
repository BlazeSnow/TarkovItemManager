use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Component, Path},
};

use anyhow::{Context, Result, bail};
use include_dir::{Dir, include_dir};
use serde::Deserialize;

static EMBEDDED_DATASET: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../dataset");

#[derive(Debug, Deserialize)]
struct NamedRecord {
    #[serde(rename = "ID")]
    id: i64,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(rename = "schemaVersion")]
    schema_version: u8,
    #[serde(rename = "gameMode")]
    game_mode: String,
    #[serde(rename = "retrievedAt")]
    retrieved_at: String,
    #[serde(rename = "upgradeFiles")]
    upgrade_files: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Upgrade {
    #[serde(rename = "facilityID")]
    pub facility_id: i64,
    pub level: i64,
    pub requirements: Vec<Requirement>,
    #[serde(rename = "facilityPrerequisites")]
    pub facility_prerequisites: Vec<FacilityPrerequisite>,
    #[serde(rename = "merchantPrerequisites")]
    pub merchant_prerequisites: Vec<MerchantPrerequisite>,
    #[serde(rename = "skillPrerequisites")]
    pub skill_prerequisites: Vec<SkillPrerequisite>,
    #[serde(rename = "taskPrerequisites")]
    pub task_prerequisites: Vec<serde_json::Value>,
    #[serde(rename = "editionPrerequisites")]
    pub edition_prerequisites: Vec<serde_json::Value>,
    #[serde(rename = "constructionTimeSeconds")]
    pub construction_time_seconds: i64,
    pub source: UpgradeSource,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Requirement {
    #[serde(rename = "itemID")]
    pub item_id: i64,
    pub quantity: i64,
    #[serde(rename = "foundInRaid")]
    pub found_in_raid: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FacilityPrerequisite {
    #[serde(rename = "facilityID")]
    pub facility_id: i64,
    pub level: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MerchantPrerequisite {
    #[serde(rename = "merchantID")]
    pub merchant_id: i64,
    pub level: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SkillPrerequisite {
    pub name: String,
    pub level: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpgradeSource {
    pub page: String,
    #[serde(rename = "sourceFacilityName")]
    pub source_facility_name: String,
    #[serde(rename = "sourceRequirementsAvailable")]
    pub source_requirements_available: bool,
    #[serde(rename = "taskAndEditionConditionsProvided")]
    pub task_and_edition_conditions_provided: bool,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub schema_version: u8,
    pub game_mode: String,
    pub retrieved_at: String,
    pub items: BTreeMap<i64, String>,
    pub facilities: BTreeMap<i64, String>,
    pub merchants: BTreeMap<i64, String>,
    pub upgrades: Vec<Upgrade>,
}

trait DatasetSource {
    fn read(&self, relative: &Path) -> Result<Vec<u8>>;
}

struct EmbeddedDataset;

impl DatasetSource for EmbeddedDataset {
    fn read(&self, relative: &Path) -> Result<Vec<u8>> {
        EMBEDDED_DATASET
            .get_file(relative)
            .map(|file| file.contents().to_vec())
            .ok_or_else(|| anyhow::anyhow!("内嵌数据集缺少 {}", relative.display()))
    }
}

struct ExternalDataset<'a> {
    root: &'a Path,
}

impl DatasetSource for ExternalDataset<'_> {
    fn read(&self, relative: &Path) -> Result<Vec<u8>> {
        let path = self.root.join(relative);
        fs::read(&path).with_context(|| format!("无法读取 {}", path.display()))
    }
}

impl Catalog {
    pub fn load_embedded() -> Result<Self> {
        Self::load_from(&EmbeddedDataset)
    }

    pub fn load_external(dir: &Path) -> Result<Self> {
        Self::load_from(&ExternalDataset { root: dir })
    }

    fn load_from(source: &impl DatasetSource) -> Result<Self> {
        let items = load_named(source, Path::new("items.json"), "items")?;
        let facilities = load_named(source, Path::new("facilities.json"), "facilities")?;
        let merchants = load_named(source, Path::new("merchants.json"), "merchants")?;
        let manifest: Manifest = read_json(source, Path::new("hideout.json"), "hideout manifest")?;
        if manifest.schema_version != 1 {
            bail!(
                "不支持的 hideout schemaVersion: {}",
                manifest.schema_version
            );
        }
        if manifest.game_mode != "PVE" {
            bail!("当前只支持 PVE 快照，收到: {}", manifest.game_mode);
        }
        if manifest.retrieved_at.trim().is_empty() || manifest.upgrade_files.is_empty() {
            bail!("hideout manifest 缺少日期或 upgradeFiles");
        }

        let mut seen_files = HashSet::new();
        let mut upgrades = Vec::new();
        for file in &manifest.upgrade_files {
            let relative = Path::new(file);
            if relative.is_absolute()
                || relative.components().any(|part| {
                    matches!(
                        part,
                        Component::ParentDir | Component::RootDir | Component::Prefix(_)
                    )
                })
            {
                bail!("非法 hideout 分片路径: {file}");
            }
            if !seen_files.insert(file) {
                bail!("hideout manifest 包含重复分片: {file}");
            }
            let shard_id = relative
                .file_stem()
                .and_then(|value| value.to_str())
                .and_then(|value| value.parse::<i64>().ok())
                .ok_or_else(|| anyhow::anyhow!("分片文件名必须是设施数字 ID: {file}"))?;
            let shard: Vec<Upgrade> = read_json(source, relative, file)?;
            if shard.is_empty() || shard.iter().any(|upgrade| upgrade.facility_id != shard_id) {
                bail!("分片 {file} 包含错误的 facilityID 或为空");
            }
            upgrades.extend(shard);
        }
        upgrades.sort_by_key(|upgrade| (upgrade.facility_id, upgrade.level));
        validate(&items, &facilities, &merchants, &upgrades)?;

        Ok(Self {
            schema_version: manifest.schema_version,
            game_mode: manifest.game_mode,
            retrieved_at: manifest.retrieved_at,
            items,
            facilities,
            merchants,
            upgrades,
        })
    }

    pub fn skill_names(&self) -> Vec<String> {
        let mut skills: Vec<_> = self
            .upgrades
            .iter()
            .flat_map(|upgrade| {
                upgrade
                    .skill_prerequisites
                    .iter()
                    .map(|skill| skill.name.clone())
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        skills.sort();
        skills
    }
}

fn load_named(
    source: &impl DatasetSource,
    path: &Path,
    label: &str,
) -> Result<BTreeMap<i64, String>> {
    let records: Vec<NamedRecord> = read_json(source, path, label)?;
    let mut result = BTreeMap::new();
    for record in records {
        if record.id < 0
            || record.name.trim().is_empty()
            || result.insert(record.id, record.name).is_some()
        {
            bail!("{label} 包含无效或重复 ID");
        }
    }
    Ok(result)
}

fn read_json<T: for<'a> Deserialize<'a>>(
    source: &impl DatasetSource,
    path: &Path,
    label: &str,
) -> Result<T> {
    let content = source.read(path)?;
    serde_json::from_slice(&content).with_context(|| format!("{label} 不是有效 JSON"))
}

fn validate(
    items: &BTreeMap<i64, String>,
    facilities: &BTreeMap<i64, String>,
    merchants: &BTreeMap<i64, String>,
    upgrades: &[Upgrade],
) -> Result<()> {
    let keys: HashSet<_> = upgrades
        .iter()
        .map(|upgrade| (upgrade.facility_id, upgrade.level))
        .collect();
    if keys.len() != upgrades.len() {
        bail!("升级记录包含重复的 facilityID 和 level");
    }
    for upgrade in upgrades {
        if upgrade.level < 1
            || upgrade.construction_time_seconds < 0
            || !facilities.contains_key(&upgrade.facility_id)
        {
            bail!("升级记录无效: {} Lv.{}", upgrade.facility_id, upgrade.level);
        }
        if !upgrade.source.page.starts_with("https://")
            || upgrade.source.source_facility_name.trim().is_empty()
        {
            bail!(
                "升级来源元数据无效: {} Lv.{}",
                upgrade.facility_id,
                upgrade.level
            );
        }
        for requirement in &upgrade.requirements {
            if requirement.quantity < 1 || !items.contains_key(&requirement.item_id) {
                bail!(
                    "升级材料引用无效: {} Lv.{}",
                    upgrade.facility_id,
                    upgrade.level
                );
            }
        }
        for prerequisite in &upgrade.facility_prerequisites {
            if prerequisite.level < 1
                || !keys.contains(&(prerequisite.facility_id, prerequisite.level))
            {
                bail!(
                    "设施前置条件无效: {} Lv.{}",
                    upgrade.facility_id,
                    upgrade.level
                );
            }
        }
        for prerequisite in &upgrade.merchant_prerequisites {
            if prerequisite.level < 1 || !merchants.contains_key(&prerequisite.merchant_id) {
                bail!(
                    "商人前置条件无效: {} Lv.{}",
                    upgrade.facility_id,
                    upgrade.level
                );
            }
        }
        for skill in &upgrade.skill_prerequisites {
            if skill.name.trim().is_empty() || skill.level < 1 {
                bail!(
                    "技能前置条件无效: {} Lv.{}",
                    upgrade.facility_id,
                    upgrade.level
                );
            }
        }
    }
    validate_cycles(upgrades)
}

fn validate_cycles(upgrades: &[Upgrade]) -> Result<()> {
    let map: HashMap<_, _> = upgrades
        .iter()
        .map(|upgrade| {
            (
                (upgrade.facility_id, upgrade.level),
                &upgrade.facility_prerequisites,
            )
        })
        .collect();
    fn visit(
        key: (i64, i64),
        map: &HashMap<(i64, i64), &Vec<FacilityPrerequisite>>,
        active: &mut HashSet<(i64, i64)>,
        done: &mut HashSet<(i64, i64)>,
    ) -> Result<()> {
        if done.contains(&key) {
            return Ok(());
        }
        if !active.insert(key) {
            bail!("设施前置条件存在循环依赖: {} Lv.{}", key.0, key.1);
        }
        for prerequisite in map[&key] {
            visit(
                (prerequisite.facility_id, prerequisite.level),
                map,
                active,
                done,
            )?;
        }
        active.remove(&key);
        done.insert(key);
        Ok(())
    }
    let mut active = HashSet::new();
    let mut done = HashSet::new();
    for key in map.keys() {
        visit(*key, &map, &mut active, &mut done)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Catalog;
    use std::path::Path;

    #[test]
    fn loads_embedded_dataset() {
        let catalog = Catalog::load_embedded().expect("embedded dataset should load");
        assert_eq!(catalog.game_mode, "PVE");
        assert!(!catalog.items.is_empty());
        assert!(!catalog.upgrades.is_empty());
    }

    #[test]
    fn rejects_missing_external_dataset() {
        assert!(Catalog::load_external(Path::new("missing-dataset")).is_err());
    }
}
