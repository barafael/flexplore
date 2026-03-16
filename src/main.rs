//! Flex My Box — interactive Bevy 0.18 flexbox explorer.

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use rand::{Rng, SeedableRng, rngs::StdRng};

const PANEL_W: f32 = 390.0;
const ART_SZ: u32 = 128;

// ─── Val wrapper ─────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
enum ValCfg {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

impl ValCfg {
    fn to_val(&self) -> Val {
        match self {
            ValCfg::Auto => Val::Auto,
            ValCfg::Px(v) => Val::Px(*v),
            ValCfg::Percent(v) => Val::Percent(*v),
            ValCfg::Vw(v) => Val::Vw(*v),
            ValCfg::Vh(v) => Val::Vh(*v),
        }
    }
    fn variant(&self) -> &'static str {
        match self {
            ValCfg::Auto => "Auto",
            ValCfg::Px(_) => "Px",
            ValCfg::Percent(_) => "Percent",
            ValCfg::Vw(_) => "Vw",
            ValCfg::Vh(_) => "Vh",
        }
    }
    fn num(&self) -> Option<f32> {
        match self {
            ValCfg::Auto => None,
            ValCfg::Px(v) | ValCfg::Percent(v) | ValCfg::Vw(v) | ValCfg::Vh(v) => Some(*v),
        }
    }
    fn set_num(&mut self, n: f32) {
        match self {
            ValCfg::Px(v) | ValCfg::Percent(v) | ValCfg::Vw(v) | ValCfg::Vh(v) => *v = n,
            _ => {}
        }
    }
    fn cast(&self, variant: &str) -> Self {
        let n = self.num().unwrap_or(100.0);
        match variant {
            "Px" => ValCfg::Px(n),
            "Percent" => ValCfg::Percent(n),
            "Vw" => ValCfg::Vw(n),
            "Vh" => ValCfg::Vh(n),
            _ => ValCfg::Auto,
        }
    }
}

// ─── Per-item config ──────────────────────────────────────────────────────────

#[derive(Clone)]
struct ItemCfg {
    flex_grow: f32,
    flex_shrink: f32,
    flex_basis: ValCfg,
    align_self: AlignSelf,
    width: ValCfg,
    height: ValCfg,
    min_width: ValCfg,
    min_height: ValCfg,
    max_width: ValCfg,
    max_height: ValCfg,
    padding: ValCfg,
    margin: ValCfg,
}

impl ItemCfg {
    fn new(w: f32, h: f32) -> Self {
        Self {
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: ValCfg::Auto,
            align_self: AlignSelf::Auto,
            width: ValCfg::Px(w),
            height: ValCfg::Px(h),
            min_width: ValCfg::Auto,
            min_height: ValCfg::Auto,
            max_width: ValCfg::Auto,
            max_height: ValCfg::Auto,
            padding: ValCfg::Px(8.0),
            margin: ValCfg::Px(0.0),
        }
    }
}

// ─── Background mode + art style ─────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
enum BgMode {
    Pastel,
    RandomArt,
}

#[derive(Clone, PartialEq, Debug)]
enum ArtStyle {
    ExprTree,
    Voronoi,
    FlowField,
    Crackle,
    OpArt,
}

impl ArtStyle {
    const ALL: &'static [(&'static str, ArtStyle)] = &[
        ("Expr Tree", ArtStyle::ExprTree),
        ("Voronoi", ArtStyle::Voronoi),
        ("Flow Field", ArtStyle::FlowField),
        ("Crackle", ArtStyle::Crackle),
        ("Op Art", ArtStyle::OpArt),
    ];
}

// ─── Main resource ────────────────────────────────────────────────────────────

#[derive(Resource, Clone)]
struct FlexCfg {
    flex_direction: FlexDirection,
    flex_wrap: FlexWrap,
    justify_content: JustifyContent,
    align_items: AlignItems,
    align_content: AlignContent,
    row_gap: ValCfg,
    column_gap: ValCfg,
    padding: ValCfg,
    container_width: ValCfg,
    container_height: ValCfg,
    container_min_height: ValCfg,
    items: Vec<ItemCfg>,
    selected: usize,
    bg_mode: BgMode,
    art_style: ArtStyle,
    art_seed: u64,
    art_depth: u32,
    art_anim: f32,
    needs_rebuild: bool,
}

impl Default for FlexCfg {
    fn default() -> Self {
        Self {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::FlexStart,
            row_gap: ValCfg::Px(8.0),
            column_gap: ValCfg::Px(8.0),
            padding: ValCfg::Px(12.0),
            container_width: ValCfg::Percent(100.0),
            container_height: ValCfg::Auto,
            container_min_height: ValCfg::Px(200.0),
            items: vec![
                ItemCfg::new(80.0, 80.0),
                ItemCfg::new(120.0, 100.0),
                ItemCfg::new(60.0, 60.0),
                ItemCfg::new(100.0, 80.0),
            ],
            selected: 0,
            bg_mode: BgMode::Pastel,
            art_style: ArtStyle::ExprTree,
            art_seed: 137,
            art_depth: 5,
            art_anim: 0.0,
            needs_rebuild: true,
        }
    }
}

