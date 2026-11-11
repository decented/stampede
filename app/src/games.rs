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
pub const CATALOG_ENTRIES: &[CatalogEntry] = &[
    CatalogEntry {
        name: "Twang Thang",
        blurb: "A neon energy-band party game.",
        package: "org.decented.twangthang",
        channel: "twangthang-android-apk",
        release_pubkey: Some("2cc65a54774d9c1d6fde7e98255dbe1064940facb10dbc3e49f63300a27178c7"),
        cert: Some("ad02f430148557fb6501c97b16a9ef60681d12d533f7579035e6d62a83dec640"),
    },
    CatalogEntry {
        name: "Cluster Flock",
        blurb: "Three-lane ostrich farm run, with ghost saboteurs.",
        package: "org.decented.gameflock",
        channel: "gameflock-android-apk",
        // Cluster Flock's own release key and APK signing certificate, as its
        // app/src/release.rs and tools/release/release.json pin them; the certificate
        // was read off its 0.2.1 APK. Pinned 2026-11-11.
        release_pubkey: Some("01b0f270e290f5cd48aa537392de04e4b22124c16d696de96158395a60343b0f"),
        cert: Some("46fd4f7a80b700cd7d951c04e38fa3344405759d53d161de3e4dd06120fb3ace"),
    },
];

/// The entries' icons (PNG, bundled in the app), indexed like [`CATALOG_ENTRIES`].
pub const ICONS: [&[u8]; 2] = [include_bytes!("../assets/games/twangthang.png"), include_bytes!("../assets/games/clusterflock.png")];

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
        assert_eq!(CATALOG_ENTRIES[1].package, "org.decented.gameflock");
        assert_eq!(CATALOG_ENTRIES[1].channel, "gameflock-android-apk");
        assert_eq!(CATALOG_ENTRIES[1].release_pubkey, Some("01b0f270e290f5cd48aa537392de04e4b22124c16d696de96158395a60343b0f"));
        assert_eq!(CATALOG_ENTRIES[1].cert, Some("46fd4f7a80b700cd7d951c04e38fa3344405759d53d161de3e4dd06120fb3ace"));
        assert_eq!(CATALOG_ENTRIES[1].name, "Cluster Flock");
        // Each game has its own release key: a shared one would let one game's
        // release stand in for another's.
        assert_ne!(CATALOG_ENTRIES[0].release_pubkey, CATALOG_ENTRIES[1].release_pubkey);
    }

    #[test]
    fn the_manifest_queries_name_every_catalog_package() {
        let manifest = include_str!("../android/AndroidManifest.xml");
        for e in CATALOG_ENTRIES {
            assert!(manifest.contains(&format!("<package android:name=\"{}\" />", e.package)), "{} missing from <queries>", e.package);
        }
    }
}
