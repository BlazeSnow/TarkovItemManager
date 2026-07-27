use super::model::{Prerequisite, Upgrade};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
struct NamedFile {
    version: u8,
    items: Option<Vec<Named>>,
    facilities: Option<Vec<Named>>,
}
#[derive(Debug, Deserialize)]
struct Named {
    id: String,
    name: String,
}
#[derive(Debug, Deserialize)]
struct HideoutFile {
    version: u8,
    upgrades: Vec<Upgrade>,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub items: BTreeMap<String, String>,
    pub facilities: BTreeMap<String, String>,
    pub upgrades: Vec<Upgrade>,
}
impl Catalog {
    pub fn load(dir: &Path) -> Result<Self> {
        let items = read_named(dir.join("items.json"), "items")?;
        let items_cn = read_named(dir.join("items.cn.json"), "items.cn")?;
        let facilities = read_facilities(dir.join("facilities.json"), "facilities")?;
        let facilities_cn = read_facilities(dir.join("facilities.cn.json"), "facilities.cn")?;
        let hideout: HideoutFile = read_json(dir.join("hideout.json"), "hideout")?;
        if hideout.version != 1 {
            bail!("hideout.json: 不支持的数据集版本 {}", hideout.version);
        }
        ensure_same_ids(&items, &items_cn, "items", "items.cn")?;
        ensure_same_ids(&facilities, &facilities_cn, "facilities", "facilities.cn")?;
        let item_names = indexed_names(items_cn, "items.cn")?;
        let facility_names = indexed_names(facilities_cn, "facilities.cn")?;
        let mut seen = HashSet::new();
        for upgrade in &hideout.upgrades {
            if upgrade.level < 1 {
                bail!("{} 等级必须大于 0", upgrade.facility_id);
            }
            if !facility_names.contains_key(&upgrade.facility_id) {
                bail!("升级引用了未知设施: {}", upgrade.facility_id);
            }
            if !seen.insert((upgrade.facility_id.clone(), upgrade.level)) {
                bail!("设施 {} 的等级 {} 重复", upgrade.facility_id, upgrade.level);
            }
            for requirement in &upgrade.requirements {
                if requirement.quantity < 1 || !item_names.contains_key(&requirement.item_id) {
                    bail!(
                        "设施 {} 的材料引用无效: {}",
                        upgrade.facility_id,
                        requirement.item_id
                    );
                }
            }
            for prerequisite in &upgrade.prerequisites {
                if prerequisite.level < 1 || !key_exists(&hideout.upgrades, prerequisite) {
                    bail!("设施 {} 的前置条件无效", upgrade.facility_id);
                }
            }
        }
        validate_cycles(&hideout.upgrades)?;
        Ok(Self {
            items: item_names,
            facilities: facility_names,
            upgrades: hideout.upgrades,
        })
    }
}
fn read_json<T: for<'a> Deserialize<'a>>(path: PathBuf, label: &str) -> Result<T> {
    let content =
        fs::read_to_string(&path).with_context(|| format!("无法读取 {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("{} 不是有效 JSON", label))
}
fn read_named(path: PathBuf, label: &str) -> Result<Vec<Named>> {
    let file: NamedFile = read_json(path, label)?;
    if file.version != 1 {
        bail!("{} 数据集版本不受支持", label)
    }
    file.items
        .ok_or_else(|| anyhow::anyhow!("{} 缺少 items", label))
}
fn read_facilities(path: PathBuf, label: &str) -> Result<Vec<Named>> {
    let file: NamedFile = read_json(path, label)?;
    if file.version != 1 {
        bail!("{} 数据集版本不受支持", label)
    }
    file.facilities
        .ok_or_else(|| anyhow::anyhow!("{} 缺少 facilities", label))
}
fn indexed_names(records: Vec<Named>, label: &str) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for record in records {
        if record.id.trim().is_empty()
            || record.name.trim().is_empty()
            || result.insert(record.id.clone(), record.name).is_some()
        {
            bail!("{} 包含空或重复 ID", label)
        }
    }
    Ok(result)
}
fn ensure_same_ids(left: &[Named], right: &[Named], a: &str, b: &str) -> Result<()> {
    if left.iter().map(|r| &r.id).collect::<HashSet<_>>() != right.iter().map(|r| &r.id).collect() {
        bail!("{} 与 {} 的 ID 集合不一致", a, b)
    }
    Ok(())
}
fn key_exists(upgrades: &[Upgrade], p: &Prerequisite) -> bool {
    upgrades
        .iter()
        .any(|u| u.facility_id == p.facility_id && u.level == p.level)
}
fn validate_cycles(upgrades: &[Upgrade]) -> Result<()> {
    fn visit(
        key: &(String, i64),
        upgrades: &[Upgrade],
        active: &mut HashSet<(String, i64)>,
        done: &mut HashSet<(String, i64)>,
    ) -> Result<()> {
        if done.contains(key) {
            return Ok(());
        }
        if !active.insert(key.clone()) {
            bail!("设施前置条件存在循环依赖")
        };
        let u = upgrades
            .iter()
            .find(|u| u.facility_id == key.0 && u.level == key.1)
            .unwrap();
        for p in &u.prerequisites {
            visit(&(p.facility_id.clone(), p.level), upgrades, active, done)?
        }
        active.remove(key);
        done.insert(key.clone());
        Ok(())
    }
    let mut a = HashSet::new();
    let mut d = HashSet::new();
    for u in upgrades {
        visit(&(u.facility_id.clone(), u.level), upgrades, &mut a, &mut d)?
    }
    Ok(())
}