// ─── RandomFart expression tree (from ../tessel/randomfart) ──────────────────

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

fn art_ch(v: f32) -> u8 {
    (((v + 1.0) * 0.5).clamp(0.0, 1.0) * 255.0) as u8
}

struct ArtExprs {
    r: Expr,
    g: Expr,
    b: Expr,
}
impl ArtExprs {
    fn generate(seed: u64, depth: u32) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        Self {
            r: Expr::build_expr(&mut rng, depth),
            g: Expr::build_expr(&mut rng, depth),
            b: Expr::build_expr(&mut rng, depth),
        }
    }
    fn render(&self, w: u32, h: u32, t: f32) -> Vec<u8> {
        let mut pix = vec![255u8; (w * h * 4) as usize];
        for (i, ch) in pix.chunks_mut(4).enumerate() {
            let i = i as u32;
            let py = (i / w) as f32 / h as f32 * 2.0 - 1.0;
            let px = (i % w) as f32 / w as f32 * 2.0 - 1.0;
            ch[0] = art_ch(self.r.eval(px, py, t));
            ch[1] = art_ch(self.g.eval(px, py, t));
            ch[2] = art_ch(self.b.eval(px, py, t));
        }
        pix
    }
}

// ─── CPU generative backgrounds (formulas from ../tessel/tessel) ─────────────

