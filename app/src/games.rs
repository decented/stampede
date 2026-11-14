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
    CatalogEntry {
        name: "Axe'n'Stax",
        blurb: "Build, mine and explore a world you own.",
        package: "com.axenstax.game",
        channel: "axenstax-android-apk",
        // Axe'n'Stax's own release key (its AppImage channel uses the same one) and
        // APK signing certificate (its permanent release keystore, CN=AxeNStax), read
        // off its published 0.2.36 event and APK. Pinned 2026-11-13.
        release_pubkey: Some("ca43e7218d03a50e20b07f54fad58a7cf4ef9ac1a883e3a0104255a46600e502"),
        cert: Some("d27f95670c4c69720dc4b679b82cc54b0ad62b0e34b68a917a1e72ae88a1c642"),
    },
];

/// The entries' icons (PNG, bundled in the app), indexed like [`CATALOG_ENTRIES`].
pub const ICONS: [&[u8]; 3] = [
    include_bytes!("../assets/games/twangthang.png"),
    include_bytes!("../assets/games/clusterflock.png"),
    include_bytes!("../assets/games/axenstax.png"),
];

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
        assert_eq!(CATALOG_ENTRIES[2].package, "com.axenstax.game");
        assert_eq!(CATALOG_ENTRIES[2].channel, "axenstax-android-apk");
        assert_eq!(CATALOG_ENTRIES[2].release_pubkey, Some("ca43e7218d03a50e20b07f54fad58a7cf4ef9ac1a883e3a0104255a46600e502"));
        assert_eq!(CATALOG_ENTRIES[2].cert, Some("d27f95670c4c69720dc4b679b82cc54b0ad62b0e34b68a917a1e72ae88a1c642"));
        assert_eq!(CATALOG_ENTRIES[2].name, "Axe'n'Stax");
        // Each game has its own release key: a shared one would let one game's
        // release stand in for another's.
        for (i, a) in CATALOG_ENTRIES.iter().enumerate() {
            for b in &CATALOG_ENTRIES[i + 1..] {
                assert_ne!(a.release_pubkey, b.release_pubkey, "{} and {} share a release key", a.name, b.name);
            }
        }
    }

    /// Axe'n'Stax's real 0.2.36 release event, as relay.ditto.pub and relay.primal.net
    /// serve it.
    const AXENSTAX_EVENT: &str = r#"{"content":"Softer thunder and a /weather command, More games in the lobby, safer respawns, flat item sprites, bigger chat text with Tab completion.","created_at":1794568039,"id":"76d97989a824d0ebed20e799f644f236ee675daded61c3222405f265a414577f","kind":30063,"pubkey":"ca43e7218d03a50e20b07f54fad58a7cf4ef9ac1a883e3a0104255a46600e502","sig":"ecc4e80dfae5ede7cae071f03a2a06b6c2f4b7b845b4abef73f38e3c74be2cd1a5674efc32332581fe7e6abc4d2a68d8cd2ce93eaa974047b833da344752c210","tags":[["d","axenstax-android-apk"],["version","0.2.36"],["version_code","2036"],["x","dd493d40694deed00292b292f371b5178720447227825da75e28be1ee3e755a6"],["size","20317642"],["cert","d27f95670c4c69720dc4b679b82cc54b0ad62b0e34b68a917a1e72ae88a1c642"],["url","https://nostr.download/dd493d40694deed00292b292f371b5178720447227825da75e28be1ee3e755a6.apk"]]}"#;

    #[test]
    fn the_published_axenstax_release_is_offered_through_the_shipped_catalog() {
        use nostr::Event;
        use nostr_release_kit::{EntryStatus, FetchReport};
        let ev: Event = serde_json::from_str(AXENSTAX_EVENT).unwrap();
        let mut games = crate::release::catalog();
        games.apply_report(&FetchReport { events: vec![ev], answered: 1, total: 3 });
        // Judged by its own entry's key, channel and certificate; the other entries see nothing.
        assert_eq!(games.status(2), EntryStatus::Get { version: "0.2.36".into(), size: 20_317_642 });
        assert_eq!(games.status(0), EntryStatus::NoRelease);
        assert_eq!(games.status(1), EntryStatus::NoRelease);
    }

    #[test]
    fn the_manifest_queries_name_every_catalog_package() {
        let manifest = include_str!("../android/AndroidManifest.xml");
        for e in CATALOG_ENTRIES {
            assert!(manifest.contains(&format!("<package android:name=\"{}\" />", e.package)), "{} missing from <queries>", e.package);
        }
    }
}
