use bevy::prelude::*;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::config::{ART_TEXTURE_SIZE, ArtStyle};

// ─── RandomArt expression tree ───────────────────────────────────────────────

#[derive(Clone)]
enum Expr {
    X,
    Y,
    T,
    Num(f32),
    Add(Box<Expr>, Box<Expr>),
    Mult(Box<Expr>, Box<Expr>),
    Sqrt(Box<Expr>),
    Abs(Box<Expr>),
    Sin(Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Mix(Box<Expr>, Box<Expr>, Box<Expr>),
}

impl Expr {
    fn eval(&self, x: f32, y: f32, t: f32) -> f32 {
        match self {
            Expr::X => x,
            Expr::Y => y,
            Expr::T => t,
            Expr::Num(n) => *n,
            Expr::Add(a, b) => a.eval(x, y, t) + b.eval(x, y, t),
            Expr::Mult(a, b) => a.eval(x, y, t) * b.eval(x, y, t),
            Expr::Sqrt(e) => e.eval(x, y, t).abs().sqrt(),
            Expr::Abs(e) => e.eval(x, y, t).abs(),
            Expr::Sin(e) => (e.eval(x, y, t) * std::f32::consts::PI).sin(),
            Expr::Mod(a, b) => {
                let bv = b.eval(x, y, t).abs().max(0.001);
                a.eval(x, y, t).rem_euclid(bv) / bv * 2.0 - 1.0
            }
            Expr::Mix(a, b, c) => {
                let w = ((c.eval(x, y, t) + 1.0) * 0.5).clamp(0.0, 1.0);
                a.eval(x, y, t) * (1.0 - w) + b.eval(x, y, t) * w
            }
        }
    }
    fn build_expr(rng: &mut StdRng, depth: u32) -> Self {
        const W: [u32; 7] = [2, 3, 3, 1, 2, 1, 1];
        let total: u32 = W.iter().sum();
        let mut ends = [0u32; 7];
        let mut acc = 0u32;
        for (i, &w) in W.iter().enumerate() {
            acc += w;
            ends[i] = acc;
        }
        let roll = rng.r#gen::<u32>() % total;
        let b = |r: &mut StdRng, d: u32| Box::new(Expr::build_expr(r, d));
        if depth == 0 || roll < ends[0] {
            return Self::terminal(rng);
        }
        if roll < ends[1] {
            return Expr::Add(b(rng, depth - 1), b(rng, depth - 1));
        }
        if roll < ends[2] {
            return Expr::Mult(b(rng, depth - 1), b(rng, depth - 1));
        }
        if roll < ends[3] {
            return Expr::Sqrt(Box::new(Expr::Abs(b(rng, depth - 1))));
        }
        if roll < ends[4] {
            return Expr::Sin(b(rng, depth - 1));
        }
        if roll < ends[5] {
            return Expr::Mod(b(rng, depth - 1), b(rng, depth - 1));
        }
        Expr::Mix(b(rng, depth - 1), b(rng, depth - 1), b(rng, depth - 1))
    }
    fn terminal(rng: &mut StdRng) -> Self {
        match rng.r#gen::<u32>() % 7 {
            0 => Expr::Num(rng.gen_range(-1.0f32..=1.0)),
            1 => Expr::X,
            2 => Expr::Y,
            3 => Expr::Abs(Box::new(Expr::X)),
            4 => Expr::Abs(Box::new(Expr::Y)),
            5 => Expr::Sqrt(Box::new(Expr::Add(
                Box::new(Expr::Mult(Box::new(Expr::X), Box::new(Expr::X))),
                Box::new(Expr::Mult(Box::new(Expr::Y), Box::new(Expr::Y))),
            ))),
            _ => Expr::T,
        }
    }
}

fn color_channel_to_byte(v: f32) -> u8 {
    (((v + 1.0) * 0.5).clamp(0.0, 1.0) * 255.0) as u8
}