fn hash2(ix: i32, iy: i32, seed: u64) -> f32 {
    let h = (ix as u64)
        .wrapping_mul(2654435761)
        .wrapping_add((iy as u64).wrapping_mul(2246822519))
        .wrapping_add(seed.wrapping_mul(0x9e3779b97f4a7c15));
    (h >> 32) as f32 / u32::MAX as f32
}
fn snoise(x: f32, y: f32, seed: u64) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (a, b, c, d) = (
        hash2(ix, iy, seed),
        hash2(ix + 1, iy, seed),
        hash2(ix, iy + 1, seed),
        hash2(ix + 1, iy + 1, seed),
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
                let fx = (cx as f32 + hash2(cx, cy, seed)) / scale - px;
                let fy = (cy as f32 + hash2(cx, cy, seed.wrapping_add(999))) / scale - py;
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
        ch[0] = ((hash2(cid as i32, 0, seed) * 0.5 + 0.5) * ef * 255.0) as u8;
        ch[1] = ((hash2(cid as i32, 1, seed) * 0.5 + 0.5) * ef * 255.0) as u8;
        ch[2] = ((hash2(cid as i32, 2, seed) * 0.5 + 0.5) * ef * 255.0) as u8;
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
    let freq = 3.5 + snoise(0.1, 0.2, seed) * 2.0;
    let warp = 0.6 + t * 0.2;
    let lr = (hash2(seed as i32, 0, seed.wrapping_add(7)) * 100.0 + 30.0) as u8;
    let lg = (hash2(seed as i32, 1, seed.wrapping_add(7)) * 100.0 + 30.0) as u8;
    let lb = (hash2(seed as i32, 2, seed.wrapping_add(7)) * 100.0 + 100.0) as u8;
    for li in 0..20usize {
        let (mut px, mut py) = (
            hash2(li as i32, 0, seed.wrapping_add(li as u64 * 17)),
            hash2(li as i32, 1, seed.wrapping_add(li as u64 * 17)),
        );
        for _ in 0..60 {
            let nx = snoise(px * freq, py * freq, seed);
            let ny = snoise(px * freq + 13.7, py * freq + 7.3, seed);
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
    let scale = 4.0 + snoise(0.0, 0.0, seed) * 3.0;
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
                let rx = (hash2(cx, cy, seed) * 2.0 - 1.0) * jitter;
                let ry = (hash2(cx, cy, seed.wrapping_add(777)) * 2.0 - 1.0) * jitter;
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
    let rings = 6.0 + snoise(0.1, 0.1, seed) * 4.0;
    let warp = 0.3 + snoise(0.2, 0.3, seed) * 0.4;
    let freq = 2.0 + snoise(0.3, 0.4, seed) * 2.0;
    let twist = snoise(0.4, 0.5, seed) * 1.5;
    let fg = [
        hash2(seed as i32, 0, seed),
        hash2(seed as i32, 1, seed),
        hash2(seed as i32, 2, seed),
    ];
    let bg = [
        hash2(seed as i32, 3, seed) * 0.3 + 0.7,
        hash2(seed as i32, 4, seed) * 0.3 + 0.7,
        hash2(seed as i32, 5, seed) * 0.3 + 0.7,
    ];
    for (i, ch) in pix.chunks_mut(4).enumerate() {
        let i = i as u32;
        let (px, py) = (
            (i % w) as f32 / w as f32 * 2.0 - 1.0,
            (i / w) as f32 / h as f32 * 2.0 - 1.0,
        );
        let wx = warp * (snoise(px * freq + 0.1, py * freq + 0.1, seed) * 2.0 - 1.0);
        let wy = warp * (snoise(px * freq + 5.7, py * freq + 3.2, seed) * 2.0 - 1.0);
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

fn render_art(style: &ArtStyle, exprs: &ArtExprs, seed: u64, t: f32) -> Vec<u8> {
    match style {
        ArtStyle::ExprTree => exprs.render(ART_SZ, ART_SZ, t),
        ArtStyle::Voronoi => render_voronoi(ART_SZ, ART_SZ, seed, t),
        ArtStyle::FlowField => render_flow_field(ART_SZ, ART_SZ, seed, t),
        ArtStyle::Crackle => render_crackle(ART_SZ, ART_SZ, seed, t),
        ArtStyle::OpArt => render_op_art(ART_SZ, ART_SZ, seed, t),
    }
}

// ─── Art state resource ───────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct ArtState {
    exprs: Vec<ArtExprs>,
    handles: Vec<Handle<Image>>,
}

// ─── Pastel palette ───────────────────────────────────────────────────────────

const PASTELS: &[(f32, f32, f32)] = &[
    (1.00, 0.80, 0.80),
    (0.80, 1.00, 0.82),
    (0.82, 0.86, 1.00),
    (1.00, 1.00, 0.80),
    (1.00, 0.90, 0.80),
    (0.80, 0.96, 1.00),
    (0.94, 0.82, 1.00),
    (0.82, 0.94, 0.82),
    (1.00, 0.86, 0.94),
    (0.88, 0.96, 1.00),
    (0.94, 1.00, 0.88),
    (1.00, 0.94, 0.86),
];

fn pastel(idx: usize) -> Color {
    let (r, g, b) = PASTELS[idx % PASTELS.len()];
    Color::srgb(r, g, b)
}

// ─── Components ───────────────────────────────────────────────────────────────

#[derive(Component)]
struct VizRoot;
#[derive(Component)]
#[allow(dead_code)]
struct ArtItemNode(usize);

// ─── App ─────────────────────────────────────────────────────────────────────

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Flex My Box — Flexbox Explorer".into(),
                    resolution: (1280u32, 800u32).into(),
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin::default(),
        ))
        .init_resource::<FlexCfg>()
        .init_resource::<ArtState>()
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, panel_system)
        .add_systems(Update, (rebuild_viz, animate_art).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ─── Panel ────────────────────────────────────────────────────────────────────

fn panel_system(
    mut contexts: EguiContexts,
    mut cfg: ResMut<FlexCfg>,
    mut preview: Local<Option<FlexCfg>>,
    mut style_done: Local<bool>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    if !*style_done {
        // Three colors: near-black bg · mid-grey surface · warm off-white text/accent
        const BG:  egui::Color32 = egui::Color32::from_rgb(0x12, 0x12, 0x14);
        const MID: egui::Color32 = egui::Color32::from_rgb(0x22, 0x22, 0x26);
        const FG:  egui::Color32 = egui::Color32::from_rgb(0xd8, 0xd4, 0xc8);

        let mut v = egui::Visuals::dark();

        // Panels / windows
        v.panel_fill             = BG;
        v.window_fill            = BG;
        v.extreme_bg_color       = BG;

        // Widgets (idle, hovered, active)
        v.widgets.inactive.bg_fill       = MID;
        v.widgets.inactive.weak_bg_fill  = MID;
        v.widgets.inactive.bg_stroke     = egui::Stroke::new(1.0, MID);
        v.widgets.inactive.fg_stroke     = egui::Stroke::new(1.0, FG);


        v.widgets.hovered.bg_fill        = egui::Color32::from_rgb(0x30, 0x30, 0x38);
        v.widgets.hovered.weak_bg_fill   = egui::Color32::from_rgb(0x30, 0x30, 0x38);
        v.widgets.hovered.bg_stroke      = egui::Stroke::new(1.0, FG);
        v.widgets.hovered.fg_stroke      = egui::Stroke::new(1.5, FG);


        v.widgets.active.bg_fill         = FG;
        v.widgets.active.weak_bg_fill    = FG;
        v.widgets.active.fg_stroke       = egui::Stroke::new(1.5, BG);


        v.widgets.open.bg_fill           = MID;
        v.widgets.open.fg_stroke         = egui::Stroke::new(1.0, FG);


        v.widgets.noninteractive.bg_fill      = BG;
        v.widgets.noninteractive.fg_stroke    = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x55, 0x54, 0x50));
        v.widgets.noninteractive.bg_stroke    = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x28, 0x28, 0x2c));

        // Text
        v.override_text_color = Some(FG);

        // Separators / faint lines
        v.window_stroke   = egui::Stroke::new(1.0, MID);
        v.selection.bg_fill = egui::Color32::from_rgb(0x38, 0x38, 0x44);

        ctx.set_visuals(v);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing   = egui::vec2(6.0, 3.0);
        style.spacing.button_padding = egui::vec2(6.0, 2.0);
        style.spacing.slider_width   = 110.0;
        ctx.set_style(style);

        *style_done = true;
    }
    let mut changed = false;
    let mut any_hovered = false;

    egui::SidePanel::left("flex_panel")
        .exact_width(PANEL_W)
        .resizable(false)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);

                // ── Container ────────────────────────────────────────────────────
                egui::CollapsingHeader::new("Container")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        egui::Grid::new("cg1").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                            ui.label("direction");
                            let h = combo(ui, "fd", &mut cfg.flex_direction, &[
                                ("Row", FlexDirection::Row),
                                ("Column", FlexDirection::Column),
                                ("RowReverse", FlexDirection::RowReverse),
                                ("ColumnReverse", FlexDirection::ColumnReverse),
                            ], &mut changed, &mut any_hovered);
                            ui.end_row();
                            if let Some(v) = h { any_hovered = true; if cfg.flex_direction != v { if preview.is_none() { *preview = Some(cfg.clone()); } cfg.flex_direction = v; cfg.needs_rebuild = true; } }

                            ui.label("wrap");
                            let h = combo(ui, "fw", &mut cfg.flex_wrap, &[
                                ("NoWrap", FlexWrap::NoWrap),
                                ("Wrap", FlexWrap::Wrap),
                                ("WrapReverse", FlexWrap::WrapReverse),
                            ], &mut changed, &mut any_hovered);
                            ui.end_row();
                            if let Some(v) = h { any_hovered = true; if cfg.flex_wrap != v { if preview.is_none() { *preview = Some(cfg.clone()); } cfg.flex_wrap = v; cfg.needs_rebuild = true; } }

                            ui.label("justify");
                            let h = combo(ui, "jc", &mut cfg.justify_content, &[
                                ("Default", JustifyContent::Default),
                                ("FlexStart", JustifyContent::FlexStart),
                                ("FlexEnd", JustifyContent::FlexEnd),
                                ("Center", JustifyContent::Center),
                                ("SpaceBetween", JustifyContent::SpaceBetween),
                                ("SpaceAround", JustifyContent::SpaceAround),
                                ("SpaceEvenly", JustifyContent::SpaceEvenly),
                                ("Stretch", JustifyContent::Stretch),
                                ("Start", JustifyContent::Start),
                                ("End", JustifyContent::End),
                            ], &mut changed, &mut any_hovered);
                            ui.end_row();
                            if let Some(v) = h { any_hovered = true; if cfg.justify_content != v { if preview.is_none() { *preview = Some(cfg.clone()); } cfg.justify_content = v; cfg.needs_rebuild = true; } }

                            ui.label("align-items");
                            let h = combo(ui, "ai", &mut cfg.align_items, &[
                                ("Default", AlignItems::Default),
                                ("FlexStart", AlignItems::FlexStart),
                                ("FlexEnd", AlignItems::FlexEnd),
                                ("Center", AlignItems::Center),
                                ("Baseline", AlignItems::Baseline),
                                ("Stretch", AlignItems::Stretch),
                                ("Start", AlignItems::Start),
                                ("End", AlignItems::End),
                            ], &mut changed, &mut any_hovered);
                            ui.end_row();
                            if let Some(v) = h { any_hovered = true; if cfg.align_items != v { if preview.is_none() { *preview = Some(cfg.clone()); } cfg.align_items = v; cfg.needs_rebuild = true; } }

                            ui.label("align-content");
                            let h = combo(ui, "ac", &mut cfg.align_content, &[
                                ("Default", AlignContent::Default),
                                ("FlexStart", AlignContent::FlexStart),
                                ("FlexEnd", AlignContent::FlexEnd),
                                ("Center", AlignContent::Center),
                                ("SpaceBetween", AlignContent::SpaceBetween),
                                ("SpaceAround", AlignContent::SpaceAround),
                                ("SpaceEvenly", AlignContent::SpaceEvenly),
                                ("Stretch", AlignContent::Stretch),
                                ("Start", AlignContent::Start),
                                ("End", AlignContent::End),
                            ], &mut changed, &mut any_hovered);
                            ui.end_row();
                            if let Some(v) = h { any_hovered = true; if cfg.align_content != v { if preview.is_none() { *preview = Some(cfg.clone()); } cfg.align_content = v; cfg.needs_rebuild = true; } }
                        });

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(4.0);

                        egui::Grid::new("cg2").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                            ui.label("row-gap");
                            let h = val_row(ui, "rg", &mut cfg.row_gap, &mut changed, &mut any_hovered);
                            ui.end_row();
                            if let Some(v) = h { any_hovered = true; if cfg.row_gap != v { if preview.is_none() { *preview = Some(cfg.clone()); } cfg.row_gap = v; cfg.needs_rebuild = true; } }

                            ui.label("column-gap");
                            let h = val_row(ui, "cg", &mut cfg.column_gap, &mut changed, &mut any_hovered);
                            ui.end_row();
                            if let Some(v) = h { any_hovered = true; if cfg.column_gap != v { if preview.is_none() { *preview = Some(cfg.clone()); } cfg.column_gap = v; cfg.needs_rebuild = true; } }

                            ui.label("padding");
                            let h = val_row(ui, "cp", &mut cfg.padding, &mut changed, &mut any_hovered);
                            ui.end_row();
                            if let Some(v) = h { any_hovered = true; if cfg.padding != v { if preview.is_none() { *preview = Some(cfg.clone()); } cfg.padding = v; cfg.needs_rebuild = true; } }

                            ui.label("width");
                            let h = val_row(ui, "cw", &mut cfg.container_width, &mut changed, &mut any_hovered);
                            ui.end_row();
                            if let Some(v) = h { any_hovered = true; if cfg.container_width != v { if preview.is_none() { *preview = Some(cfg.clone()); } cfg.container_width = v; cfg.needs_rebuild = true; } }

                            ui.label("height");
                            let h = val_row(ui, "ch", &mut cfg.container_height, &mut changed, &mut any_hovered);
                            ui.end_row();
                            if let Some(v) = h { any_hovered = true; if cfg.container_height != v { if preview.is_none() { *preview = Some(cfg.clone()); } cfg.container_height = v; cfg.needs_rebuild = true; } }

                            ui.label("min-height");
                            let h = val_row(ui, "cmh", &mut cfg.container_min_height, &mut changed, &mut any_hovered);
                            ui.end_row();
                            if let Some(v) = h { any_hovered = true; if cfg.container_min_height != v { if preview.is_none() { *preview = Some(cfg.clone()); } cfg.container_min_height = v; cfg.needs_rebuild = true; } }
                        });
                        ui.add_space(2.0);
                    });

                ui.add_space(6.0);

                // ── Items ─────────────────────────────────────────────────────────
                egui::CollapsingHeader::new("Items")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        let mut n = cfg.items.len();
                        if ui
                            .add(egui::Slider::new(&mut n, 1..=12).text("count"))
                            .changed()
                        {
                            while cfg.items.len() < n {
                                let w = 60.0 + (cfg.items.len() % 6) as f32 * 20.0;
                                cfg.items.push(ItemCfg::new(w, w));
                            }
                            cfg.items.truncate(n);
                            if cfg.selected >= cfg.items.len() {
                                cfg.selected = cfg.items.len() - 1;
                            }
                            changed = true;
                        }
                        ui.add_space(3.0);
                        ui.horizontal_wrapped(|ui| {
                            for i in 0..cfg.items.len() {
                                if ui
                                    .selectable_label(cfg.selected == i, format!(" {} ", i + 1))
                                    .clicked()
                                {
                                    cfg.selected = i;
                                }
                            }
                            if ui.button(" + ").clicked() && cfg.items.len() < 12 {
                                let w = 60.0 + (cfg.items.len() % 6) as f32 * 20.0;
                                cfg.items.push(ItemCfg::new(w, w));
                                cfg.selected = cfg.items.len() - 1;
                                changed = true;
                            }
                            if ui.button(" - ").clicked() && cfg.items.len() > 1 {
                                let sel = cfg.selected;
                                cfg.items.remove(sel);
                                if cfg.selected >= cfg.items.len() {
                                    cfg.selected = cfg.items.len() - 1;
                                }
                                changed = true;
                            }
                        });
                        ui.separator();
                        let sel = cfg.selected;

                        // Collect hover events inside the item borrow block,
                        // then apply them after — avoids conflict with cfg.clone().
                        let mut ih_align: Option<AlignSelf> = None;
                        let mut ih_flex_basis: Option<ValCfg> = None;
                        let mut ih_width: Option<ValCfg> = None;
                        let mut ih_height: Option<ValCfg> = None;
                        let mut ih_min_width: Option<ValCfg> = None;
                        let mut ih_min_height: Option<ValCfg> = None;
                        let mut ih_max_width: Option<ValCfg> = None;
                        let mut ih_max_height: Option<ValCfg> = None;
                        let mut ih_padding: Option<ValCfg> = None;
                        let mut ih_margin: Option<ValCfg> = None;
                        let mut apply_all_clicked = false;
                        let mut apply_all_item: Option<ItemCfg> = None;

                        if let Some(item) = cfg.items.get_mut(sel) {
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(format!("Item {}", sel + 1)).strong());
                            ui.add_space(4.0);

                            egui::Grid::new("ig1").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                                ui.label("flex-grow");
                                changed |= ui.add(egui::Slider::new(&mut item.flex_grow, 0.0..=5.0).max_decimals(2)).changed();
                                ui.end_row();

                                ui.label("flex-shrink");
                                changed |= ui.add(egui::Slider::new(&mut item.flex_shrink, 0.0..=5.0).max_decimals(2)).changed();
                                ui.end_row();

                                ui.label("flex-basis");
                                ih_flex_basis = val_row(ui, "ib", &mut item.flex_basis, &mut changed, &mut any_hovered);
                                ui.end_row();

                                ui.label("align-self");
                                ih_align = combo(ui, "as", &mut item.align_self, &[
                                    ("Auto", AlignSelf::Auto),
                                    ("FlexStart", AlignSelf::FlexStart),
                                    ("FlexEnd", AlignSelf::FlexEnd),
                                    ("Center", AlignSelf::Center),
                                    ("Baseline", AlignSelf::Baseline),
                                    ("Stretch", AlignSelf::Stretch),
                                    ("Start", AlignSelf::Start),
                                    ("End", AlignSelf::End),
                                ], &mut changed, &mut any_hovered);
                                ui.end_row();
                            });

                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            egui::Grid::new("ig2").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                                ui.label("width");
                                ih_width = val_row(ui, "iw", &mut item.width, &mut changed, &mut any_hovered);
                                ui.end_row();

                                ui.label("height");
                                ih_height = val_row(ui, "ih", &mut item.height, &mut changed, &mut any_hovered);
                                ui.end_row();

                                ui.label("min-width");
                                ih_min_width = val_row(ui, "iminw", &mut item.min_width, &mut changed, &mut any_hovered);
                                ui.end_row();

                                ui.label("min-height");
                                ih_min_height = val_row(ui, "iminh", &mut item.min_height, &mut changed, &mut any_hovered);
                                ui.end_row();

                                ui.label("max-width");
                                ih_max_width = val_row(ui, "imaxw", &mut item.max_width, &mut changed, &mut any_hovered);
                                ui.end_row();

                                ui.label("max-height");
                                ih_max_height = val_row(ui, "imaxh", &mut item.max_height, &mut changed, &mut any_hovered);
                                ui.end_row();

                                ui.label("padding");
                                ih_padding = val_row(ui, "ipad", &mut item.padding, &mut changed, &mut any_hovered);
                                ui.end_row();

                                ui.label("margin");
                                ih_margin = val_row(ui, "imar", &mut item.margin, &mut changed, &mut any_hovered);
                                ui.end_row();
                            });

                            ui.add_space(6.0);
                            if ui.button("Apply to all").clicked() {
                                apply_all_item = Some(item.clone());
                                apply_all_clicked = true;
                            }
                        }

                        // Apply item hover previews (item borrow is released above).
                        let item_has_hover = ih_align.is_some()
                            || ih_flex_basis.is_some()
                            || ih_width.is_some()
                            || ih_height.is_some()
                            || ih_min_width.is_some()
                            || ih_min_height.is_some()
                            || ih_max_width.is_some()
                            || ih_max_height.is_some()
                            || ih_padding.is_some()
                            || ih_margin.is_some();
                        if item_has_hover {
                            any_hovered = true;
                            let mut rebuild = false;
                            if let Some(v) = ih_align {
                                if cfg.items[sel].align_self != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.items[sel].align_self = v; rebuild = true;
                                }
                            }
                            if let Some(v) = ih_flex_basis {
                                if cfg.items[sel].flex_basis != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.items[sel].flex_basis = v; rebuild = true;
                                }
                            }
                            if let Some(v) = ih_width {
                                if cfg.items[sel].width != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.items[sel].width = v; rebuild = true;
                                }
                            }
                            if let Some(v) = ih_height {
                                if cfg.items[sel].height != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.items[sel].height = v; rebuild = true;
                                }
                            }
                            if let Some(v) = ih_min_width {
                                if cfg.items[sel].min_width != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.items[sel].min_width = v; rebuild = true;
                                }
                            }
                            if let Some(v) = ih_min_height {
                                if cfg.items[sel].min_height != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.items[sel].min_height = v; rebuild = true;
                                }
                            }
                            if let Some(v) = ih_max_width {
                                if cfg.items[sel].max_width != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.items[sel].max_width = v; rebuild = true;
                                }
                            }
                            if let Some(v) = ih_max_height {
                                if cfg.items[sel].max_height != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.items[sel].max_height = v; rebuild = true;
                                }
                            }
                            if let Some(v) = ih_padding {
                                if cfg.items[sel].padding != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.items[sel].padding = v; rebuild = true;
                                }
                            }
                            if let Some(v) = ih_margin {
                                if cfg.items[sel].margin != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.items[sel].margin = v; rebuild = true;
                                }
                            }
                            if rebuild { cfg.needs_rebuild = true; }
                        }

                        if apply_all_clicked {
                            if let Some(t) = apply_all_item {
                                for it in cfg.items.iter_mut() {
                                    *it = t.clone();
                                }
                                changed = true;
                            }
                        }
                    });

                ui.add_space(6.0);

                // ── Background ────────────────────────────────────────────────────
                egui::CollapsingHeader::new("Background")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let prev = cfg.bg_mode.clone();
                            ui.radio_value(&mut cfg.bg_mode, BgMode::Pastel, "Pastel");
                            ui.radio_value(&mut cfg.bg_mode, BgMode::RandomArt, "Generative Art");
                            if cfg.bg_mode != prev {
                                changed = true;
                            }
                        });
                        if cfg.bg_mode == BgMode::RandomArt {
                            let cur = ArtStyle::ALL
                                .iter()
                                .find(|(_, s)| *s == cfg.art_style)
                                .map(|(n, _)| *n)
                                .unwrap_or("?");
                            let mut hover_art: Option<ArtStyle> = None;
                            let art_resp = egui::ComboBox::from_label("style")
                                .selected_text(cur)
                                .show_ui(ui, |ui| {
                                    for (name, style) in ArtStyle::ALL {
                                        let r = ui.selectable_label(cfg.art_style == *style, *name);
                                        if r.clicked() {
                                            cfg.art_style = style.clone();
                                            changed = true;
                                        } else if r.hovered() {
                                            hover_art = Some(style.clone());
                                        }
                                    }
                                });
                            if art_resp.inner.is_some() {
                                any_hovered = true;
                            }
                            if let Some(v) = hover_art {
                                any_hovered = true;
                                if cfg.art_style != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.art_style = v; cfg.needs_rebuild = true;
                                }
                            }
                            let pd = cfg.art_depth;
                            ui.add(egui::Slider::new(&mut cfg.art_depth, 1..=9).text("depth"));
                            if cfg.art_depth != pd {
                                changed = true;
                            }
                            ui.add(
                                egui::Slider::new(&mut cfg.art_anim, 0.0..=2.0)
                                    .text("anim speed")
                                    .step_by(0.05),
                            );
                            ui.horizontal(|ui| {
                                if ui.button("New seed").clicked() {
                                    cfg.art_seed = rand::random::<u64>();
                                    changed = true;
                                }
                                if ui.button("Regenerate").clicked() {
                                    changed = true;
                                }
                            });
                        }
                    });

                ui.add_space(6.0);
                if ui.button("Reset to defaults").clicked() {
                    *cfg = FlexCfg::default();
                    *preview = None;
                }
            });
        });

    if changed {
        *preview = None;
        cfg.needs_rebuild = true;
    } else if !any_hovered {
        if let Some(saved) = preview.take() {
            *cfg = saved;
            cfg.needs_rebuild = true;
        }
    }
    Ok(())
}

