//! Materials, grades, made pieces, wear, crafting and the trade in them.

use gahturiyu_sim::sim::{
    items::{self, catalogue, item, variant, Kind},
    materials::{Grade, Material},
};

#[test]
fn the_catalogue_holds_every_form_in_its_materials_and_grades() {
    let cat = catalogue();
    assert!(cat.defs.len() > 400 && cat.defs.len() < 60_000, "{} entries", cat.defs.len());
    let sword = items::id("longsword");
    // The usual, common version is the plain item itself.
    assert_eq!(variant(sword, Material::Forgeiron, Material::None, Grade::Common), Some(sword));
    let fine = variant(sword, Material::Forgeiron, Material::None, Grade::Fine).unwrap();
    let edge = variant(sword, Material::Edgeglass, Material::None, Grade::Common).unwrap();
    let both = variant(sword, Material::Edgeglass, Material::Forgeiron, Grade::Common).unwrap();
    let w = |id| *item(id).weapon().unwrap();
    assert!(w(fine).cut > w(sword).cut && item(fine).value > item(sword).value);
    assert!(w(edge).cut > w(sword).cut, "Edgeglass takes the keener edge");
    assert!(w(sword).pierce > w(edge).pierce, "Forgeiron gets through armour better");
    assert!(w(both).cut > w(sword).cut && w(both).pierce >= w(sword).pierce, "the two-tradition sword has both");
    assert!(!items::repairable(edge) && items::repairable(sword));
    println!("{} · {} · {}", item(fine).name, item(edge).name, item(both).name);
    // Nacre turns cuts but not blows.
    let coat = items::id("scale_hauberk");
    let nacre = variant(coat, Material::Nacre, Material::Slatewing, Grade::Common).unwrap();
    let (a, b) = (item(coat).armor().unwrap(), item(nacre).armor().unwrap());
    assert!(b.cut > a.cut && b.blunt < a.blunt);
    assert!(matches!(item(items::id("manual_smithing")).kind, Kind::Manual(_)));
}
