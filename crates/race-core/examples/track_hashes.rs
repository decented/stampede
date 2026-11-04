//! Printable cross-target geometry comparisons. Run with:
//! `cargo run --locked -p race-core --example track_hashes`.
//! These hashes are diagnostics, not a storage or network format.

use race_core::custom::{auto_fill, custom_track_from, shape};
use race_core::math::forward;
use race_core::{track_hash, Track, TrackId, Vec2};

fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn main() {
    for id in TrackId::ALL {
        println!("builtin:{id:?} geometry={:016x}", track_hash(id));
    }
    // The same fixed strokes and seeds on every target. The eight exercises
    // crossing-angle acos; non-flat tracks exercise the hill harmonics' powf.
    for (name, eight, hills, seed) in [
        ("flat-loop", false, 0, 7),
        ("hilly-loop", false, 3, 41),
        ("hilly-eight", true, 2, 93),
    ] {
        let stroke: Vec<Vec2> = (0..=500)
            .map(|i| {
                let t = std::f32::consts::TAU * i as f32 / 500.0;
                let direction = forward(t);
                if eight {
                    Vec2::new(170.0 * direction.x, 110.0 * direction.x * direction.z)
                } else {
                    Vec2::new(140.0 * direction.z, 100.0 * direction.x)
                }
            })
            .collect();
        let shaped = shape(&stroke, false, 18.0).expect("fixed custom stroke");
        assert_eq!(shaped.crossings.len(), usize::from(eight));
        let mut saved = custom_track_from(
            &shaped,
            name,
            TrackId::Savanna,
            2,
            180,
            auto_fill(&shaped, seed),
            seed,
        );
        saved.hills = hills;
        saved.validate().expect("fixed custom track");
        let def = saved.to_def().expect("fixed custom definition");
        let built = Track::build(&def);
        println!(
            "custom:{name} saved={:016x} definition={:016x} built={:016x}",
            hash(&saved.to_bytes()),
            hash(&bincode::serialize(&def).unwrap()),
            hash(&bincode::serialize(&built).unwrap()),
        );
    }
}
