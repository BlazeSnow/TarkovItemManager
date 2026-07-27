use super::loader::{Catalog, Upgrade};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone)]
pub struct CatalogProgress {
    pub facilities: Vec<FacilityProgress>,
    pub materials: Vec<MaterialProgress>,
}
#[derive(Debug, Clone)]
pub struct FacilityProgress {
    pub id: i64,
    pub name: String,
    pub max_level: i64,
    pub current_level: i64,
    pub upgrades: Vec<UpgradeProgress>,
}
#[derive(Debug, Clone)]
pub struct UpgradeProgress {
    pub level: i64,
    pub available: bool,
    pub construction_time_seconds: i64,
    pub requirements: Vec<RequirementProgress>,
    pub facility_prerequisites: Vec<FacilityGate>,
    pub merchant_prerequisites: Vec<MerchantGate>,
    pub skill_prerequisites: Vec<SkillGate>,
    pub source_requirements_available: bool,
}
#[derive(Debug, Clone)]
pub struct RequirementProgress {
    pub item_id: i64,
    pub name: String,
    pub quantity: i64,
    pub found_in_raid: bool,
}
#[derive(Debug, Clone)]
pub struct FacilityGate {
    pub facility_id: i64,
    pub name: String,
    pub level: i64,
    pub satisfied: bool,
}
#[derive(Debug, Clone)]
pub struct MerchantGate {
    pub merchant_id: i64,
    pub name: String,
    pub level: i64,
    pub satisfied: bool,
}
#[derive(Debug, Clone)]
pub struct SkillGate {
    pub name: String,
    pub level: i64,
    pub satisfied: bool,
}
#[derive(Debug, Clone)]
pub struct MaterialProgress {
    pub item_id: i64,
    pub name: String,
    pub quantity: i64,
    pub found_in_raid: bool,
}

pub fn calculate(
    catalog: &Catalog,
    levels: &HashMap<i64, i64>,
    merchant_levels: &HashMap<i64, i64>,
    skill_levels: &HashMap<String, i64>,
) -> CatalogProgress {
    let max = maximum_levels(&catalog.upgrades);
    let facilities = catalog
        .facilities
        .iter()
        .map(|(&id, name)| {
            let current = *levels.get(&id).unwrap_or(&0);
            let upgrades = catalog
                .upgrades
                .iter()
                .filter(|u| u.facility_id == id && u.level > current)
                .map(|u| map_upgrade(catalog, u, levels, merchant_levels, skill_levels))
                .collect();
            FacilityProgress {
                id,
                name: name.clone(),
                max_level: max[&id],
                current_level: current,
                upgrades,
            }
        })
        .collect();
    let mut totals = BTreeMap::new();
    for u in &catalog.upgrades {
        if u.level > *levels.get(&u.facility_id).unwrap_or(&0) {
            for r in &u.requirements {
                *totals.entry((r.item_id, r.found_in_raid)).or_insert(0) += r.quantity;
            }
        }
    }
    let materials = totals
        .into_iter()
        .map(|((id, fir), quantity)| MaterialProgress {
            item_id: id,
            name: catalog.items[&id].clone(),
            quantity,
            found_in_raid: fir,
        })
        .collect();
    CatalogProgress {
        facilities,
        materials,
    }
}
fn map_upgrade(
    c: &Catalog,
    u: &Upgrade,
    levels: &HashMap<i64, i64>,
    merchants: &HashMap<i64, i64>,
    skills: &HashMap<String, i64>,
) -> UpgradeProgress {
    let facility_prerequisites = u
        .facility_prerequisites
        .iter()
        .map(|p| FacilityGate {
            facility_id: p.facility_id,
            name: c.facilities[&p.facility_id].clone(),
            level: p.level,
            satisfied: levels.get(&p.facility_id).copied().unwrap_or(0) >= p.level,
        })
        .collect::<Vec<_>>();
    let merchant_prerequisites = u
        .merchant_prerequisites
        .iter()
        .map(|p| MerchantGate {
            merchant_id: p.merchant_id,
            name: c.merchants[&p.merchant_id].clone(),
            level: p.level,
            satisfied: merchants.get(&p.merchant_id).copied().unwrap_or(0) >= p.level,
        })
        .collect::<Vec<_>>();
    let skill_prerequisites = u
        .skill_prerequisites
        .iter()
        .map(|p| SkillGate {
            name: p.name.clone(),
            level: p.level,
            satisfied: skills.get(&p.name).copied().unwrap_or(0) >= p.level,
        })
        .collect::<Vec<_>>();
    let available = facility_prerequisites.iter().all(|p| p.satisfied)
        && merchant_prerequisites.iter().all(|p| p.satisfied)
        && skill_prerequisites.iter().all(|p| p.satisfied);
    UpgradeProgress {
        level: u.level,
        available,
        construction_time_seconds: u.construction_time_seconds,
        requirements: u
            .requirements
            .iter()
            .map(|r| RequirementProgress {
                item_id: r.item_id,
                name: c.items[&r.item_id].clone(),
                quantity: r.quantity,
                found_in_raid: r.found_in_raid,
            })
            .collect(),
        facility_prerequisites,
        merchant_prerequisites,
        skill_prerequisites,
        source_requirements_available: u.source.source_requirements_available,
    }
}
pub fn maximum_levels(upgrades: &[Upgrade]) -> HashMap<i64, i64> {
    upgrades.iter().fold(HashMap::new(), |mut all, u| {
        all.entry(u.facility_id)
            .and_modify(|l| *l = (*l).max(u.level))
            .or_insert(u.level);
        all
    })
}