// ─── Rebuild ──────────────────────────────────────────────────────────────────

fn rebuild_viz(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut cfg: ResMut<FlexCfg>,
    mut art: ResMut<ArtState>,
    roots: Query<Entity, With<VizRoot>>,
) {
    if !cfg.needs_rebuild {
        return;
    }
    cfg.needs_rebuild = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    art.exprs.clear();
    art.handles.clear();
    if cfg.bg_mode == BgMode::RandomArt {
        let (n, base, depth, style) = (
            cfg.items.len(),
            cfg.art_seed,
            cfg.art_depth,
            cfg.art_style.clone(),
        );
        for i in 0..n {
            let iseed = base.wrapping_add((i as u64).wrapping_mul(0x9e3779b97f4a7c15));
            let exprs = ArtExprs::generate(iseed, depth);
            let pixels = render_art(&style, &exprs, iseed, 0.0);
            let image = Image::new(
                Extent3d {
                    width: ART_SZ,
                    height: ART_SZ,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                pixels,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            art.handles.push(images.add(image));
            art.exprs.push(exprs);
        }
    }
    spawn_viz(&mut commands, &*cfg, &*art);
}

fn spawn_viz(commands: &mut Commands, cfg: &FlexCfg, art: &ArtState) {
    commands
        .spawn((
            VizRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Px(PANEL_W),
                flex_shrink: 0.0,
                ..default()
            });
            root.spawn(Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            })
            .with_children(|area| {
                area.spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: cfg.flex_direction,
                        flex_wrap: cfg.flex_wrap,
                        justify_content: cfg.justify_content,
                        align_items: cfg.align_items,
                        align_content: cfg.align_content,
                        row_gap: cfg.row_gap.to_val(),
                        column_gap: cfg.column_gap.to_val(),
                        padding: UiRect::all(cfg.padding.to_val()),
                        width: cfg.container_width.to_val(),
                        height: cfg.container_height.to_val(),
                        min_height: cfg.container_min_height.to_val(),
                        flex_grow: 1.0,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.11, 0.11, 0.17, 1.0)),
                    BorderColor::all(Color::srgba(0.35, 0.35, 0.55, 1.0)),
                ))
                .with_children(|container| {
                    for (idx, item) in cfg.items.iter().enumerate() {
                        let sel = idx == cfg.selected;
                        let (bw, bc) = if sel {
                            (3.0, Color::srgba(1.0, 0.85, 0.1, 1.0))
                        } else {
                            (1.5, Color::srgba(0.0, 0.0, 0.0, 0.35))
                        };
                        let mut e = container.spawn((
                            ArtItemNode(idx),
                            Node {
                                flex_grow: item.flex_grow,
                                flex_shrink: item.flex_shrink,
                                flex_basis: item.flex_basis.to_val(),
                                align_self: item.align_self,
                                width: item.width.to_val(),
                                height: item.height.to_val(),
                                min_width: item.min_width.to_val(),
                                min_height: item.min_height.to_val(),
                                max_width: item.max_width.to_val(),
                                max_height: item.max_height.to_val(),
                                padding: UiRect::all(item.padding.to_val()),
                                margin: UiRect::all(item.margin.to_val()),
                                border: UiRect::all(Val::Px(bw)),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(if cfg.bg_mode == BgMode::Pastel {
                                pastel(idx)
                            } else {
                                Color::WHITE
                            }),
                            BorderColor::all(bc),
                        ));
                        if cfg.bg_mode == BgMode::RandomArt {
                            if let Some(h) = art.handles.get(idx) {
                                e.insert(ImageNode::new(h.clone()));
                            }
                        }
                        let info = item_info(idx, item);
                        e.with_children(|node| {
                            node.spawn((
                                Text::new(format!("{}", idx + 1)),
                                TextFont {
                                    font_size: 26.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
                            ));
                            node.spawn((
                                Text::new(info),
                                TextFont {
                                    font_size: 9.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.05, 0.05, 0.1, 0.70)),
                            ));
                        });
                    }
                });
            });
        });
}

