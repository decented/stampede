//! More games (spec 20.3): the catalog of other games built into the app, as
//! data. There is no remote list. Each entry pins the game's package, its
//! release channel, the key that signs its release events and the certificate
//! its APK is signed with.
//!
//! The rules (judging an entry's release event by THAT entry's key and channel,
//! the status a card shows, the package and certificate check on the downloaded
//! file) are nostr-release-kit's `Catalog`; Stampede keeps only the entries
//! and their icons here. See `crate::release` for the `Catalog` built from them.

use nostr_release_kit::CatalogEntry;

/// Stampede's catalog. It omits Stampede itself. Keep the manifest `<queries>`
/// in step. Order: as shown (and as [`ICONS`]).
pub const CATALOG_ENTRIES: &[CatalogEntry] = &[CatalogEntry {
    name: "Twang Thang",
    blurb: "A neon energy-band party game.",
    package: "org.decented.twangthang",
    channel: "twangthang-android-apk",
    release_pubkey: Some("2cc65a54774d9c1d6fde7e98255dbe1064940facb10dbc3e49f63300a27178c7"),
    cert: Some("ad02f430148557fb6501c97b16a9ef60681d12d533f7579035e6d62a83dec640"),
}];

/// The entries' icons (PNG, bundled in the app), indexed like [`CATALOG_ENTRIES`].
pub const ICONS: [&[u8]; 1] = [include_bytes!("../assets/games/twangthang.png")];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::release::OWN_PACKAGE;

    #[test]
    fn the_catalog_pins_are_well_formed() {
        assert_eq!(CATALOG_ENTRIES.len(), ICONS.len(), "one icon per entry");
        for (e, icon) in CATALOG_ENTRIES.iter().zip(ICONS) {
            assert!(e.ready(), "{} should be fetchable", e.name);
            assert!(!e.name.is_empty() && !e.blurb.is_empty() && e.blurb.len() < 60 && !icon.is_empty());
            assert_ne!(e.package, OWN_PACKAGE, "the catalog omits Stampede itself");
        }
        assert_eq!(CATALOG_ENTRIES[0].package, "org.decented.twangthang");
        assert_eq!(CATALOG_ENTRIES[0].channel, "twangthang-android-apk");
        // Pinned key and certificate, exactly as shipped.
        assert_eq!(CATALOG_ENTRIES[0].release_pubkey, Some("2cc65a54774d9c1d6fde7e98255dbe1064940facb10dbc3e49f63300a27178c7"));
        assert_eq!(CATALOG_ENTRIES[0].cert, Some("ad02f430148557fb6501c97b16a9ef60681d12d533f7579035e6d62a83dec640"));
        assert_eq!(CATALOG_ENTRIES[0].name, "Twang Thang");
        assert_eq!(CATALOG_ENTRIES[0].blurb, "A neon energy-band party game.");
    }

    #[test]
    fn the_manifest_queries_name_every_catalog_package() {
        let manifest = include_str!("../android/AndroidManifest.xml");
        for e in CATALOG_ENTRIES {
            assert!(manifest.contains(&format!("<package android:name=\"{}\" />", e.package)), "{} missing from <queries>", e.package);
        }
    }
}
