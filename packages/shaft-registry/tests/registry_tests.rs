use enum_map::Enum as _;
use shaft_registry::PkgId;

#[test]
fn test_registry_in_sync() {
    for i in 0..PkgId::LENGTH {
        let pkg = PkgId::from_usize(i);
        assert_eq!(pkg.package().id(), pkg);
    }
}

#[test]
fn test_descriptions_for_all_packages() {
    for i in 0..PkgId::LENGTH {
        let pkg = PkgId::from_usize(i);
        let package = pkg.package();
        if !package.enabled {
            continue;
        }
        assert!(!package.short_desc.is_empty(), "package '{pkg}' is missing docs");
    }
}