// ─── Animation ────────────────────────────────────────────────────────────────

fn animate_art(
    mut images: ResMut<Assets<Image>>,
    art: Res<ArtState>,
    cfg: Res<FlexCfg>,
    time: Res<Time>,
    mut last_t: Local<f32>,
) {
    if cfg.art_anim < 1e-4 || cfg.bg_mode != BgMode::RandomArt {
        return;
    }
    let t = (time.elapsed_secs() * cfg.art_anim).sin();
    if (t - *last_t).abs() < 1e-4 {
        return;
    }
    *last_t = t;
    for (exprs, handle) in art.exprs.iter().zip(art.handles.iter()) {
        if let Some(image) = images.get_mut(handle) {
            let pixels: Vec<u8> = exprs.render(ART_SZ, ART_SZ, t);
            image.data = Some(pixels);
        }
    }
}

// ─── egui helpers ─────────────────────────────────────────────────────────────

fn combo<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut T,
    options: &[(&str, T)],
    changed: &mut bool,
    popup_open: &mut bool,
) -> Option<T> {
    let sel = options
        .iter()
        .find(|(_, v)| *v == *val)
        .map(|(s, _)| *s)
        .unwrap_or("?");
    let mut hover = None;
    let resp = egui::ComboBox::from_id_salt(label)
        .selected_text(sel)
        .width(130.0)
        .show_ui(ui, |ui| {
            for (name, opt) in options {
                let r = ui.selectable_label(*val == *opt, *name);
                if r.clicked() {
                    *val = *opt;
                    *changed = true;
                } else if r.hovered() {
                    hover = Some(*opt);
                }
            }
        });
    if resp.inner.is_some() {
        *popup_open = true;
    }
    hover
}