pub struct ArtExpressions {
    r: Expr,
    g: Expr,
    b: Expr,
}
impl ArtExpressions {
    pub fn generate(seed: u64, depth: u32) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        Self {
            r: Expr::build_expr(&mut rng, depth),
            g: Expr::build_expr(&mut rng, depth),
            b: Expr::build_expr(&mut rng, depth),
        }
    }
    pub fn render(&self, w: u32, h: u32, t: f32) -> Vec<u8> {
        let mut pix = vec![255u8; (w * h * 4) as usize];
        for (i, ch) in pix.chunks_mut(4).enumerate() {
            let i = i as u32;
            let py = (i / w) as f32 / h as f32 * 2.0 - 1.0;
            let px = (i % w) as f32 / w as f32 * 2.0 - 1.0;
            ch[0] = color_channel_to_byte(self.r.eval(px, py, t));
            ch[1] = color_channel_to_byte(self.g.eval(px, py, t));
            ch[2] = color_channel_to_byte(self.b.eval(px, py, t));
        }
        pix
    }
}

// ─── CPU generative backgrounds ───────────────────────────────────────────────

fn hash_2d(ix: i32, iy: i32, seed: u64) -> f32 {
    let h = (ix as u64)
        .wrapping_mul(2654435761)
        .wrapping_add((iy as u64).wrapping_mul(2246822519))
        .wrapping_add(seed.wrapping_mul(0x9e3779b97f4a7c15));
    (h >> 32) as f32 / u32::MAX as f32
}
fn smooth_noise(x: f32, y: f32, seed: u64) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (a, b, c, d) = (
        hash_2d(ix, iy, seed),
        hash_2d(ix + 1, iy, seed),
        hash_2d(ix, iy + 1, seed),
        hash_2d(ix + 1, iy + 1, seed),
    );
    a * (1.0 - ux) * (1.0 - uy) + b * ux * (1.0 - uy) + c * (1.0 - ux) * uy + d * ux * uy
}

fn render_voronoi(w: u32, h: u32, seed: u64, t: f32) -> Vec<u8> {
    let mut pix = vec![255u8; (w * h * 4) as usize];
    let scale = 5.0 + (t + 1.0) * 1.5;
    for (i, ch) in pix.chunks_mut(4).enumerate() {
        let i = i as u32;
        let (px, py) = ((i % w) as f32 / w as f32, (i / w) as f32 / h as f32);
        let (sx, sy) = ((px * scale).floor() as i32, (py * scale).floor() as i32);
        let (mut d1, mut d2, mut cid) = (f32::MAX, f32::MAX, 0u64);
        for dy in -2..=2i32 {
            for dx in -2..=2i32 {
                let (cx, cy) = (sx + dx, sy + dy);
                let fx = (cx as f32 + hash_2d(cx, cy, seed)) / scale - px;
                let fy = (cy as f32 + hash_2d(cx, cy, seed.wrapping_add(999))) / scale - py;
                let d = (fx * fx + fy * fy).sqrt();
                if d < d1 {
                    d2 = d1;
                    d1 = d;
                    cid = (cx as u64).wrapping_mul(1000003).wrapping_add(cy as u64);
                } else if d < d2 {
                    d2 = d;
                }
            }
        }
        let e = ((d2 - d1) / (d1 + d2 + 0.001)).clamp(0.0, 1.0);
        let ef = (e * 8.0).clamp(0.0, 1.0);
        ch[0] = ((hash_2d(cid as i32, 0, seed) * 0.5 + 0.5) * ef * 255.0) as u8;
        ch[1] = ((hash_2d(cid as i32, 1, seed) * 0.5 + 0.5) * ef * 255.0) as u8;
        ch[2] = ((hash_2d(cid as i32, 2, seed) * 0.5 + 0.5) * ef * 255.0) as u8;
    }
    pix
}

