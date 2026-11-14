//! Stampede's side of in-app updates and More games (spec 20): the values and
//! the wiring, nothing else. The rules (the signed kind-30063 release event, the
//! pinned key, the canonical Blossom URLs, hash, size and certificate checks, the
//! `Updater` and `Catalog` state machines, the Android installer) are
//! nostr-release-kit's; `tools/release/release.json` configures the kit's
//! publisher for the same channel, package and key.
//!
//! One function builds the kit's `Config` ([`update_config`]) and one the
//! `CatalogConfig` ([`catalog_config`]); the engine and the pin tests both call
//! them, so what the tests pin is what ships.

use crate::games::CATALOG_ENTRIES;
use crate::nostr_fetch::AppFetcher;
use nostr_release_kit::{Catalog, CatalogConfig, Config, Updater};
use std::path::Path;

/// The `d` tag of this app's Android channel.
pub const RELEASE_CHANNEL: &str = "stampede-android-apk";
/// Read for the release event; also what the publisher writes to (`tools/release/release.json`).
/// Public relays only: the operator's own relay is never a default.
pub const RELEASE_RELAYS: [&str; 3] = ["wss://relay.damus.io", "wss://relay.ditto.pub", "wss://relay.primal.net"];
/// The release key's public half (hex; npub1ka2v5p8ansyaz7uwzgu5aan4g3r4qfdug34vpe2jvavgcgcj3z5qqa984j),
/// Stampede's own (one release key per game). The publisher refuses to sign with any
/// other key. Rotating it later means shipping a new APK by hand: there is no in-band rotation.
pub const RELEASE_PUBKEY_HEX: &str = "b754ca04fd9c09d17b8e12394ef67544475025bc446ac0e55267588c231288a8";
/// This app's own package: the one installer's "owner" while Stampede updates itself.
pub const OWN_PACKAGE: &str = "org.decented.stampede";
/// The name used in sentences ("Allow installs from Stampede ...").
pub const APP_NAME: &str = "Stampede";
/// The file next to `profile.json`: last check time and the best verified release event.
pub const STATE_FILE: &str = "update.json";

/// The kit's `Config` for this app. `state_dir`: where `update.json` lives (next to
/// `profile.json`); `None` keeps the state in memory only (screenshots and tests: their
/// staged offers are throwaway-signed and must never reach a file).
pub fn update_config(state_dir: Option<&Path>) -> Config {
    // The versionCode is derived from the workspace version (build-apk.sh stamps the same formula);
    // `the_config_the_engine_builds_is_stampedes_own` fails the build if it ever has none.
    let mut config = Config::new(RELEASE_CHANNEL, OWN_PACKAGE, APP_NAME, env!("CARGO_PKG_VERSION"))
        .expect("the workspace version has a versionCode and the constants are valid")
        .with_release_pubkey_hex(RELEASE_PUBKEY_HEX)
        .expect("RELEASE_PUBKEY_HEX is a valid x-only key");
    config.relays = RELEASE_RELAYS.iter().map(|s| s.to_string()).collect();
    // "v0.4.14 . <build>" on the Settings card.
    config.version_label = Some(crate::version_label());
    match state_dir {
        Some(dir) => config.with_state_file(dir.join(STATE_FILE)),
        None => config,
    }
}

/// The in-app updater, reading the relays through [`AppFetcher`].
pub fn updater(state_dir: Option<&Path>) -> Updater {
    Updater::new(update_config(state_dir), Box::new(AppFetcher))
}

/// The kit's `CatalogConfig` for More games: [`CATALOG_ENTRIES`] and the release relays.
pub fn catalog_config() -> CatalogConfig {
    let mut config = CatalogConfig::new(CATALOG_ENTRIES.to_vec());
    config.relays = RELEASE_RELAYS.iter().map(|s| s.to_string()).collect();
    config
}