fn val_row(
    ui: &mut egui::Ui,
    id: &str,
    val: &mut ValCfg,
    changed: &mut bool,
    popup_open: &mut bool,
) -> Option<ValCfg> {
    const VARIANTS: &[&str] = &["Auto", "Px", "Percent", "Vw", "Vh"];
    let mut hover = None;
    let mut is_open = false;
    ui.horizontal(|ui| {
        let cur = val.variant();
        let resp = egui::ComboBox::from_id_salt(id)
            .width(72.0)
            .selected_text(cur)
            .show_ui(ui, |ui| {
                for &v in VARIANTS {
                    let r = ui.selectable_label(cur == v, v);
                    if r.clicked() {
                        *val = val.cast(v);
                        *changed = true;
                    } else if r.hovered() {
                        hover = Some(val.cast(v));
                    }
                }
            });
        if resp.inner.is_some() {
            is_open = true;
        }
        if let Some(n) = val.num() {
            let mut n = n;
            let (lo, hi) = if matches!(val, ValCfg::Px(_)) {
                (0.0_f32, 600.0_f32)
            } else {
                (0.0_f32, 100.0_f32)
            };
            if ui
                .add(egui::Slider::new(&mut n, lo..=hi).max_decimals(0))
                .changed()
            {
                val.set_num(n);
                *changed = true;
            }
        }
    });
    if is_open {
        *popup_open = true;
    }
    hover
}

// ─── Item info ────────────────────────────────────────────────────────────────

fn item_info(idx: usize, item: &ItemCfg) -> String {
    format!(
        "#{} g:{} s:{}\nbasis:{} w:{} h:{}",
        idx + 1,
        ff(item.flex_grow),
        ff(item.flex_shrink),
        item.flex_basis.variant(),
        fv(&item.width),
        fv(&item.height)
    )
}

fn ff(v: f32) -> String {
    if (v - v.round()).abs() < 0.005 {
        format!("{}", v as i32)
    } else {
        format!("{v:.1}")
    }
}

fn fv(v: &ValCfg) -> String {
    match v {
        ValCfg::Auto => "auto".into(),
        ValCfg::Px(n) => format!("{:.0}px", n),
        ValCfg::Percent(n) => format!("{:.0}%", n),
        ValCfg::Vw(n) => format!("{:.0}vw", n),
        ValCfg::Vh(n) => format!("{:.0}vh", n),
    }
}
