use super::loader::Catalog;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct CatalogProgress {
    pub facilities: Vec<FacilityProgress>,
    pub materials: Vec<MaterialProgress>,
}
#[derive(Debug, Clone)]
pub struct FacilityProgress {
    pub id: String,
    pub name: String,
    pub max_level: i64,
    pub current_level: i64,
    pub prerequisites: Vec<PendingPrerequisite>,
}
#[derive(Debug, Clone)]
pub struct PendingPrerequisite {
    pub upgrade_level: i64,
    pub facility_id: String,
    pub facility_name: String,
    pub level: i64,
    pub satisfied: bool,
}
#[derive(Debug, Clone)]
pub struct MaterialProgress {
    pub id: String,
    pub name: String,
    pub quantity: i64,
    pub checked: bool,
}

pub fn calculate(
    catalog: &Catalog,
    current_levels: &HashMap<String, i64>,
    checked: &HashSet<String>,
) -> CatalogProgress {
    let max_levels = maximum_levels(&catalog.upgrades);
    let facilities = catalog
        .facilities
        .iter()
        .map(|(id, name)| {
            let current_level = *current_levels.get(id).unwrap_or(&0);
            let prerequisites = catalog
                .upgrades
                .iter()
                .filter(|u| u.facility_id == *id && u.level > current_level)
                .flat_map(|u| {
                    u.prerequisites.iter().map(move |p| PendingPrerequisite {
                        upgrade_level: u.level,
                        facility_id: p.facility_id.clone(),
                        facility_name: catalog.facilities[&p.facility_id].clone(),
                        level: p.level,
                        satisfied: current_levels.get(&p.facility_id).copied().unwrap_or(0)
                            >= p.level,
                    })
                })
                .collect();
            FacilityProgress {
                id: id.clone(),
                name: name.clone(),
                max_level: max_levels[id],
                current_level,
                prerequisites,
            }
        })
        .collect();
    let mut quantities = BTreeMap::new();
    for upgrade in &catalog.upgrades {
        if upgrade.level
            > current_levels
                .get(&upgrade.facility_id)
                .copied()
                .unwrap_or(0)
        {
            for r in &upgrade.requirements {
                *quantities.entry(r.item_id.clone()).or_default() += r.quantity
            }
        }
    }
    let materials = quantities
        .into_iter()
        .map(|(id, quantity)| MaterialProgress {
            name: catalog.items[&id].clone(),
            checked: checked.contains(&id),
            id,
            quantity,
        })
        .collect();
    CatalogProgress {
        facilities,
        materials,
    }
}
pub fn maximum_levels(upgrades: &[super::model::Upgrade]) -> HashMap<String, i64> {
    upgrades.iter().fold(HashMap::new(), |mut all, u| {
        all.entry(u.facility_id.clone())
            .and_modify(|l| *l = (*l).max(u.level))
            .or_insert(u.level);
        all
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    fn catalog() -> Catalog {
        Catalog::load(Path::new("../dataset")).unwrap()
    }
    fn quantity(p: &CatalogProgress, id: &str) -> i64 {
        p.materials
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.quantity)
            .unwrap_or(0)
    }
    #[test]
    fn remaining_materials() {
        let p = calculate(&catalog(), &HashMap::new(), &HashSet::new());
        assert_eq!(quantity(&p, "screw-nut"), 18)
    }
    #[test]
    fn owned_levels_excluded() {
        let p = calculate(
            &catalog(),
            &HashMap::from([(String::from("generator"), 1)]),
            &HashSet::new(),
        );
        assert_eq!(quantity(&p, "screw-nut"), 14)
    }
}
