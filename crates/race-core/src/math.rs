//! Coordinate conventions and small helpers (spec section 3).
//!
//! Metres, seconds, radians. Right-handed, Y up. Yaw 0 faces +Z.
//! `forward = (sin yaw, 0, cos yaw)`, `right = forward x up = (-cos yaw, 0, sin yaw)`.
//! Positive steer turns right, which decreases yaw.
//!
//! The eight transcendental functions the simulation uses (`sin`, `cos`, `tan`,
//! `atan2`, `exp`, `powf`, `asin` and `acos`) are defined below and nowhere
//! else. Each runs the same pure-Rust algorithm (the ports in the submodules, or
//! the pinned `libm` crate) on every target, so these eight compute identical
//! bits everywhere. Never call the `f32` methods (`.sin()`, `.exp()`,
//! `.powf()`, ...) in simulation code: they use the toolchain's or the
//! platform's maths library and can differ between targets. A source scan in
//! this module's tests fails the build if one appears.
//!
//! That is all this module claims; it is not full float determinism across
//! targets. `f32::min` and `f32::max` of opposite zeros (`+0.0` and `-0.0`)
//! return an unspecified one of them, which may differ between targets, and
//! nothing here covers that. `%` and `rem_euclid` still call the platform's
//! `fmod`, which is exact by definition and so is expected to agree, but is not
//! routed through this module.

use glam::Vec3;

/// Simulation rate in ticks per second.
pub const TICK_HZ: u32 = 60;
/// Fixed simulation step in seconds.
pub const DT: f32 = 1.0 / TICK_HZ as f32;

mod angle;
mod sine;
mod tangent;

/// Sine of an angle in radians (Arm Optimized Routines port).
#[inline]
pub(crate) fn sin(value: f32) -> f32 {
    sine::sin(value)
}

/// Cosine of an angle in radians (Arm Optimized Routines port).
#[inline]
pub(crate) fn cos(value: f32) -> f32 {
    sine::cos(value)
}

/// Tangent of an angle in radians.
#[inline]
pub(crate) fn tan(value: f32) -> f32 {
    tangent::tan(value)
}

/// Four-quadrant arctangent of `y / x`.
#[inline]
pub(crate) fn atan2(y: f32, x: f32) -> f32 {
    angle::atan2(y, x)
}

/// `e^x`, from the pinned `libm` crate.
#[inline]
pub(crate) fn exp(value: f32) -> f32 {
    libm::expf(value)
}

/// `base^exponent`, from the pinned `libm` crate.
#[inline]
pub(crate) fn powf(base: f32, exponent: f32) -> f32 {
    libm::powf(base, exponent)
}

/// Arcsine, from the pinned `libm` crate.
#[inline]
pub(crate) fn asin(value: f32) -> f32 {
    libm::asinf(value)
}

/// Arccosine, from the pinned `libm` crate.
#[inline]
pub(crate) fn acos(value: f32) -> f32 {
    libm::acosf(value)
}

/// Unit forward vector (horizontal) for a yaw.
#[inline]
pub fn forward(yaw: f32) -> Vec3 {
    Vec3::new(sin(yaw), 0.0, cos(yaw))
}

/// Unit right vector (horizontal) for a yaw: `forward x up`.
#[inline]
pub fn right(yaw: f32) -> Vec3 {
    Vec3::new(-cos(yaw), 0.0, sin(yaw))
}

/// Yaw of a (horizontal) direction; inverse of [`forward`].
#[inline]
pub fn yaw_of(dir: Vec3) -> f32 {
    atan2(dir.x, dir.z)
}

/// Wraps an angle into (-PI, PI].
#[inline]
pub fn wrap_angle(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut a = a % tau;
    if a > std::f32::consts::PI {
        a -= tau;
    } else if a <= -std::f32::consts::PI {
        a += tau;
    }
    a
}

/// Moves `cur` toward `target` by at most `step`.
#[inline]
pub fn approach(cur: f32, target: f32, step: f32) -> f32 {
    if cur < target {
        (cur + step).min(target)
    } else {
        (cur - step).max(target)
    }
}

/// Linear interpolation.
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Hermite smoothstep of `x` between `e0` and `e1`.
#[inline]
pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Horizontal (XZ) part of a vector.
#[inline]
pub fn flat(v: Vec3) -> Vec3 {
    Vec3::new(v.x, 0.0, v.z)
}