fn render_flow_field(w: u32, h: u32, seed: u64, t: f32) -> Vec<u8> {
    let mut pix = vec![225u8; (w * h * 4) as usize];
    for ch in pix.chunks_mut(4) {
        ch[0] = 225;
        ch[1] = 235;
        ch[2] = 250;
        ch[3] = 255;
    }
    let freq = 3.5 + smooth_noise(0.1, 0.2, seed) * 2.0;
    let warp = 0.6 + t * 0.2;
    let lr = (hash_2d(seed as i32, 0, seed.wrapping_add(7)) * 100.0 + 30.0) as u8;
    let lg = (hash_2d(seed as i32, 1, seed.wrapping_add(7)) * 100.0 + 30.0) as u8;
    let lb = (hash_2d(seed as i32, 2, seed.wrapping_add(7)) * 100.0 + 100.0) as u8;
    for li in 0..20usize {
        let (mut px, mut py) = (
            hash_2d(li as i32, 0, seed.wrapping_add(li as u64 * 17)),
            hash_2d(li as i32, 1, seed.wrapping_add(li as u64 * 17)),
        );
        for _ in 0..60 {
            let nx = smooth_noise(px * freq, py * freq, seed);
            let ny = smooth_noise(px * freq + 13.7, py * freq + 7.3, seed);
            let angle = (nx * 2.0 - 1.0 + warp * (ny * 2.0 - 1.0)) * std::f32::consts::TAU;
            let (xi, yi) = ((px * w as f32) as i32, (py * h as f32) as i32);
            for ddy in -1i32..=1 {
                for ddx in -1i32..=1 {
                    let (x, y) = (xi + ddx, yi + ddy);
                    if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
                        let idx = (y as u32 * w + x as u32) as usize * 4;
                        pix[idx] = lr;
                        pix[idx + 1] = lg;
                        pix[idx + 2] = lb;
                    }
                }
            }
            px = (px + angle.cos() * 0.004).rem_euclid(1.0);
            py = (py + angle.sin() * 0.004).rem_euclid(1.0);
        }
    }
    pix
}

fn render_crackle(w: u32, h: u32, seed: u64, t: f32) -> Vec<u8> {
    let mut pix = vec![255u8; (w * h * 4) as usize];
    let scale = 4.0 + smooth_noise(0.0, 0.0, seed) * 3.0;
    let jitter = 0.8 + t * 0.1;
    let (bg, crack) = ([0.93f32, 0.97, 0.99], [0.05f32, 0.18, 0.28]);
    for (i, ch) in pix.chunks_mut(4).enumerate() {
        let i = i as u32;
        let (px, py) = ((i % w) as f32 / w as f32, (i / w) as f32 / h as f32);
        let (sx, sy) = ((px * scale).floor() as i32, (py * scale).floor() as i32);
        let (mut d1, mut d2) = (f32::MAX, f32::MAX);
        for dy in -2..=2i32 {
            for dx in -2..=2i32 {
                let (cx, cy) = (sx + dx, sy + dy);
                let rx = (hash_2d(cx, cy, seed) * 2.0 - 1.0) * jitter;
                let ry = (hash_2d(cx, cy, seed.wrapping_add(777)) * 2.0 - 1.0) * jitter;
                let fx = (cx as f32 + 0.5 + rx) / scale - px;
                let fy = (cy as f32 + 0.5 + ry) / scale - py;
                let d = (fx * fx + fy * fy).sqrt();
                if d < d1 {
                    d2 = d1;
                    d1 = d;
                } else if d < d2 {
                    d2 = d;
                }
            }
        }
        let e = ((d2 - d1) * scale * 4.0).clamp(0.0, 1.0);
        ch[0] = ((crack[0] * (1.0 - e) + bg[0] * e) * 255.0) as u8;
        ch[1] = ((crack[1] * (1.0 - e) + bg[1] * e) * 255.0) as u8;
        ch[2] = ((crack[2] * (1.0 - e) + bg[2] * e) * 255.0) as u8;
    }
    pix
}

