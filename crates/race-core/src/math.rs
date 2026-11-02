//! Coordinate conventions and small helpers (spec section 3).
//!
//! Metres, seconds, radians. Right-handed, Y up. Yaw 0 faces +Z.
//! `forward = (sin yaw, 0, cos yaw)`, `right = forward x up = (-cos yaw, 0, sin yaw)`.
//! Positive steer turns right, which decreases yaw.
//!
//! Every transcendental function the simulation uses goes through this module,
//! and nowhere else: `sin`, `cos`, `tan`, `atan2`, `exp`, `powf`, `asin` and
//! `acos` below are the only ones, and each runs the same pure-Rust algorithm
//! (the ports in the submodules, or the pinned `libm` crate) on every target, so
//! all targets compute identical bits. Never call the `f32` methods
//! (`.sin()`, `.exp()`, `.powf()`, ...) in simulation code: they use the
//! toolchain's or the platform's maths library and differ between targets. A
//! source scan in this module's tests fails the build if one appears.

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

    const FORBIDDEN: &[&str] = &[
        "sin", "cos", "tan", "atan2", "asin", "acos", "atan", "exp", "exp2", "ln", "log", "log2",
        "log10", "powf", "hypot", "cbrt", "sin_cos", "mul_add", "sinh", "cosh", "tanh", "asinh",
        "acosh", "atanh", "exp_m1", "ln_1p",
    ];

    /// Files (relative to `crates/`) whose test-only code may call the platform
    /// maths: it drives inputs into the sim or builds sample data and never
    /// reaches simulation state. In these files only the text from the first
    /// `#[cfg(test)]` line onward is exempt.
    const TEST_TAIL_ALLOWED: &[&str] = &[
        // Prediction test drivers feeding steering inputs (lines in `#[cfg(test)]` modules).
        "race-session/src/client.rs",
        // `#[cfg(test)]` sample-track helper.
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

    /// Lines of `text` that call a forbidden method or `f32::`/`f64::` path
    /// function directly. Line comments are ignored.
    fn violations(text: &str) -> Vec<(usize, String)> {
        let mut found = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for name in FORBIDDEN {
                let method = format!(".{name}(");
                let f32_path = format!("f32::{name}(");
                let f64_path = format!("f64::{name}(");
                if code.contains(&method) || code.contains(&f32_path) || code.contains(&f64_path) {
                    found.push((n + 1, line.trim().to_string()));
                    break;
                }
            }
        }
        found
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
                if TEST_TAIL_ALLOWED.contains(&rel.as_str()) {
                    if let Some(at) = text.find("#[cfg(test)]") {
                        text.truncate(at);
                    }
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
        assert!(violations("let a = x.sqrt() + x.powi(2);").is_empty());
    }
}