/// A 64-bit mix (splitmix64) used to derive deterministic seeds.
#[inline]
pub fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Golden bit vectors for the eight shared helpers. They were generated once
    // from the helpers themselves (inputs below are exact f32 bit patterns) and
    // detect any change in these routines, their ports, the pinned `libm`
    // version or the toolchain's codegen for them. They record what the code
    // does, not that it is correct. Rows are (input bits, expected bits), or
    // (a bits, b bits, expected bits) for two-argument helpers.
    const SIN: &[(u32, u32)] = &[
        (0x00000000, 0x00000000),
        (0x80000000, 0x80000000),
        (0x0da24260, 0x0da24260),
        (0x8da24260, 0x8da24260),
        (0x000116c2, 0x000116c2),
        (0x800116c2, 0x800116c2),
        (0x3a83126f, 0x3a83126e),
        (0x3dcccccd, 0x3dcc7577),
        (0xbdcccccd, 0xbdcc7577),
        (0x3f000000, 0x3ef57744),
        (0x3f800000, 0x3f576aa4),
        (0xbf800000, 0xbf576aa4),
        (0x40000000, 0x3f68c7b7),
        (0x3f060a92, 0x3f000000),
        (0x3f490fdb, 0x3f3504f3),
        (0x3fc90fdb, 0x3f800000),
        (0x40490fdb, 0xb3bbbd2e),
        (0xc0490fdb, 0x33bbbd2e),
        (0x4096cbe4, 0xbf800000),
        (0x40c90fdb, 0x343bbd2e),
        (0x40400000, 0x3e1081c3),
        (0x40800000, 0xbf41bdcf),
        (0x41200000, 0xbf0b44f8),
        (0xc1200000, 0x3f0b44f8),
        (0x42c80000, 0xbf01a12e),
        (0x447a0000, 0x3f53ae61),
        (0x47f12000, 0xbf3d8337),
        (0x49742400, 0xbeb33259),
        (0x501502f9, 0xbef99a64),
        (0xd01502f9, 0x3ef99a64),
        (0x60ad78ec, 0x3f281569),
        (0x7f61b1e6, 0x3f5ff9c4),
        (0x7f7fffff, 0xbf0599b3),
        (0x00800000, 0x00800000),
        (0x3fc90fdb, 0x3f800000),
        (0x3fc90fda, 0x3f800000),
        (0x3f490fdb, 0x3f3504f3),
        (0x3c8efa34, 0x3c8ef858),
        (0x40c90fdb, 0x343bbd2e),
        (0x41fb53d1, 0xb5155386),
    ];
    const COS: &[(u32, u32)] = &[
        (0x00000000, 0x3f800000),
        (0x80000000, 0x3f800000),
        (0x0da24260, 0x3f800000),
        (0x8da24260, 0x3f800000),
        (0x000116c2, 0x3f800000),
        (0x800116c2, 0x3f800000),
        (0x3a83126f, 0x3f7ffff8),
        (0x3dcccccd, 0x3f7eb898),
        (0xbdcccccd, 0x3f7eb898),
        (0x3f000000, 0x3f60a940),
        (0x3f800000, 0x3f0a5140),
        (0xbf800000, 0x3f0a5140),
        (0x40000000, 0xbed51133),
        (0x3f060a92, 0x3f5db3d7),
        (0x3f490fdb, 0x3f3504f3),
        (0x3fc90fdb, 0xb33bbd2e),
        (0x40490fdb, 0xbf800000),
        (0xc0490fdb, 0xbf800000),
        (0x4096cbe4, 0x324cde2e),
        (0x40c90fdb, 0x3f800000),
        (0x40400000, 0xbf7d7026),
        (0x40800000, 0xbf275530),
        (0x41200000, 0xbf56cd64),
        (0xc1200000, 0xbf56cd64),
        (0x42c80000, 0x3f5cc0ee),
        (0x447a0000, 0x3f0ff813),
        (0x47f12000, 0xbf2c1b84),
        (0x49742400, 0x3f6fcefd),
        (0x501502f9, 0x3f5f84c5),
        (0xd01502f9, 0x3f5f84c5),
        (0x60ad78ec, 0x3f411723),
        (0x7f61b1e6, 0xbef7f57c),
        (0x7f7fffff, 0x3f5a5f96),
        (0x00800000, 0x3f800000),
        (0x3fc90fdb, 0xb33bbd2e),
        (0x3fc90fda, 0x33a22169),
        (0x3f490fdb, 0x3f3504f3),
        (0x3c8efa34, 0x3f7ff605),
        (0x40c90fdb, 0x3f800000),
        (0x41fb53d1, 0x3f800000),
    ];
    const TAN: &[(u32, u32)] = &[
        (0x00000000, 0x00000000),
        (0x80000000, 0x80000000),
        (0x0da24260, 0x0da24260),
        (0x8da24260, 0x8da24260),
        (0x000116c2, 0x000116c2),
        (0x800116c2, 0x800116c2),
        (0x3a83126f, 0x3a831272),
        (0x3dcccccd, 0x3dcd7c44),
        (0xbdcccccd, 0xbdcd7c44),
        (0x3f000000, 0x3f0bda7b),
        (0x3f800000, 0x3fc75923),
        (0xbf800000, 0xbfc75923),
        (0x40000000, 0xc00bd7b1),
        (0x3f060a92, 0x3f13cd3b),
        (0x3f490fdb, 0x3f800000),
        (0x3fc90fdb, 0xcbae8a4b),
        (0x40490fdb, 0x33bbbd2e),
        (0xc0490fdb, 0xb3bbbd2e),
        (0x4096cbe4, 0xcc9ff26d),
        (0x40c90fdb, 0x343bbd2e),
        (0x40400000, 0xbe11f7b9),
        (0x40800000, 0x3f94337d),
        (0x41200000, 0x3f25fafa),
        (0xc1200000, 0xbf25fafa),
        (0x42c80000, 0xbf1653a7),
        (0x447a0000, 0x3fbc3395),
        (0x47f12000, 0x3f8cf1cc),
        (0x49742400, 0xbebf4bb4),
        (0x501502f9, 0xbf0ef000),
        (0xd01502f9, 0x3f0ef000),
        (0x60ad78ec, 0x3f5ed890),
        (0x7f61b1e6, 0xbfe73d2d),
        (0x7f7fffff, 0xbf1c9eca),
        (0x00800000, 0x00800000),
        (0x3fc90fdb, 0xcbae8a4b),
        (0x3fc90fda, 0x4b4a1bd9),
        (0x3f490fdb, 0x3f800000),
        (0x3c8efa34, 0x3c8efdec),
        (0x40c90fdb, 0x343bbd2e),
        (0x41fb53d1, 0xb5155386),
    ];
    const EXP: &[(u32, u32)] = &[
        (0x00000000, 0x3f800000),
        (0x80000000, 0x3f800000),
        (0x3f800000, 0x402df854),
        (0xbf800000, 0x3ebc5ab2),
        (0x3f000000, 0x3fd3094c),
        (0xbf000000, 0x3f1b4598),
        (0x40000000, 0x40ec7326),
        (0x41200000, 0x46ac14ee),
        (0xc1200000, 0x383e6bce),
        (0xbd4ccccd, 0x3f7383c6),
        (0xbc88888a, 0x3f7bc4c9),
        (0xbd638e3a, 0x3f722a66),
        (0xbd088880, 0x3f779b7a),
        (0xbd4cccce, 0x3f7383c5),
        (0x0da24260, 0x3f800000),
        (0x8da24260, 0x3f800000),
        (0x322bcc77, 0x3f800000),
        (0x41a00000, 0x4de75844),
        (0x42a00000, 0x792abbce),
        (0x42b00000, 0x7ef882b7),
        (0x42b17213, 0x7f7ffd84),
        (0x42b17218, 0x7f800000),
        (0x42b20000, 0x7f800000),
        (0x42c80000, 0x7f800000),
        (0xc2a00000, 0x05bfecba),
        (0xc2ae0000, 0x00b33687),
        (0xc2b00000, 0x0041edc4),
        (0xc2c80000, 0x0000001b),
        (0xc2ce0000, 0x00000001),
        (0xc2d00000, 0x00000000),
        (0xc47a0000, 0x00000000),
        (0x7f800000, 0x7f800000),
        (0xff800000, 0x00000000),
    ];
    const ASIN: &[(u32, u32)] = &[
        (0x00000000, 0x00000000),
        (0x80000000, 0x80000000),
        (0x0da24260, 0x0da24260),
        (0x8da24260, 0x8da24260),
        (0x000116c2, 0x000116c2),
        (0x38d1b717, 0x38d1b717),
        (0xb8d1b717, 0xb8d1b717),
        (0x3dcccccd, 0x3dcd2494),
        (0xbdcccccd, 0xbdcd2494),
        (0x3e800000, 0x3e815f4e),
        (0x3f000000, 0x3f060a92),
        (0xbf000000, 0xbf060a92),
        (0x3f3504f3, 0x3f490fda),
        (0xbf3504f3, 0xbf490fda),
        (0x3f666666, 0x3f8f549b),
        (0xbf666666, 0xbf8f549b),
        (0x3f7d70a4, 0x3fb6f1e4),
        (0x3f7fbe77, 0x3fc35650),
        (0x3f7fffff, 0x3fc9048a),
        (0xbf7fffff, 0xbfc9048a),
        (0x3f800000, 0x3fc90fdb),
        (0xbf800000, 0xbfc90fdb),
        (0x3eaaaaaa, 0x3eadff1a),
        (0x3f2aaaab, 0x3f3acf6f),
        (0x3f5db3d7, 0x3f860a91),
        (0x3d800000, 0x3d80155f),
        (0x3f7ff972, 0x3fc74067),
        (0x3f060a92, 0x3f0d12e6),
        (0x38d1b717, 0x38d1b717),
    ];
    const ACOS: &[(u32, u32)] = &[
        (0x00000000, 0x3fc90fda),
        (0x80000000, 0x3fc90fda),
        (0x0da24260, 0x3fc90fda),
        (0x8da24260, 0x3fc90fda),
        (0x000116c2, 0x3fc90fda),
        (0x38d1b717, 0x3fc90c94),
        (0xb8d1b717, 0x3fc91321),
        (0x3dcccccd, 0x3fbc3d91),
        (0xbdcccccd, 0x3fd5e224),
        (0x3e800000, 0x3fa8b807),
        (0x3f000000, 0x3f860a92),
        (0xbf000000, 0x40060a92),
        (0x3f3504f3, 0x3f490fdb),
        (0xbf3504f3, 0x4016cbe4),
        (0x3f666666, 0x3ee6ecfe),
        (0xbf666666, 0x402c323b),
        (0x3f7d70a4, 0x3e10efb5),
        (0x3f7fbe77, 0x3d37315a),
        (0x3f7fffff, 0x39b504f3),
        (0xbf7fffff, 0x40490a32),
        (0x3f800000, 0x00000000),
        (0xbf800000, 0x40490fda),
        (0x3eaaaaaa, 0x3f9d9014),
        (0x3f2aaaab, 0x3f575046),
        (0x3f5db3d7, 0x3f060a92),
        (0x3d800000, 0x3fc10e85),
        (0x3f7ff972, 0x3c67b9d5),
        (0x3f060a92, 0x3f828668),
        (0x38d1b717, 0x3fc90c94),
    ];
    const POWF: &[(u32, u32, u32)] = &[
        (0x3f800000, 0x3f4ccccd, 0x3f800000),
        (0x40000000, 0x3f4ccccd, 0x3fdedc67),
        (0x40400000, 0x3f4ccccd, 0x401a205a),
        (0x40800000, 0x3f4ccccd, 0x40420300),
        (0x40a00000, 0x3f4ccccd, 0x4067edf4),
        (0x41000000, 0x3f4ccccd, 0x40a8e5a3),
        (0x41800000, 0x3f4ccccd, 0x4113088c),
        (0x3f000000, 0x3fc00000, 0x3eb504f3),
        (0x3e800000, 0x3fc00000, 0x3e000000),
        (0x3f666666, 0x3fc00000, 0x3f5a939e),
        (0x3dfcd680, 0x3fc00000, 0x3d31ad01),
        (0x3f7fffef, 0x3fc00000, 0x3f7fffe7),
        (0x358637bd, 0x3fc00000, 0x3089705f),
        (0x00000000, 0x3fc00000, 0x00000000),
        (0x80000000, 0x40400000, 0x80000000),
        (0x00000000, 0x00000000, 0x3f800000),
        (0x00000000, 0xbf800000, 0x7f800000),
        (0x40000000, 0x00000000, 0x3f800000),
        (0x40000000, 0xbf800000, 0x3f000000),
        (0x40000000, 0x41200000, 0x44800000),
        (0x40000000, 0x3f000000, 0x3fb504f3),
        (0x41200000, 0xc0400000, 0x3a83126f),
        (0x3f800000, 0x7149f2ca, 0x3f800000),
        (0xc0000000, 0x40400000, 0xc1000000),
        (0xc0000000, 0x40000000, 0x40800000),
        (0x3f333333, 0x3e4ccccd, 0x3f6e5fd7),
        (0x3f333333, 0x3c888889, 0x3f7e7b92),
        (0x3f59999a, 0x3daaaaa6, 0x3f7c8e6a),
        (0x3f733333, 0x3eaaaaaa, 0x3f7ba901),
        (0x3f000000, 0x42c80000, 0x0d800000),
        (0x3f000000, 0xc3480000, 0x7f800000),
        (0x7149f2ca, 0x40000000, 0x7f800000),
        (0x7f61b1e6, 0x3f000000, 0x5f705ece),
        (0x7f800000, 0x3f000000, 0x7f800000),
        (0x7f800000, 0xbf000000, 0x00000000),
        (0x00800000, 0x3f000000, 0x20000000),
        (0x3f800001, 0x4b189680, 0x4052d05e),
        (0x40a00000, 0x40a00000, 0x45435000),
        (0x42c80000, 0x41200000, 0x60ad78ec),
        (0x2edbe6ff, 0x406ccccd, 0x02081ce1),
    ];
    const ATAN2: &[(u32, u32, u32)] = &[
        (0x00000000, 0x3f800000, 0x00000000),
        (0x80000000, 0x3f800000, 0x80000000),
        (0x00000000, 0xbf800000, 0x40490fdb),
        (0x80000000, 0xbf800000, 0xc0490fdb),
        (0x00000000, 0x00000000, 0x00000000),
        (0x80000000, 0x80000000, 0xc0490fdb),
        (0x3f800000, 0x00000000, 0x3fc90fdb),
        (0xbf800000, 0x00000000, 0xbfc90fdb),
        (0x3f800000, 0x3f800000, 0x3f490fdb),
        (0x3f800000, 0xbf800000, 0x4016cbe4),
        (0xbf800000, 0xbf800000, 0xc016cbe4),
        (0xbf800000, 0x3f800000, 0xbf490fdb),
        (0x3dcccccd, 0x40133333, 0x3d31f991),
        (0x40133333, 0x3dcccccd, 0x3fc3800e),
        (0xc0133333, 0x3dcccccd, 0xbfc3800e),
        (0x3a83126f, 0x447a0000, 0x358637bd),
        (0x447a0000, 0x3a83126f, 0x3fc90fd2),
        (0x0da24260, 0x3f800000, 0x0da24260),
        (0x0da24260, 0x0da24260, 0x3f490fdb),
        (0x7149f2ca, 0x0da24260, 0x3fc90fdb),
        (0xf149f2ca, 0xf149f2ca, 0xc016cbe4),
        (0x7f61b1e6, 0x7f61b1e6, 0x3f490fdb),
        (0x7f7fffff, 0x3f800000, 0x3fc90fdb),
        (0x3f800000, 0x7f7fffff, 0x00200000),
        (0x00800000, 0x00800000, 0x3f490fdb),
        (0x000116c2, 0x000116c2, 0x3f490fdb),
        (0x7f800000, 0x3f800000, 0x3fc90fdb),
        (0x3f800000, 0x7f800000, 0x00000000),
        (0x3f800000, 0xff800000, 0x40490fdb),
        (0x7f800000, 0x7f800000, 0x3f490fdb),
        (0xff800000, 0xff800000, 0xc016cbe4),
        (0x3f000000, 0xbf000000, 0x4016cbe4),
        (0x42f6e979, 0xc29dcccd, 0x4008ed05),
        (0xbe99999a, 0xbecccccd, 0xc01fe0bc),
        (0x3f19999a, 0x3f4ccccd, 0x3f24bc7d),
        (0xbf19999a, 0x3f4ccccd, 0xbf24bc7d),
    ];

    fn check1(name: &str, table: &[(u32, u32)], f: fn(f32) -> f32) {
        for &(input, expected) in table {
            let got = f(f32::from_bits(input)).to_bits();
            assert_eq!(got, expected, "{name}({input:08x}): got {got:08x}, expected {expected:08x}");
        }
    }

    fn check2(name: &str, table: &[(u32, u32, u32)], f: fn(f32, f32) -> f32) {
        for &(a, b, expected) in table {
            let got = f(f32::from_bits(a), f32::from_bits(b)).to_bits();
            assert_eq!(got, expected, "{name}({a:08x}, {b:08x}): got {got:08x}, expected {expected:08x}");
        }
    }

    #[test]
    fn golden_bits_sin_cos_tan() {
        check1("sin", SIN, sin);
        check1("cos", COS, cos);
        check1("tan", TAN, tan);
    }

    #[test]
    fn golden_bits_exp_asin_acos() {
        check1("exp", EXP, exp);
        check1("asin", ASIN, asin);
        check1("acos", ACOS, acos);
    }

    #[test]
    fn golden_bits_powf_atan2() {
        check2("powf", POWF, powf);
        check2("atan2", ATAN2, atan2);
    }

    #[test]
    fn domain_errors_are_nan() {
        assert!(sin(f32::INFINITY).is_nan() && cos(f32::NEG_INFINITY).is_nan());
        assert!(sin(f32::NAN).is_nan() && exp(f32::NAN).is_nan());
        assert!(asin(1.000_000_1).is_nan() && acos(-1.000_000_1).is_nan());
        assert!(powf(-2.0, 0.5).is_nan());
    }

    // ---- Bypass guard -----------------------------------------------------

    /// `f32` methods whose result comes from the toolchain's or the platform's
    /// maths library, or is not specified to be the same on every target
    /// (`powi`, `mul_add`, `sin_cos`).
    const FORBIDDEN: &[&str] = &[
        "sin", "cos", "tan", "atan2", "asin", "acos", "atan", "exp", "exp2", "ln", "log", "log2",
        "log10", "powf", "powi", "hypot", "cbrt", "sin_cos", "mul_add", "sinh", "cosh", "tanh",
        "asinh", "acosh", "atanh", "exp_m1", "ln_1p",
    ];

    /// glam helpers that call `sin`, `cos`, `atan2` or `acos` inside glam, which
    /// uses the platform's maths library.
    const FORBIDDEN_GLAM: &[&str] = &[
        "from_angle", "to_angle", "angle_to", "angle_between", "rotate_x", "rotate_y", "rotate_z",
        "slerp",
    ];

    /// Files (relative to `crates/`) that are scanned except for their
    /// `#[cfg(test)]` items (modules and functions, found by brace matching):
    /// that code drives inputs into the sim or builds sample data and never
    /// reaches simulation state. All other code in these files is scanned.
    const TEST_ITEMS_ALLOWED: &[&str] = &[
        // Prediction test drivers feeding steering inputs (`#[cfg(test)]` modules).
        "race-session/src/client.rs",
        // `#[cfg(test)]` sample-track helper and test module.
        "race-session/src/track_transfer.rs",
    ];
    /// Files that are wholly test code (`#[cfg(test)] mod tests;`).
    const TEST_ONLY_FILES: &[&str] = &["race-session/src/tests.rs"];

    fn rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    fn is_ident(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    /// True if `code` contains `needle` and the text after it does not continue
    /// an identifier (so `f32::sin` matches in `map(f32::sin)` but not `f32::sine`).
    fn has_word(code: &str, needle: &str) -> bool {
        code.match_indices(needle)
            .any(|(at, _)| !code[at + needle.len()..].chars().next().is_some_and(is_ident))
    }

    /// Lines of `text` that use a forbidden maths function: a method call
    /// (whatever the spacing), an `f32::`/`f64::`/`<f32>::` path (called or
    /// passed as a value), or one of the glam angle helpers. Line comments are
    /// ignored.
    fn violations(text: &str) -> Vec<(usize, String)> {
        let mut found = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            let compact: String = code.chars().filter(|c| !c.is_whitespace()).collect();
            let float_maths = FORBIDDEN.iter().any(|name| {
                compact.contains(&format!(".{name}("))
                    || ["f32::", "f64::", "f32>::", "f64>::"]
                        .iter()
                        .any(|path| has_word(&compact, &format!("{path}{name}")))
            });
            let glam_maths = FORBIDDEN_GLAM
                .iter()
                .any(|name| compact.contains(&format!(".{name}(")) || compact.contains(&format!("::{name}(")));
            if float_maths || glam_maths {
                found.push((n + 1, line.trim().to_string()));
            }
        }
        found
    }

    /// Index just past the item that begins at `from` (after an attribute): the
    /// matching `}` of its first `{ .. }`, or its `;` if that comes first.
    /// Comments, strings and character literals are skipped.
    fn item_end(b: &[u8], from: usize) -> usize {
        let (mut i, mut braces, mut parens, mut opened) = (from, 0usize, 0usize, false);
        while i < b.len() {
            match b[i] {
                b'/' if b.get(i + 1) == Some(&b'/') => {
                    while i < b.len() && b[i] != b'\n' {
                        i += 1;
                    }
                    continue;
                }
                b'/' if b.get(i + 1) == Some(&b'*') => {
                    let mut depth = 1;
                    i += 2;
                    while i < b.len() && depth > 0 {
                        if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
                            depth += 1;
                            i += 1;
                        } else if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                            depth -= 1;
                            i += 1;
                        }
                        i += 1;
                    }
                    continue;
                }
                b'r' if i == 0 || !is_ident(b[i - 1] as char) || b[i - 1] == b'b' => {
                    let hashes = b[i + 1..].iter().take_while(|&&c| c == b'#').count();
                    if b.get(i + 1 + hashes) == Some(&b'"') {
                        let mut close = vec![b'"'];
                        close.extend(std::iter::repeat(b'#').take(hashes));
                        i += 2 + hashes;
                        while i < b.len() && !b[i..].starts_with(&close) {
                            i += 1;
                        }
                        i += close.len();
                        continue;
                    }
                }
                b'"' => {
                    i += 1;
                    while i < b.len() && b[i] != b'"' {
                        i += if b[i] == b'\\' { 2 } else { 1 };
                    }
                }
                b'\'' => {
                    if b.get(i + 1) == Some(&b'\\') {
                        i += 3;
                        while i < b.len() && b[i] != b'\'' {
                            i += 1;
                        }
                    } else if b.get(i + 2) == Some(&b'\'') {
                        i += 2;
                    }
                }
                b'(' | b'[' => parens += 1,
                b')' | b']' => parens = parens.saturating_sub(1),
                b'{' => {
                    braces += 1;
                    opened = true;
                }
                b'}' => {
                    braces = braces.saturating_sub(1);
                    if opened && braces == 0 {
                        return i + 1;
                    }
                }
                b';' if !opened && parens == 0 => return i + 1,
                _ => {}
            }
            i += 1;
        }
        b.len()
    }

    /// `text` with every item marked `#[cfg(test)]` (module, function, `use`)
    /// blanked out. Line breaks stay, so line numbers still match the file.
    fn without_test_items(text: &str) -> String {
        let b = text.as_bytes();
        let mut out = b.to_vec();
        let mut line_start = 0;
        let mut blanked_to = 0;
        for line in text.split_inclusive('\n') {
            let at = line_start;
            line_start += line.len();
            let attr = "#[cfg(test)]";
            if at >= blanked_to && line.trim_start().starts_with(attr) {
                let from = at + line.len() - line.trim_start().len() + attr.len();
                let end = item_end(b, from);
                for byte in &mut out[at..end] {
                    if *byte != b'\n' {
                        *byte = b' ';
                    }
                }
                blanked_to = end;
            }
        }
        String::from_utf8(out).expect("blanking whole bytes inside ASCII delimiters keeps UTF-8")
    }

    fn scan_simulation_sources() -> Vec<String> {
        let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut report = Vec::new();
        for krate in ["race-core", "race-session", "race-authority"] {
            let mut files = Vec::new();
            rust_files(&crates.join(krate).join("src"), &mut files);
            files.sort();
            for file in files {
                let rel = file.strip_prefix(&crates).unwrap().to_string_lossy().replace('\\', "/");
                // The math module itself, and its ported routines, are the one place.
                if rel == "race-core/src/math.rs" || rel.starts_with("race-core/src/math/") {
                    continue;
                }
                if TEST_ONLY_FILES.contains(&rel.as_str()) || rel.starts_with("race-session/src/tests/") {
                    continue;
                }
                let mut text = std::fs::read_to_string(&file).unwrap();
                if TEST_ITEMS_ALLOWED.contains(&rel.as_str()) {
                    text = without_test_items(&text);
                }
                for (line, code) in violations(&text) {
                    report.push(format!("{rel}:{line}: {code}"));
                }
            }
        }
        report
    }

    #[test]
    fn simulation_code_never_calls_platform_maths_directly() {
        let report = scan_simulation_sources();
        assert!(
            report.is_empty(),
            "route these through race_core::math so every target computes identical bits:\n{}",
            report.join("\n")
        );
    }

    #[test]
    fn the_bypass_scan_recognises_direct_calls() {
        assert!(!violations("let a = x.sin();").is_empty());
        assert!(!violations("let a = (x * 2.0).powf(1.5);").is_empty());
        assert!(!violations("let a = f32::exp(x);").is_empty());
        assert!(!violations("let a = v.dot(w).acos().to_degrees();").is_empty());
        assert!(violations("let a = crate::math::sin(x); // not .sin() here").is_empty());
        assert!(violations("let a = x.sqrt() + x.hypot2();").is_empty());
        assert!(violations("let a = f32::EPSILON + f32::consts_like + f32::sine_table;").is_empty());
    }

    #[test]
    fn the_bypass_scan_recognises_the_less_obvious_forms() {
        // `powi` is not specified to give the same bits on every target.
        assert!(!violations("let a = x.powi(2);").is_empty());
        assert!(!violations("let a = f32::powi(x, 2);").is_empty());
        // Paths passed as values, qualified paths and odd spacing.
        assert!(!violations("let v: Vec<f32> = xs.iter().copied().map(f32::sin).collect();").is_empty());
        assert!(!violations("let a = <f32>::exp(x);").is_empty());
        assert!(!violations("let a = f64::from(x).powf(0.5);").is_empty());
        assert!(!violations("let a = x . sin ();").is_empty());
        assert!(!violations("    .atan2(y)").is_empty());
        // glam helpers that call platform maths inside glam.
        assert!(!violations("let a = Vec2::from_angle(yaw);").is_empty());
        assert!(!violations("let a = v.to_angle() + v.angle_to(w);").is_empty());
        assert!(!violations("let a = v.angle_between(w);").is_empty());
        assert!(!violations("let a = v.rotate_x(1.0).rotate_y(1.0).rotate_z(1.0);").is_empty());
        assert!(!violations("let a = q.slerp(r, t);").is_empty());
        // Our own helpers stay allowed, however they are qualified.
        assert!(violations("let a = math::sin(x) + crate::math::powf(x, 1.5);").is_empty());
        assert!(violations("let a = race_core::math::yaw_of(d);").is_empty());
    }

    #[test]
    fn test_items_are_blanked_by_brace_matching_and_nothing_else() {
        let text = concat!(
            "fn production() { let a = x.sin(); }\n",
            "#[cfg(test)]\n",
            "mod tests {\n",
            "    fn t<'a>(s: &'a str) { let _ = (s, '}', \"}{\", r#\"}\"#, '\\''); x.sin(); }\n",
            "    // }\n",
            "    /* } */\n",
            "}\n",
            "fn also_production() { let b = y.cos(); }\n",
            "#[cfg(test)]\n",
            "pub(crate) fn helper(n: [u8; 4]) -> f32 { { n[0] as f32 }.exp() }\n",
            "#[cfg(test)]\n",
            "use std::collections::{HashMap, HashSet};\n",
            "fn last() { z.tan(); }\n",
        );
        let kept = without_test_items(text);
        assert_eq!(kept.lines().count(), text.lines().count(), "line numbers must survive");
        let lines: Vec<usize> = violations(&kept).into_iter().map(|(n, _)| n).collect();
        assert_eq!(lines, vec![1, 8, 13], "production lines stay scanned, test items are blanked: {kept}");
        assert!(!violations(text).is_empty() && violations(text).len() > 3);
    }

    #[test]
    fn the_scan_still_reads_the_production_code_in_the_partly_test_files() {
        // A wrong brace match would silently hide code from the guard, so check
        // that known production code in each file survives the blanking.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../race-session/src");
        let client = without_test_items(&std::fs::read_to_string(root.join("client.rs")).unwrap());
        for marker in ["fn sanitize_own", "fn fx_events", "impl Race", "fn net_yaw"] {
            assert!(client.contains(marker), "client.rs: {marker} was blanked out of the scan");
        }
        for marker in ["mod rose_view_tests", "mod effect_replay_tests", "mod keep_pass_tests"] {
            assert!(!client.contains(marker), "client.rs: {marker} should be blanked");
        }
        let transfer = without_test_items(&std::fs::read_to_string(root.join("track_transfer.rs")).unwrap());
        assert!(!transfer.contains("fn sample_track") && !transfer.contains("mod tests"));
    }
}