/// The More games screen's data, reading the relays through [`AppFetcher`].
pub fn catalog() -> Catalog {
    Catalog::new(catalog_config(), Box::new(AppFetcher))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::util::JsonUtil;
    use nostr::{Event, PublicKey};
    use nostr_release_kit::{parse_release, version_code_of, Verdict};

    /// Stampede's signing certificate, as the publisher pins it (`tools/release/release.json`).
    const EXPECTED_CERT: &str = "eacd24505a50630e2a115226fe592518bc16cc2421cfc019372b58baeaf08aa9";
    /// Twang Thang's release key: not Stampede's, so it must never be pinned as the release key here.
    const TWANG_KEY: &str = "2cc65a54774d9c1d6fde7e98255dbe1064940facb10dbc3e49f63300a27178c7";

    #[test]
    fn the_shipped_constants_are_stampedes_own() {
        assert_eq!(RELEASE_CHANNEL, "stampede-android-apk");
        assert_eq!(RELEASE_RELAYS, ["wss://relay.damus.io", "wss://relay.ditto.pub", "wss://relay.primal.net"]);
        assert_eq!(OWN_PACKAGE, "org.decented.stampede");
        assert_eq!(APP_NAME, "Stampede");
        assert_eq!(RELEASE_PUBKEY_HEX, "b754ca04fd9c09d17b8e12394ef67544475025bc446ac0e55267588c231288a8");
        // Twang Thang's release key is not Stampede's: it must never be pinned here.
        assert_ne!(RELEASE_PUBKEY_HEX, TWANG_KEY);
        assert!(CATALOG_ENTRIES.iter().all(|e| e.release_pubkey != Some(RELEASE_PUBKEY_HEX) && e.channel != RELEASE_CHANNEL));
        // The operator's own relay is never a default.
        assert!(RELEASE_RELAYS.iter().all(|r| !r.contains("trotters")));
    }

    #[test]
    fn the_pinned_key_is_a_real_key() {
        assert!(PublicKey::from_hex(RELEASE_PUBKEY_HEX).is_ok());
        assert!(RELEASE_PUBKEY_HEX.bytes().any(|b| b != b'0'));
    }

    #[test]
    fn the_config_the_engine_builds_is_stampedes_own() {
        let dir = std::path::Path::new("some/dir");
        let c = update_config(Some(dir));
        assert_eq!((c.channel.as_str(), c.own_package.as_str(), c.app_name.as_str()), (RELEASE_CHANNEL, OWN_PACKAGE, APP_NAME));
        assert_eq!(c.release_pubkey, Some(PublicKey::from_hex(RELEASE_PUBKEY_HEX).unwrap()));
        assert_ne!(c.release_pubkey, Some(PublicKey::from_hex(TWANG_KEY).unwrap()));
        assert_eq!(c.relays, RELEASE_RELAYS.map(String::from).to_vec());
        assert_eq!(c.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(c.version_label, Some(crate::version_label()));
        assert_eq!(c.check_interval, std::time::Duration::from_secs(6 * 3600));
        assert_eq!(c.fetch_timeout, std::time::Duration::from_secs(8));
        assert_eq!(c.state_file, Some(dir.join("update.json")));
        assert_eq!(update_config(None).state_file, None, "screenshots and tests keep nothing on disk");
        // The installed versionCode is the workspace version's, as build-apk.sh stamps it.
        assert_eq!(updater(None).installed_code(), version_code_of(env!("CARGO_PKG_VERSION")).unwrap());
        assert_eq!(c.version_code, updater(None).installed_code());
        assert!(updater(None).installed_code() > 0 && updater(None).configured());
    }

    #[test]
    fn the_catalog_config_carries_the_entries_and_the_release_relays() {
        let c = catalog_config();
        assert_eq!(c.entries, CATALOG_ENTRIES.to_vec());
        assert_eq!(c.relays, RELEASE_RELAYS.map(String::from).to_vec());
        let cat = catalog();
        assert_eq!(cat.entries().len(), 3);
        // Every entry names a real package that is not this app's own, and none repeats.
        assert_eq!(c.validate(OWN_PACKAGE), Ok(()));
        let filters = cat.filters();
        assert_eq!(filters.len(), 3, "one filter per ready entry");
        let json = filters.iter().map(|f| f.as_json()).collect::<Vec<_>>().join(" ");
        assert!(json.contains(TWANG_KEY), "{json}");
        assert!(json.contains(CATALOG_ENTRIES[1].release_pubkey.unwrap()), "{json}");
        assert!(json.contains(CATALOG_ENTRIES[2].release_pubkey.unwrap()), "{json}");
        assert!(json.contains("30063") && json.contains("twangthang-android-apk") && json.contains("gameflock-android-apk"), "{json}");
        assert!(json.contains("axenstax-android-apk"), "{json}");
        assert!(!json.contains("stampede-android-apk"), "{json}");
    }

    /// `tools/release/release.json` configures the kit's publisher; it and the Rust constants
    /// must say the same thing, or the publisher would sign what the app does not trust.
    #[test]
    fn the_publisher_config_matches_the_apps_constants() {
        let json: serde_json::Value = serde_json::from_str(include_str!("../../tools/release/release.json")).unwrap();
        let c = update_config(None);
        assert_eq!(json["channel"], c.channel.as_str());
        assert_eq!(json["package"], c.own_package.as_str());
        assert_eq!(json["release_pubkey"], c.release_pubkey.unwrap().to_hex().as_str());
        assert_eq!(json["relays"], serde_json::json!(c.relays));
        assert_eq!(json["expected_cert_sha256"], EXPECTED_CERT);
        assert_eq!(json["mirrors"], serde_json::json!(["https://nostr.download"]));
        assert_eq!(json["version_source"], "semver", "the versionCode is derived the way build-apk.sh derives it");
        // The key FILE is never read here: only the names the publisher looks it up by.
        assert_eq!(json["key_file_env"], "STAMPEDE_RELEASE_KEY_FILE");
        assert!(!json.to_string().contains("trotters"));
    }

    /// The Node publisher's own output (before the kit), signed with a THROWAWAY key whose secret
    /// was discarded, on this app's real channel: it is what a Stampede release event looks like to
    /// the app. Never commit an event signed by the real release key (it would be a publishable
    /// announcement).
    const EVENT: &str = r#"{"kind":30063,"created_at":1791886220,"tags":[["d","stampede-android-apk"],["version","0.1.0"],["version_code","1000"],["x","3ff9a9900fdf6c4e4227f9cd7874b97fc78a4f7c42796a32199fa994eec87b21"],["size","10021738"],["cert","eacd24505a50630e2a115226fe592518bc16cc2421cfc019372b58baeaf08aa9"],["url","https://nostr.download/3ff9a9900fdf6c4e4227f9cd7874b97fc78a4f7c42796a32199fa994eec87b21.apk"]],"content":"First in-app update release.","pubkey":"8835941fd71d7197381916c90e3ee71bcba254259ef28f7d95828fe26e9a05f2","id":"e06bd01082aa35fd96ac9af607c305f736377c206706800fb9981fa7249cb18f","sig":"69e1f0c0d0a004417bb070c38be377ed3d093f2fe60dd37972c4b43a7856ac8187ff3b42d9370417b9d2af7a9b80435902868d0abe4a8755d7e7146b7eb22608"}"#;
    const TEST_PUBKEY: &str = "8835941fd71d7197381916c90e3ee71bcba254259ef28f7d95828fe26e9a05f2";

    #[test]
    fn an_event_on_stampedes_channel_is_judged_through_the_real_config() {
        let ev: Event = serde_json::from_str(EVENT).unwrap();
        let test_key = PublicKey::from_hex(TEST_PUBKEY).unwrap();
        // The app's real channel accepts what the publisher signs, for the key it was signed by...
        let r = parse_release(&ev, &test_key, RELEASE_CHANNEL).expect("the app accepts what the publisher signs");
        assert_eq!((r.version.as_str(), r.version_code, r.size), ("0.1.0", 1000, 10_021_738));
        assert_eq!(r.notes, "First in-app update release.");
        // ...and the production key does not trust it: nothing is offered, nothing remembered.
        let mut prod = updater(None);
        prod.set_installed(500);
        prod.apply_events(std::slice::from_ref(&ev), 5);
        assert!(matches!(prod.verdict(), Verdict::Unknown) && prod.pill().is_none());
        assert!(prod.due(5), "nothing was recorded");
        // With the throwaway key pinned in the otherwise real Config, the same event is an offer for
        // an older install, and not one for an install that already has it.
        let mut cfg = update_config(None);
        cfg.release_pubkey = Some(test_key);
        let mut u = Updater::new(cfg.clone(), Box::new(AppFetcher));
        u.set_installed(500);
        u.apply_events(std::slice::from_ref(&ev), 5);
        assert!(matches!(u.verdict(), Verdict::Offer(o) if o.version_code == 1000));
        assert_eq!(u.pill().as_deref(), Some("Update to v0.1.0"));
        let mut current = Updater::new(cfg, Box::new(AppFetcher));
        current.set_installed(1000);
        current.apply_events(std::slice::from_ref(&ev), 5);
        assert!(matches!(current.verdict(), Verdict::UpToDate));
    }

    /// Twang Thang's own publisher signed this with a THROWAWAY key whose secret was discarded,
    /// on its real channel and carrying its real pinned certificate.
    const TWANG_EVENT: &str = r#"{"kind":30063,"created_at":1791721775,"tags":[["d","twangthang-android-apk"],["version","0.1.0"],["version_code","1000"],["x","2a6f9fe9a8233340827bb1851aeb02fee659c7060fa97d27a7352aa16763016a"],["size","9796461"],["cert","ad02f430148557fb6501c97b16a9ef60681d12d533f7579035e6d62a83dec640"],["url","https://nostr.download/2a6f9fe9a8233340827bb1851aeb02fee659c7060fa97d27a7352aa16763016a.apk"]],"content":"First in-app update release.","pubkey":"44e49e42cde28b3a1cb5d2b43fe7f3d3a2fd36bb977d1319c7d05a34f497c878","id":"e8867c4aa8ced84f3c676f4dea1636eaac658ee1682c160e82c02fb6ecdebb50","sig":"f9d4e55bac1bf7ef6258be3df0dbb4a67fdbea7f50cf3fbacd4ea2eec6defd0a39694c0c9568aeb7a9dd76d70bd72ff93ddd68c5ee6278483f859e4c4e0c453a"}"#;
    const TWANG_TEST_PUBKEY: &str = "44e49e42cde28b3a1cb5d2b43fe7f3d3a2fd36bb977d1319c7d05a34f497c878";

    #[test]
    fn a_twang_release_event_is_judged_through_the_real_catalog() {
        use nostr_release_kit::{EntryStatus, FetchReport};
        let ev: Event = serde_json::from_str(TWANG_EVENT).unwrap();
        let report = FetchReport { events: vec![ev], answered: 1, total: 3 };
        // The shipped catalog pins other keys: nothing is offered, for Twang Thang or Cluster Flock.
        let mut prod = catalog();
        prod.apply_report(&report);
        assert_eq!(prod.status(0), EntryStatus::NoRelease);
        assert_eq!(prod.status(1), EntryStatus::NoRelease);
        // With the throwaway key pinned in the otherwise real Twang Thang entry, the same event is a Get.
        let mut cfg = catalog_config();
        cfg.entries[0].release_pubkey = Some(TWANG_TEST_PUBKEY);
        let mut pinned = Catalog::new(cfg, Box::new(AppFetcher));
        pinned.apply_report(&report);
        assert_eq!(pinned.status(0), EntryStatus::Get { version: "0.1.0".into(), size: 9_796_461 });
        assert_eq!(pinned.status(1), EntryStatus::NoRelease);
    }

    #[test]
    fn the_permission_line_names_stampede() {
        use nostr_release_kit::ApkStatus;
        let ev: Event = serde_json::from_str(EVENT).unwrap();
        let mut cfg = update_config(None);
        cfg.release_pubkey = Some(PublicKey::from_hex(TEST_PUBKEY).unwrap());
        let mut u = Updater::new(cfg, Box::new(AppFetcher));
        u.set_installed(500);
        u.apply_events(&[ev], 5);
        u.demo_install_state(ApkStatus::NeedsPermission);
        let v = u.view(6);
        assert_eq!(v.status, "Allow installs from Stampede in the settings page that opened, then tap Update again");
        assert_eq!(v.installed, crate::version_label());
    }

    #[test]
    fn a_staged_offer_never_reaches_the_state_file() {
        let dir = std::env::temp_dir().join(format!("stampede-release-test-{}", std::process::id()));
        let mut file_backed = updater(Some(&dir));
        file_backed.demo_offer("0.3.0", "x");
        assert!(matches!(file_backed.verdict(), Verdict::Unknown), "a file-backed updater refuses a throwaway-signed offer");
        assert!(!dir.join(STATE_FILE).exists());
        let mut shots = updater(None);
        shots.demo_offer("0.3.0", "Faster starts.");
        assert!(matches!(shots.verdict(), Verdict::Offer(_)));
    }
}