fn render_op_art(w: u32, h: u32, seed: u64, t: f32) -> Vec<u8> {
    let mut pix = vec![255u8; (w * h * 4) as usize];
    let rings = 6.0 + smooth_noise(0.1, 0.1, seed) * 4.0;
    let warp = 0.3 + smooth_noise(0.2, 0.3, seed) * 0.4;
    let freq = 2.0 + smooth_noise(0.3, 0.4, seed) * 2.0;
    let twist = smooth_noise(0.4, 0.5, seed) * 1.5;
    let fg = [
        hash_2d(seed as i32, 0, seed),
        hash_2d(seed as i32, 1, seed),
        hash_2d(seed as i32, 2, seed),
    ];
    let bg = [
        hash_2d(seed as i32, 3, seed) * 0.3 + 0.7,
        hash_2d(seed as i32, 4, seed) * 0.3 + 0.7,
        hash_2d(seed as i32, 5, seed) * 0.3 + 0.7,
    ];
    for (i, ch) in pix.chunks_mut(4).enumerate() {
        let i = i as u32;
        let (px, py) = (
            (i % w) as f32 / w as f32 * 2.0 - 1.0,
            (i / w) as f32 / h as f32 * 2.0 - 1.0,
        );
        let wx = warp * (smooth_noise(px * freq + 0.1, py * freq + 0.1, seed) * 2.0 - 1.0);
        let wy = warp * (smooth_noise(px * freq + 5.7, py * freq + 3.2, seed) * 2.0 - 1.0);
        let r = ((px + wx) * (px + wx) + (py + wy) * (py + wy)).sqrt();
        let angle = (py + wy).atan2(px + wx);
        let v = (r * rings + angle * twist + t * std::f32::consts::TAU).sin();
        let tr = (v + 1.0) * 0.5;
        ch[0] = ((fg[0] * tr + bg[0] * (1.0 - tr)) * 255.0) as u8;
        ch[1] = ((fg[1] * tr + bg[1] * (1.0 - tr)) * 255.0) as u8;
        ch[2] = ((fg[2] * tr + bg[2] * (1.0 - tr)) * 255.0) as u8;
    }
    pix
}

impl ArtStyle {
    pub fn render(&self, exprs: &ArtExpressions, seed: u64, t: f32) -> Vec<u8> {
        match self {
            ArtStyle::ExprTree => exprs.render(ART_TEXTURE_SIZE, ART_TEXTURE_SIZE, t),
            ArtStyle::Voronoi => render_voronoi(ART_TEXTURE_SIZE, ART_TEXTURE_SIZE, seed, t),
            ArtStyle::FlowField => render_flow_field(ART_TEXTURE_SIZE, ART_TEXTURE_SIZE, seed, t),
            ArtStyle::Crackle => render_crackle(ART_TEXTURE_SIZE, ART_TEXTURE_SIZE, seed, t),
            ArtStyle::OpArt => render_op_art(ART_TEXTURE_SIZE, ART_TEXTURE_SIZE, seed, t),
        }
    }
}

// ─── Art state resource ───────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct ArtState {
    pub exprs: Vec<ArtExpressions>,
    pub seeds: Vec<u64>,
    pub handles: Vec<Handle<Image>>,
}

// ─── Color palettes (via colorous) ───────────────────────────────────────────

use crate::config::ColorPalette;

fn colorous_to_f32(c: colorous::Color) -> (f32, f32, f32) {
    (c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0)
}

pub fn palette_color(palette: ColorPalette, idx: usize) -> (f32, f32, f32) {
    match palette {
        ColorPalette::Pastel1 => colorous_to_f32(colorous::PASTEL1[idx % colorous::PASTEL1.len()]),
        ColorPalette::Pastel2 => colorous_to_f32(colorous::PASTEL2[idx % colorous::PASTEL2.len()]),
        ColorPalette::Set1 => colorous_to_f32(colorous::SET1[idx % colorous::SET1.len()]),
        ColorPalette::Set2 => colorous_to_f32(colorous::SET2[idx % colorous::SET2.len()]),
        ColorPalette::Set3 => colorous_to_f32(colorous::SET3[idx % colorous::SET3.len()]),
        ColorPalette::Tableau10 => {
            colorous_to_f32(colorous::TABLEAU10[idx % colorous::TABLEAU10.len()])
        }
        ColorPalette::Category10 => {
            colorous_to_f32(colorous::CATEGORY10[idx % colorous::CATEGORY10.len()])
        }
        ColorPalette::Accent => colorous_to_f32(colorous::ACCENT[idx % colorous::ACCENT.len()]),
        ColorPalette::Dark2 => colorous_to_f32(colorous::DARK2[idx % colorous::DARK2.len()]),
        ColorPalette::Paired => colorous_to_f32(colorous::PAIRED[idx % colorous::PAIRED.len()]),
    }
}

pub fn palette_bevy_color(palette: ColorPalette, idx: usize) -> Color {
    let (r, g, b) = palette_color(palette, idx);
    Color::srgb(r, g, b)
}
