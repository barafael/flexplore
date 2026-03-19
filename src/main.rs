//! Flexplore — interactive Bevy 0.18 flexbox explorer.

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use rand::{Rng, SeedableRng, rngs::StdRng};

const PANEL_W: f32 = 390.0;
const ART_SZ: u32 = 128;

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

// ─── Node config (recursive tree) ────────────────────────────────────────────

#[derive(Clone)]
struct NodeCfg {
    label: String,
    // flex container props (how children are arranged)
    flex_direction: FlexDirection,
    flex_wrap: FlexWrap,
    justify_content: JustifyContent,
    align_items: AlignItems,
    align_content: AlignContent,
    row_gap: ValCfg,
    column_gap: ValCfg,
    // flex item + sizing props (how this node fits in its parent)
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
    // children
    children: Vec<NodeCfg>,
}

impl NodeCfg {
    fn new_leaf(label: impl Into<String>, w: f32, h: f32) -> Self {
        Self {
            label: label.into(),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            align_content: AlignContent::FlexStart,
            row_gap: ValCfg::Px(4.0),
            column_gap: ValCfg::Px(4.0),
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
            children: vec![],
        }
    }

    fn new_container(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::FlexStart,
            row_gap: ValCfg::Px(8.0),
            column_gap: ValCfg::Px(8.0),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_basis: ValCfg::Auto,
            align_self: AlignSelf::Auto,
            width: ValCfg::Percent(100.0),
            height: ValCfg::Auto,
            min_width: ValCfg::Auto,
            min_height: ValCfg::Px(0.0),
            max_width: ValCfg::Auto,
            max_height: ValCfg::Auto,
            padding: ValCfg::Px(12.0),
            margin: ValCfg::Px(0.0),
            children: vec![],
        }
    }
}

fn get_node<'a>(root: &'a NodeCfg, path: &[usize]) -> &'a NodeCfg {
    if path.is_empty() { root } else { get_node(&root.children[path[0]], &path[1..]) }
}

fn get_node_mut<'a>(root: &'a mut NodeCfg, path: &[usize]) -> &'a mut NodeCfg {
    if path.is_empty() { root } else { get_node_mut(&mut root.children[path[0]], &path[1..]) }
}

fn path_valid(root: &NodeCfg, path: &[usize]) -> bool {
    if path.is_empty() { return true; }
    if path[0] >= root.children.len() { return false; }
    path_valid(&root.children[path[0]], &path[1..])
}

fn count_leaves(node: &NodeCfg) -> usize {
    if node.children.is_empty() { 1 } else { node.children.iter().map(count_leaves).sum() }
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
    root: NodeCfg,
    selected: Vec<usize>, // path to selected node; empty = root
    bg_mode: BgMode,
    art_style: ArtStyle,
    art_seed: u64,
    art_depth: u32,
    art_anim: f32,
    needs_rebuild: bool,
}

impl Default for FlexCfg {
    fn default() -> Self {
        let mut root = NodeCfg::new_container("root");
        root.min_height = ValCfg::Px(200.0);
        root.children = vec![
            NodeCfg::new_leaf("A", 80.0, 80.0),
            NodeCfg::new_leaf("B", 120.0, 100.0),
            NodeCfg::new_leaf("C", 60.0, 60.0),
            NodeCfg::new_leaf("D", 100.0, 80.0),
        ];
        Self {
            root,
            selected: vec![],
            bg_mode: BgMode::Pastel,
            art_style: ArtStyle::ExprTree,
            art_seed: 137,
            art_depth: 5,
            art_anim: 0.0,
            needs_rebuild: true,
        }
    }
}

// ─── RandomFart expression tree ───────────────────────────────────────────────

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
        if roll < ends[1] { return Expr::Add(b(rng, depth - 1), b(rng, depth - 1)); }
        if roll < ends[2] { return Expr::Mult(b(rng, depth - 1), b(rng, depth - 1)); }
        if roll < ends[3] { return Expr::Sqrt(Box::new(Expr::Abs(b(rng, depth - 1)))); }
        if roll < ends[4] { return Expr::Sin(b(rng, depth - 1)); }
        if roll < ends[5] { return Expr::Mod(b(rng, depth - 1), b(rng, depth - 1)); }
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

struct ArtExprs { r: Expr, g: Expr, b: Expr }
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

// ─── CPU generative backgrounds ───────────────────────────────────────────────

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
        hash2(ix, iy, seed), hash2(ix + 1, iy, seed),
        hash2(ix, iy + 1, seed), hash2(ix + 1, iy + 1, seed),
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
                if d < d1 { d2 = d1; d1 = d; cid = (cx as u64).wrapping_mul(1000003).wrapping_add(cy as u64); }
                else if d < d2 { d2 = d; }
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
    for ch in pix.chunks_mut(4) { ch[0] = 225; ch[1] = 235; ch[2] = 250; ch[3] = 255; }
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
                        pix[idx] = lr; pix[idx + 1] = lg; pix[idx + 2] = lb;
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
                if d < d1 { d2 = d1; d1 = d; } else if d < d2 { d2 = d; }
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
    let warp  = 0.3 + snoise(0.2, 0.3, seed) * 0.4;
    let freq  = 2.0 + snoise(0.3, 0.4, seed) * 2.0;
    let twist = snoise(0.4, 0.5, seed) * 1.5;
    let fg = [hash2(seed as i32, 0, seed), hash2(seed as i32, 1, seed), hash2(seed as i32, 2, seed)];
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
        ArtStyle::ExprTree  => exprs.render(ART_SZ, ART_SZ, t),
        ArtStyle::Voronoi   => render_voronoi(ART_SZ, ART_SZ, seed, t),
        ArtStyle::FlowField => render_flow_field(ART_SZ, ART_SZ, seed, t),
        ArtStyle::Crackle   => render_crackle(ART_SZ, ART_SZ, seed, t),
        ArtStyle::OpArt     => render_op_art(ART_SZ, ART_SZ, seed, t),
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
    (1.00, 0.80, 0.80), (0.80, 1.00, 0.82), (0.82, 0.86, 1.00),
    (1.00, 1.00, 0.80), (1.00, 0.90, 0.80), (0.80, 0.96, 1.00),
    (0.94, 0.82, 1.00), (0.82, 0.94, 0.82), (1.00, 0.86, 0.94),
    (0.88, 0.96, 1.00), (0.94, 1.00, 0.88), (1.00, 0.94, 0.86),
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
                    title: "Flexplore".into(),
                    fit_canvas_to_parent: true,
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

// ─── Tree UI helper ───────────────────────────────────────────────────────────

/// Returns (clicked_path, remove_requested)
fn draw_tree_ui(
    ui: &mut egui::Ui,
    node: &mut NodeCfg,
    path: &mut Vec<usize>,
    selected: &[usize],
    changed: &mut bool,
) -> (Option<Vec<usize>>, bool) {
    let mut clicked = None;
    let mut remove = false;
    let is_selected = path.as_slice() == selected;
    let is_root = path.is_empty();
    ui.horizontal(|ui| {
        ui.add_space(path.len() as f32 * 14.0);
        let icon = if node.children.is_empty() { "□" } else { "▣" };
        if is_selected {
            let _ = ui.selectable_label(true, icon);
            let r = ui.add(egui::TextEdit::singleline(&mut node.label).desired_width(80.0));
            if r.changed() { *changed = true; }
            if !is_root && ui.small_button("x").clicked() {
                remove = true;
            }
        } else if ui.selectable_label(false, format!("{} {}", icon, node.label)).clicked() {
            clicked = Some(path.clone());
        }
    });
    for i in 0..node.children.len() {
        path.push(i);
        let (r, rem) = draw_tree_ui(ui, &mut node.children[i], path, selected, changed);
        path.pop();
        if r.is_some() { clicked = r; }
        if rem { remove = true; }
    }
    (clicked, remove)
}

/// Estimate a text scale factor from a node's configured dimensions.
/// Returns a multiplier relative to the "base" 80px node size, capped at 2x.
fn text_scale(node: &NodeCfg) -> f32 {
    fn approx_px(v: &ValCfg) -> Option<f32> {
        match v {
            ValCfg::Px(n) => Some(*n),
            ValCfg::Percent(n) => Some(n / 100.0 * 600.0), // assume ~600px reference
            ValCfg::Vw(n) | ValCfg::Vh(n) => Some(n / 100.0 * 800.0),
            ValCfg::Auto => None,
        }
    }
    let w = approx_px(&node.width);
    let h = approx_px(&node.height);
    let min_dim = match (w, h) {
        (Some(w), Some(h)) => w.min(h),
        (Some(v), None) | (None, Some(v)) => v,
        (None, None) => 80.0,
    };
    (min_dim / 80.0).clamp(0.25, 2.0)
}

// ─── Panel helpers ────────────────────────────────────────────────────────────

// ─── Code generation ─────────────────────────────────────────────────────────

fn emit_val(v: &ValCfg) -> String {
    match v {
        ValCfg::Auto => "Val::Auto".into(),
        ValCfg::Px(n) => format!("Val::Px({n:.1})"),
        ValCfg::Percent(n) => format!("Val::Percent({n:.1})"),
        ValCfg::Vw(n) => format!("Val::Vw({n:.1})"),
        ValCfg::Vh(n) => format!("Val::Vh({n:.1})"),
    }
}

fn emit_bevy_code(root: &NodeCfg) -> String {
    let mut buf = String::from("fn spawn_ui(commands: &mut Commands) {\n");
    emit_node(&mut buf, root, 1, &mut 0, true);
    buf.push_str("}\n");
    buf
}

fn emit_node(buf: &mut String, node: &NodeCfg, depth: usize, leaf_idx: &mut usize, is_root: bool) {
    let pad = "    ".repeat(depth);
    let is_leaf = node.children.is_empty();

    let bg = if is_leaf {
        let (r, g, b) = PASTELS[*leaf_idx % PASTELS.len()];
        *leaf_idx += 1;
        format!("Color::srgb({r:.2}, {g:.2}, {b:.2})")
    } else {
        "Color::srgba(0.11, 0.11, 0.17, 1.0)".into()
    };

    let spawner = if is_root { "commands" } else { "parent" };
    buf.push_str(&format!("{pad}// {}\n", node.label));
    buf.push_str(&format!("{pad}{spawner}.spawn((\n"));

    // Node component
    buf.push_str(&format!("{pad}    Node {{\n"));
    buf.push_str(&format!("{pad}        display: Display::Flex,\n"));
    emit_field(buf, &pad, "flex_direction", &format!("FlexDirection::{:?}", node.flex_direction));
    emit_field(buf, &pad, "flex_wrap", &format!("FlexWrap::{:?}", node.flex_wrap));
    emit_field(buf, &pad, "justify_content", &format!("JustifyContent::{:?}", node.justify_content));
    emit_field(buf, &pad, "align_items", &format!("AlignItems::{:?}", node.align_items));
    emit_field(buf, &pad, "align_content", &format!("AlignContent::{:?}", node.align_content));
    emit_field(buf, &pad, "row_gap", &emit_val(&node.row_gap));
    emit_field(buf, &pad, "column_gap", &emit_val(&node.column_gap));
    emit_field(buf, &pad, "flex_grow", &format!("{:.1}", node.flex_grow));
    emit_field(buf, &pad, "flex_shrink", &format!("{:.1}", node.flex_shrink));
    emit_field(buf, &pad, "flex_basis", &emit_val(&node.flex_basis));
    emit_field(buf, &pad, "align_self", &format!("AlignSelf::{:?}", node.align_self));
    emit_field(buf, &pad, "width", &emit_val(&node.width));
    emit_field(buf, &pad, "height", &emit_val(&node.height));
    emit_field(buf, &pad, "min_width", &emit_val(&node.min_width));
    emit_field(buf, &pad, "min_height", &emit_val(&node.min_height));
    emit_field(buf, &pad, "max_width", &emit_val(&node.max_width));
    emit_field(buf, &pad, "max_height", &emit_val(&node.max_height));
    emit_field(buf, &pad, "padding", &format!("UiRect::all({})", emit_val(&node.padding)));
    emit_field(buf, &pad, "margin", &format!("UiRect::all({})", emit_val(&node.margin)));
    buf.push_str(&format!("{pad}        ..default()\n"));
    buf.push_str(&format!("{pad}    }},\n"));

    // BackgroundColor
    buf.push_str(&format!("{pad}    BackgroundColor({bg}),\n"));
    buf.push_str(&format!("{pad}))"));

    if is_leaf {
        // Leaf: spawn with text children
        buf.push_str(".with_children(|parent| {\n");
        buf.push_str(&format!("{pad}    parent.spawn((\n"));
        buf.push_str(&format!("{pad}        Text::new({:?}),\n", node.label));
        buf.push_str(&format!("{pad}        TextFont {{ font_size: 26.0, ..default() }},\n"));
        buf.push_str(&format!("{pad}        TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),\n"));
        buf.push_str(&format!("{pad}    ));\n"));
        buf.push_str(&format!("{pad}}});\n"));
    } else if node.children.is_empty() {
        buf.push_str(";\n");
    } else {
        buf.push_str(".with_children(|parent| {\n");
        for child in &node.children {
            emit_node(buf, child, depth + 1, leaf_idx, false);
        }
        buf.push_str(&format!("{pad}}});\n"));
    }
}

fn emit_field(buf: &mut String, pad: &str, name: &str, value: &str) {
    buf.push_str(&format!("{pad}        {name}: {value},\n"));
}

// ─── HTML/CSS code generation ────────────────────────────────────────────────

fn emit_css_val(v: &ValCfg) -> String {
    match v {
        ValCfg::Auto => "auto".into(),
        ValCfg::Px(n) => format!("{n:.1}px"),
        ValCfg::Percent(n) => format!("{n:.1}%"),
        ValCfg::Vw(n) => format!("{n:.1}vw"),
        ValCfg::Vh(n) => format!("{n:.1}vh"),
    }
}

fn css_flex_direction(d: FlexDirection) -> &'static str {
    match d {
        FlexDirection::Row => "row",
        FlexDirection::Column => "column",
        FlexDirection::RowReverse => "row-reverse",
        FlexDirection::ColumnReverse => "column-reverse",
    }
}

fn css_flex_wrap(w: FlexWrap) -> &'static str {
    match w {
        FlexWrap::NoWrap => "nowrap",
        FlexWrap::Wrap => "wrap",
        FlexWrap::WrapReverse => "wrap-reverse",
    }
}

fn css_justify_content(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::FlexStart => "flex-start",
        JustifyContent::FlexEnd => "flex-end",
        JustifyContent::Center => "center",
        JustifyContent::SpaceBetween => "space-between",
        JustifyContent::SpaceAround => "space-around",
        JustifyContent::SpaceEvenly => "space-evenly",
        _ => "flex-start",
    }
}

fn css_align_items(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => "flex-start",
        AlignItems::FlexEnd => "flex-end",
        AlignItems::Center => "center",
        AlignItems::Baseline => "baseline",
        AlignItems::Stretch => "stretch",
        _ => "stretch",
    }
}

fn css_align_content(a: AlignContent) -> &'static str {
    match a {
        AlignContent::FlexStart => "flex-start",
        AlignContent::FlexEnd => "flex-end",
        AlignContent::Center => "center",
        AlignContent::SpaceBetween => "space-between",
        AlignContent::SpaceAround => "space-around",
        AlignContent::SpaceEvenly => "space-evenly",
        AlignContent::Stretch => "stretch",
        _ => "stretch",
    }
}

fn css_align_self(a: AlignSelf) -> &'static str {
    match a {
        AlignSelf::Auto => "auto",
        AlignSelf::FlexStart => "flex-start",
        AlignSelf::FlexEnd => "flex-end",
        AlignSelf::Center => "center",
        AlignSelf::Baseline => "baseline",
        AlignSelf::Stretch => "stretch",
        _ => "auto",
    }
}

fn emit_html_css(root: &NodeCfg) -> String {
    let mut css = String::new();
    let mut html = String::new();
    emit_html_node(&mut css, &mut html, root, 0, &mut 0, &mut 0);
    format!(
        "<style>\n{css}</style>\n\n{html}"
    )
}

fn emit_html_node(
    css: &mut String,
    html: &mut String,
    node: &NodeCfg,
    depth: usize,
    leaf_idx: &mut usize,
    id_counter: &mut usize,
) {
    let id = *id_counter;
    *id_counter += 1;
    let is_leaf = node.children.is_empty();
    let pad_html = "  ".repeat(depth);
    let class = format!("node-{id}");

    // CSS
    let bg = if is_leaf {
        let (r, g, b) = PASTELS[*leaf_idx % PASTELS.len()];
        *leaf_idx += 1;
        format!(
            "rgb({}, {}, {})",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )
    } else {
        "rgba(28, 28, 43, 1)".into()
    };

    css.push_str(&format!(".{class} {{\n"));
    css.push_str("  display: flex;\n");
    css.push_str(&format!("  flex-direction: {};\n", css_flex_direction(node.flex_direction)));
    css.push_str(&format!("  flex-wrap: {};\n", css_flex_wrap(node.flex_wrap)));
    css.push_str(&format!("  justify-content: {};\n", css_justify_content(node.justify_content)));
    css.push_str(&format!("  align-items: {};\n", css_align_items(node.align_items)));
    css.push_str(&format!("  align-content: {};\n", css_align_content(node.align_content)));
    css.push_str(&format!("  row-gap: {};\n", emit_css_val(&node.row_gap)));
    css.push_str(&format!("  column-gap: {};\n", emit_css_val(&node.column_gap)));
    css.push_str(&format!("  flex-grow: {:.1};\n", node.flex_grow));
    css.push_str(&format!("  flex-shrink: {:.1};\n", node.flex_shrink));
    css.push_str(&format!("  flex-basis: {};\n", emit_css_val(&node.flex_basis)));
    css.push_str(&format!("  align-self: {};\n", css_align_self(node.align_self)));
    css.push_str(&format!("  width: {};\n", emit_css_val(&node.width)));
    css.push_str(&format!("  height: {};\n", emit_css_val(&node.height)));
    css.push_str(&format!("  min-width: {};\n", emit_css_val(&node.min_width)));
    css.push_str(&format!("  min-height: {};\n", emit_css_val(&node.min_height)));
    css.push_str(&format!("  max-width: {};\n", emit_css_val(&node.max_width)));
    css.push_str(&format!("  max-height: {};\n", emit_css_val(&node.max_height)));
    css.push_str(&format!("  padding: {};\n", emit_css_val(&node.padding)));
    css.push_str(&format!("  margin: {};\n", emit_css_val(&node.margin)));
    css.push_str(&format!("  background: {bg};\n"));
    css.push_str(&format!("  box-sizing: border-box;\n"));
    if is_leaf {
        css.push_str("  color: rgba(13, 13, 26, 0.85);\n");
        css.push_str("  font-size: 26px;\n");
    }
    css.push_str("}\n\n");

    // HTML
    if is_leaf {
        html.push_str(&format!(
            "{pad_html}<div class=\"{class}\">{}</div>\n",
            node.label
        ));
    } else {
        html.push_str(&format!("{pad_html}<div class=\"{class}\">\n"));
        for child in &node.children {
            emit_html_node(css, html, child, depth + 1, leaf_idx, id_counter);
        }
        html.push_str(&format!("{pad_html}</div>\n"));
    }
}

// ─── Tailwind HTML code generation ───────────────────────────────────────────

fn tw_flex_direction(d: FlexDirection) -> &'static str {
    match d {
        FlexDirection::Row => "flex-row",
        FlexDirection::Column => "flex-col",
        FlexDirection::RowReverse => "flex-row-reverse",
        FlexDirection::ColumnReverse => "flex-col-reverse",
    }
}

fn tw_flex_wrap(w: FlexWrap) -> &'static str {
    match w {
        FlexWrap::NoWrap => "flex-nowrap",
        FlexWrap::Wrap => "flex-wrap",
        FlexWrap::WrapReverse => "flex-wrap-reverse",
    }
}

fn tw_justify_content(j: JustifyContent) -> &'static str {
    match j {
        JustifyContent::FlexStart => "justify-start",
        JustifyContent::FlexEnd => "justify-end",
        JustifyContent::Center => "justify-center",
        JustifyContent::SpaceBetween => "justify-between",
        JustifyContent::SpaceAround => "justify-around",
        JustifyContent::SpaceEvenly => "justify-evenly",
        _ => "justify-start",
    }
}

fn tw_align_items(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => "items-start",
        AlignItems::FlexEnd => "items-end",
        AlignItems::Center => "items-center",
        AlignItems::Baseline => "items-baseline",
        AlignItems::Stretch => "items-stretch",
        _ => "items-stretch",
    }
}

fn tw_align_content(a: AlignContent) -> &'static str {
    match a {
        AlignContent::FlexStart => "content-start",
        AlignContent::FlexEnd => "content-end",
        AlignContent::Center => "content-center",
        AlignContent::SpaceBetween => "content-between",
        AlignContent::SpaceAround => "content-around",
        AlignContent::SpaceEvenly => "content-evenly",
        AlignContent::Stretch => "content-stretch",
        _ => "content-stretch",
    }
}

fn tw_align_self(a: AlignSelf) -> &'static str {
    match a {
        AlignSelf::Auto => "self-auto",
        AlignSelf::FlexStart => "self-start",
        AlignSelf::FlexEnd => "self-end",
        AlignSelf::Center => "self-center",
        AlignSelf::Baseline => "self-baseline",
        AlignSelf::Stretch => "self-stretch",
        _ => "self-auto",
    }
}

fn tw_val(property: &str, v: &ValCfg) -> String {
    match v {
        ValCfg::Auto => format!("{property}-auto"),
        ValCfg::Px(n) => format!("{property}-[{n:.1}px]"),
        ValCfg::Percent(n) => format!("{property}-[{n:.1}%]"),
        ValCfg::Vw(n) => format!("{property}-[{n:.1}vw]"),
        ValCfg::Vh(n) => format!("{property}-[{n:.1}vh]"),
    }
}

fn emit_tailwind(root: &NodeCfg) -> String {
    let mut buf = String::new();
    emit_tailwind_node(&mut buf, root, 0, &mut 0);
    buf
}

fn emit_tailwind_node(buf: &mut String, node: &NodeCfg, depth: usize, leaf_idx: &mut usize) {
    let pad = "  ".repeat(depth);
    let is_leaf = node.children.is_empty();

    let bg = if is_leaf {
        let (r, g, b) = PASTELS[*leaf_idx % PASTELS.len()];
        *leaf_idx += 1;
        format!(
            "bg-[rgb({},{},{})]",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )
    } else {
        "bg-[rgba(28,28,43,1)]".into()
    };

    let mut classes = vec![
        "flex".into(),
        tw_flex_direction(node.flex_direction).into(),
        tw_flex_wrap(node.flex_wrap).into(),
        tw_justify_content(node.justify_content).into(),
        tw_align_items(node.align_items).into(),
        tw_align_content(node.align_content).into(),
        tw_val("gap-x", &node.column_gap),
        tw_val("gap-y", &node.row_gap),
        format!("grow-[{:.1}]", node.flex_grow),
        format!("shrink-[{:.1}]", node.flex_shrink),
        tw_val("basis", &node.flex_basis),
        tw_align_self(node.align_self).into(),
        tw_val("w", &node.width),
        tw_val("h", &node.height),
        tw_val("min-w", &node.min_width),
        tw_val("min-h", &node.min_height),
        tw_val("max-w", &node.max_width),
        tw_val("max-h", &node.max_height),
        tw_val("p", &node.padding),
        tw_val("m", &node.margin),
        bg,
        "box-border".into(),
    ];

    if is_leaf {
        classes.push("text-[26px]".into());
        classes.push("text-[rgba(13,13,26,0.85)]".into());
    }

    let cls = classes.join(" ");

    if is_leaf {
        buf.push_str(&format!("{pad}<div class=\"{cls}\">{}</div>\n", node.label));
    } else {
        buf.push_str(&format!("{pad}<div class=\"{cls}\">\n"));
        for child in &node.children {
            emit_tailwind_node(buf, child, depth + 1, leaf_idx);
        }
        buf.push_str(&format!("{pad}</div>\n"));
    }
}

// ─── SwiftUI code generation ─────────────────────────────────────────────────

fn swift_val(v: &ValCfg) -> String {
    match v {
        ValCfg::Auto => ".infinity".into(),
        ValCfg::Px(n) => format!("{n:.1}"),
        ValCfg::Percent(n) => format!("{n:.1} /* {n:.1}% — use GeometryReader for relative sizing */"),
        ValCfg::Vw(n) => format!("UIScreen.main.bounds.width * {:.3}", n / 100.0),
        ValCfg::Vh(n) => format!("UIScreen.main.bounds.height * {:.3}", n / 100.0),
    }
}

fn swift_optional_val(v: &ValCfg) -> Option<String> {
    match v {
        ValCfg::Auto => None,
        _ => Some(swift_val(v)),
    }
}

fn swift_alignment(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => ".top",
        AlignItems::FlexEnd => ".bottom",
        AlignItems::Center => ".center",
        AlignItems::Baseline => ".firstTextBaseline",
        AlignItems::Stretch => ".center",
        _ => ".center",
    }
}

fn swift_h_alignment(a: AlignItems) -> &'static str {
    match a {
        AlignItems::FlexStart => ".leading",
        AlignItems::FlexEnd => ".trailing",
        AlignItems::Center => ".center",
        _ => ".center",
    }
}

fn emit_swiftui(root: &NodeCfg) -> String {
    let mut buf = String::from("struct ContentView: View {\n    var body: some View {\n");
    emit_swiftui_node(&mut buf, root, 2, &mut 0);
    buf.push_str("    }\n}\n");
    buf
}

fn emit_swiftui_node(buf: &mut String, node: &NodeCfg, depth: usize, leaf_idx: &mut usize) {
    let pad = "    ".repeat(depth);
    let is_leaf = node.children.is_empty();

    if is_leaf {
        let (r, g, b) = PASTELS[*leaf_idx % PASTELS.len()];
        *leaf_idx += 1;

        buf.push_str(&format!("{pad}Text({:?})\n", node.label));
        buf.push_str(&format!("{pad}    .font(.system(size: 26))\n"));
        buf.push_str(&format!(
            "{pad}    .foregroundColor(Color(red: 0.05, green: 0.05, blue: 0.1).opacity(0.85))\n"
        ));

        // Frame
        let w = swift_optional_val(&node.width);
        let h = swift_optional_val(&node.height);
        if w.is_some() || h.is_some() {
            let w_str = w.as_deref().unwrap_or("nil");
            let h_str = h.as_deref().unwrap_or("nil");
            buf.push_str(&format!("{pad}    .frame(width: {w_str}, height: {h_str})\n"));
        }
        // Min/max frame
        let min_w = swift_optional_val(&node.min_width);
        let min_h = swift_optional_val(&node.min_height);
        let max_w = swift_optional_val(&node.max_width);
        let max_h = swift_optional_val(&node.max_height);
        if min_w.is_some() || min_h.is_some() || max_w.is_some() || max_h.is_some() {
            buf.push_str(&format!(
                "{pad}    .frame(minWidth: {}, minHeight: {}, maxWidth: {}, maxHeight: {})\n",
                min_w.as_deref().unwrap_or("nil"),
                min_h.as_deref().unwrap_or("nil"),
                max_w.as_deref().unwrap_or("nil"),
                max_h.as_deref().unwrap_or("nil"),
            ));
        }
        if let Some(p) = swift_optional_val(&node.padding) {
            buf.push_str(&format!("{pad}    .padding({p})\n"));
        }
        buf.push_str(&format!(
            "{pad}    .background(Color(red: {r:.2}, green: {g:.2}, blue: {b:.2}))\n"
        ));
    } else {
        // Container: choose HStack or VStack based on flex_direction
        let is_row = matches!(
            node.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );

        let spacing = match &node.column_gap {
            ValCfg::Px(n) if is_row => format!(", spacing: {n:.1}"),
            _ => match &node.row_gap {
                ValCfg::Px(n) if !is_row => format!(", spacing: {n:.1}"),
                _ => String::new(),
            },
        };

        let alignment = if is_row {
            swift_alignment(node.align_items)
        } else {
            swift_h_alignment(node.align_items)
        };

        let stack = if is_row { "HStack" } else { "VStack" };
        buf.push_str(&format!("{pad}{stack}(alignment: {alignment}{spacing}) {{\n"));

        for child in &node.children {
            emit_swiftui_node(buf, child, depth + 1, leaf_idx);
        }

        buf.push_str(&format!("{pad}}}\n"));

        // Frame modifiers on the container
        let w = swift_optional_val(&node.width);
        let h = swift_optional_val(&node.height);
        if w.is_some() || h.is_some() {
            let w_str = w.as_deref().unwrap_or("nil");
            let h_str = h.as_deref().unwrap_or("nil");
            buf.push_str(&format!("{pad}.frame(width: {w_str}, height: {h_str})\n"));
        }
        let min_w = swift_optional_val(&node.min_width);
        let min_h = swift_optional_val(&node.min_height);
        let max_w = swift_optional_val(&node.max_width);
        let max_h = swift_optional_val(&node.max_height);
        if min_w.is_some() || min_h.is_some() || max_w.is_some() || max_h.is_some() {
            buf.push_str(&format!(
                "{pad}.frame(minWidth: {}, minHeight: {}, maxWidth: {}, maxHeight: {})\n",
                min_w.as_deref().unwrap_or("nil"),
                min_h.as_deref().unwrap_or("nil"),
                max_w.as_deref().unwrap_or("nil"),
                max_h.as_deref().unwrap_or("nil"),
            ));
        }
        if let Some(p) = swift_optional_val(&node.padding) {
            buf.push_str(&format!("{pad}.padding({p})\n"));
        }
        buf.push_str(&format!(
            "{pad}.background(Color(red: 0.11, green: 0.11, blue: 0.17))\n"
        ));
    }
}

fn apply_hover<T: PartialEq + Clone>(
    opt: Option<T>,
    cfg: &mut FlexCfg,
    preview: &mut Option<FlexCfg>,
    path: &[usize],
    get: impl Fn(&NodeCfg) -> T,
    set: impl FnOnce(&mut NodeCfg, T),
) -> bool {
    let Some(v) = opt else { return false };
    if get(get_node(&cfg.root, path)) != v {
        if preview.is_none() { *preview = Some(cfg.clone()); }
        set(get_node_mut(&mut cfg.root, path), v);
        true
    } else {
        false
    }
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
        const BG:  egui::Color32 = egui::Color32::from_rgb(0x10, 0x10, 0x14);
        const MID: egui::Color32 = egui::Color32::from_rgb(0x2a, 0x2a, 0x30);
        const FG:  egui::Color32 = egui::Color32::from_rgb(0xe8, 0xe4, 0xd8);
        let mut v = egui::Visuals::dark();
        v.panel_fill = BG; v.window_fill = BG; v.extreme_bg_color = BG;
        v.widgets.inactive.bg_fill      = MID;
        v.widgets.inactive.weak_bg_fill = MID;
        v.widgets.inactive.bg_stroke    = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x3a, 0x3a, 0x42));
        v.widgets.inactive.fg_stroke    = egui::Stroke::new(1.0, FG);
        v.widgets.hovered.bg_fill       = egui::Color32::from_rgb(0x38, 0x38, 0x42);
        v.widgets.hovered.weak_bg_fill  = egui::Color32::from_rgb(0x38, 0x38, 0x42);
        v.widgets.hovered.bg_stroke     = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x88, 0x88, 0x98));
        v.widgets.hovered.fg_stroke     = egui::Stroke::new(1.5, FG);
        v.widgets.active.bg_fill        = FG;
        v.widgets.active.weak_bg_fill   = FG;
        v.widgets.active.fg_stroke      = egui::Stroke::new(1.5, BG);
        v.widgets.open.bg_fill          = MID;
        v.widgets.open.fg_stroke        = egui::Stroke::new(1.0, FG);
        v.widgets.noninteractive.bg_fill    = BG;
        v.widgets.noninteractive.fg_stroke  = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x70, 0x6e, 0x66));
        v.widgets.noninteractive.bg_stroke  = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x34, 0x34, 0x3a));
        v.override_text_color = Some(FG);
        v.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x3a, 0x3a, 0x42));
        v.selection.bg_fill = egui::Color32::from_rgb(0x40, 0x40, 0x52);
        let no_rounding = egui::CornerRadius::ZERO;
        v.window_corner_radius = no_rounding;
        v.menu_corner_radius = no_rounding;
        v.widgets.inactive.corner_radius = no_rounding;
        v.widgets.hovered.corner_radius = no_rounding;
        v.widgets.active.corner_radius = no_rounding;
        v.widgets.open.corner_radius = no_rounding;
        v.widgets.noninteractive.corner_radius = no_rounding;
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

    // Hover vars collected inside borrow blocks, applied after the block ends.
    let mut hc_dir: Option<FlexDirection> = None;
    let mut hc_wrap: Option<FlexWrap> = None;
    let mut hc_justify: Option<JustifyContent> = None;
    let mut hc_ai: Option<AlignItems> = None;
    let mut hc_ac: Option<AlignContent> = None;
    let mut hc_rg: Option<ValCfg> = None;
    let mut hc_cgap: Option<ValCfg> = None;
    let mut hs_w: Option<ValCfg> = None;
    let mut hs_h: Option<ValCfg> = None;
    let mut hs_minw: Option<ValCfg> = None;
    let mut hs_minh: Option<ValCfg> = None;
    let mut hs_maxw: Option<ValCfg> = None;
    let mut hs_maxh: Option<ValCfg> = None;
    let mut hs_pad: Option<ValCfg> = None;
    let mut hi_basis: Option<ValCfg> = None;
    let mut hi_as: Option<AlignSelf> = None;
    let mut hi_margin: Option<ValCfg> = None;

    let mut sel_path = cfg.selected.clone();
    let mut is_root = sel_path.is_empty();

    egui::SidePanel::left("flex_panel")
        .exact_width(PANEL_W)
        .resizable(false)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);

                // ── Tree ─────────────────────────────────────────────────────────
                egui::CollapsingHeader::new("Tree")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("+ Child").clicked() {
                                let n = count_leaves(&cfg.root);
                                let lbl = format!("node{}", n + 1);
                                get_node_mut(&mut cfg.root, &sel_path)
                                    .children.push(NodeCfg::new_leaf(&lbl, 80.0, 80.0));
                                changed = true;
                            }
                            if !is_root && ui.button("+ Sibling").clicked() {
                                let pidx = sel_path.len() - 1;
                                let n = count_leaves(&cfg.root);
                                let lbl = format!("node{}", n + 1);
                                get_node_mut(&mut cfg.root, &sel_path[..pidx])
                                    .children.push(NodeCfg::new_leaf(&lbl, 80.0, 80.0));
                                changed = true;
                            }
                        });
                        ui.add_space(2.0);
                        let sel_snapshot = cfg.selected.clone();
                        let (clicked, remove_req) = draw_tree_ui(ui, &mut cfg.root, &mut vec![], &sel_snapshot, &mut changed);
                        if remove_req && !sel_path.is_empty() {
                            let pidx = sel_path.len() - 1;
                            let idx = sel_path[pidx];
                            get_node_mut(&mut cfg.root, &sel_path[..pidx]).children.remove(idx);
                            let new_path = sel_path[..pidx].to_vec();
                            cfg.selected = new_path.clone();
                            sel_path = new_path;
                            is_root = sel_path.is_empty();
                            changed = true;
                        }
                        if let Some(p) = clicked
                            && p != cfg.selected {
                                cfg.selected = p.clone();
                                sel_path = p;
                                is_root = sel_path.is_empty();
                                *preview = None;
                            }
                    });

                ui.add_space(6.0);

                // ── Flex Container / Sizing / Item ───────────────────────────────
                // Guard: sel_path may have been invalidated this frame (remove/click)
                if path_valid(&cfg.root, &sel_path) {

                egui::CollapsingHeader::new("Flex Container")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        egui::Grid::new("cg1").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                            {
                                let n = get_node_mut(&mut cfg.root, &sel_path);
                                ui.label("direction");
                                hc_dir = combo(ui, "fd", &mut n.flex_direction, &[
                                    ("Row", FlexDirection::Row), ("Column", FlexDirection::Column),
                                    ("RowReverse", FlexDirection::RowReverse),
                                    ("ColumnReverse", FlexDirection::ColumnReverse),
                                ], &mut changed, &mut any_hovered); ui.end_row();

                                ui.label("wrap");
                                hc_wrap = combo(ui, "fw", &mut n.flex_wrap, &[
                                    ("NoWrap", FlexWrap::NoWrap), ("Wrap", FlexWrap::Wrap),
                                    ("WrapReverse", FlexWrap::WrapReverse),
                                ], &mut changed, &mut any_hovered); ui.end_row();

                                ui.label("justify");
                                hc_justify = combo(ui, "jc", &mut n.justify_content, &[
                                    ("Default", JustifyContent::Default),
                                    ("FlexStart", JustifyContent::FlexStart),
                                    ("FlexEnd", JustifyContent::FlexEnd),
                                    ("Center", JustifyContent::Center),
                                    ("SpaceBetween", JustifyContent::SpaceBetween),
                                    ("SpaceAround", JustifyContent::SpaceAround),
                                    ("SpaceEvenly", JustifyContent::SpaceEvenly),
                                    ("Stretch", JustifyContent::Stretch),
                                    ("Start", JustifyContent::Start), ("End", JustifyContent::End),
                                ], &mut changed, &mut any_hovered); ui.end_row();

                                ui.label("align-items");
                                hc_ai = combo(ui, "ai", &mut n.align_items, &[
                                    ("Default", AlignItems::Default),
                                    ("FlexStart", AlignItems::FlexStart),
                                    ("FlexEnd", AlignItems::FlexEnd),
                                    ("Center", AlignItems::Center),
                                    ("Baseline", AlignItems::Baseline),
                                    ("Stretch", AlignItems::Stretch),
                                    ("Start", AlignItems::Start), ("End", AlignItems::End),
                                ], &mut changed, &mut any_hovered); ui.end_row();

                                ui.label("align-content");
                                hc_ac = combo(ui, "ac", &mut n.align_content, &[
                                    ("Default", AlignContent::Default),
                                    ("FlexStart", AlignContent::FlexStart),
                                    ("FlexEnd", AlignContent::FlexEnd),
                                    ("Center", AlignContent::Center),
                                    ("SpaceBetween", AlignContent::SpaceBetween),
                                    ("SpaceAround", AlignContent::SpaceAround),
                                    ("SpaceEvenly", AlignContent::SpaceEvenly),
                                    ("Stretch", AlignContent::Stretch),
                                    ("Start", AlignContent::Start), ("End", AlignContent::End),
                                ], &mut changed, &mut any_hovered); ui.end_row();
                            }
                        });
                        ui.add_space(4.0); ui.separator(); ui.add_space(4.0);
                        egui::Grid::new("cg2").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                            {
                                let n = get_node_mut(&mut cfg.root, &sel_path);
                                ui.label("row-gap");
                                hc_rg = val_row(ui, "rg", &mut n.row_gap, &mut changed, &mut any_hovered);
                                ui.end_row();
                                ui.label("column-gap");
                                hc_cgap = val_row(ui, "cgap", &mut n.column_gap, &mut changed, &mut any_hovered);
                                ui.end_row();
                            }
                        });
                        ui.add_space(2.0);

                        // Apply container hover previews
                        let has_c = hc_dir.is_some() || hc_wrap.is_some() || hc_justify.is_some()
                            || hc_ai.is_some() || hc_ac.is_some()
                            || hc_rg.is_some() || hc_cgap.is_some();
                        if has_c {
                            any_hovered = true;
                            let p = &mut *preview; let sp = &sel_path;
                            let rb =
                                apply_hover(hc_dir,     &mut cfg, p, sp, |n| n.flex_direction,        |n, v| n.flex_direction  = v) |
                                apply_hover(hc_wrap,    &mut cfg, p, sp, |n| n.flex_wrap,              |n, v| n.flex_wrap        = v) |
                                apply_hover(hc_justify, &mut cfg, p, sp, |n| n.justify_content,        |n, v| n.justify_content  = v) |
                                apply_hover(hc_ai,      &mut cfg, p, sp, |n| n.align_items,            |n, v| n.align_items      = v) |
                                apply_hover(hc_ac,      &mut cfg, p, sp, |n| n.align_content,          |n, v| n.align_content    = v) |
                                apply_hover(hc_rg,      &mut cfg, p, sp, |n| n.row_gap.clone(),        |n, v| n.row_gap          = v) |
                                apply_hover(hc_cgap,    &mut cfg, p, sp, |n| n.column_gap.clone(),     |n, v| n.column_gap       = v);
                            if rb { cfg.needs_rebuild = true; }
                        }
                    });

                ui.add_space(6.0);

                // ── Sizing ────────────────────────────────────────────────────────
                egui::CollapsingHeader::new("Sizing")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        egui::Grid::new("sg").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                            {
                                let n = get_node_mut(&mut cfg.root, &sel_path);
                                ui.label("width");    hs_w    = val_row(ui, "sw",    &mut n.width,      &mut changed, &mut any_hovered); ui.end_row();
                                ui.label("height");   hs_h    = val_row(ui, "sh",    &mut n.height,     &mut changed, &mut any_hovered); ui.end_row();
                                ui.label("min-width");  hs_minw = val_row(ui, "sminw", &mut n.min_width,  &mut changed, &mut any_hovered); ui.end_row();
                                ui.label("min-height"); hs_minh = val_row(ui, "sminh", &mut n.min_height, &mut changed, &mut any_hovered); ui.end_row();
                                ui.label("max-width");  hs_maxw = val_row(ui, "smaxw", &mut n.max_width,  &mut changed, &mut any_hovered); ui.end_row();
                                ui.label("max-height"); hs_maxh = val_row(ui, "smaxh", &mut n.max_height, &mut changed, &mut any_hovered); ui.end_row();
                                ui.label("padding");    hs_pad  = val_row(ui, "spad",  &mut n.padding,    &mut changed, &mut any_hovered); ui.end_row();
                            }
                        });
                        ui.add_space(2.0);

                        // Apply sizing hover previews
                        let has_s = hs_w.is_some() || hs_h.is_some() || hs_minw.is_some()
                            || hs_minh.is_some() || hs_maxw.is_some() || hs_maxh.is_some()
                            || hs_pad.is_some();
                        if has_s {
                            any_hovered = true;
                            let p = &mut *preview; let sp = &sel_path;
                            let rb =
                                apply_hover(hs_w,    &mut cfg, p, sp, |n| n.width.clone(),      |n, v| n.width      = v) |
                                apply_hover(hs_h,    &mut cfg, p, sp, |n| n.height.clone(),     |n, v| n.height     = v) |
                                apply_hover(hs_minw, &mut cfg, p, sp, |n| n.min_width.clone(),  |n, v| n.min_width  = v) |
                                apply_hover(hs_minh, &mut cfg, p, sp, |n| n.min_height.clone(), |n, v| n.min_height = v) |
                                apply_hover(hs_maxw, &mut cfg, p, sp, |n| n.max_width.clone(),  |n, v| n.max_width  = v) |
                                apply_hover(hs_maxh, &mut cfg, p, sp, |n| n.max_height.clone(), |n, v| n.max_height = v) |
                                apply_hover(hs_pad,  &mut cfg, p, sp, |n| n.padding.clone(),    |n, v| n.padding    = v);
                            if rb { cfg.needs_rebuild = true; }
                        }
                    });

                ui.add_space(6.0);

                // ── Flex Item (hidden for root) ───────────────────────────────────
                if !is_root {
                    egui::CollapsingHeader::new("Flex Item")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            egui::Grid::new("ig").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                                {
                                    let n = get_node_mut(&mut cfg.root, &sel_path);
                                    ui.label("flex-grow");
                                    changed |= ui.add(egui::Slider::new(&mut n.flex_grow, 0.0..=5.0).max_decimals(2)).changed();
                                    ui.end_row();
                                    ui.label("flex-shrink");
                                    changed |= ui.add(egui::Slider::new(&mut n.flex_shrink, 0.0..=5.0).max_decimals(2)).changed();
                                    ui.end_row();
                                    ui.label("flex-basis");
                                    hi_basis = val_row(ui, "ib", &mut n.flex_basis, &mut changed, &mut any_hovered);
                                    ui.end_row();
                                    ui.label("align-self");
                                    hi_as = combo(ui, "ias", &mut n.align_self, &[
                                        ("Auto", AlignSelf::Auto), ("FlexStart", AlignSelf::FlexStart),
                                        ("FlexEnd", AlignSelf::FlexEnd), ("Center", AlignSelf::Center),
                                        ("Baseline", AlignSelf::Baseline), ("Stretch", AlignSelf::Stretch),
                                        ("Start", AlignSelf::Start), ("End", AlignSelf::End),
                                    ], &mut changed, &mut any_hovered);
                                    ui.end_row();
                                    ui.label("margin");
                                    hi_margin = val_row(ui, "im", &mut n.margin, &mut changed, &mut any_hovered);
                                    ui.end_row();
                                }
                            });
                            ui.add_space(2.0);

                            // Apply item hover previews
                            let has_i = hi_basis.is_some() || hi_as.is_some() || hi_margin.is_some();
                            if has_i {
                                any_hovered = true;
                                let p = &mut *preview; let sp = &sel_path;
                                let rb =
                                    apply_hover(hi_basis,  &mut cfg, p, sp, |n| n.flex_basis.clone(), |n, v| n.flex_basis = v) |
                                    apply_hover(hi_as,     &mut cfg, p, sp, |n| n.align_self,         |n, v| n.align_self = v) |
                                    apply_hover(hi_margin, &mut cfg, p, sp, |n| n.margin.clone(),     |n, v| n.margin     = v);
                                if rb { cfg.needs_rebuild = true; }
                            }
                        });

                    ui.add_space(6.0);
                } // end if !is_root

                } // end if path_valid

                // ── Background ────────────────────────────────────────────────────
                egui::CollapsingHeader::new("Background")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let prev = cfg.bg_mode.clone();
                            ui.radio_value(&mut cfg.bg_mode, BgMode::Pastel, "Pastel");
                            ui.radio_value(&mut cfg.bg_mode, BgMode::RandomArt, "Generative Art");
                            if cfg.bg_mode != prev { changed = true; }
                        });
                        if cfg.bg_mode == BgMode::RandomArt {
                            let cur = ArtStyle::ALL.iter().find(|(_, s)| *s == cfg.art_style)
                                .map(|(n, _)| *n).unwrap_or("?");
                            let mut hover_art: Option<ArtStyle> = None;
                            let art_resp = egui::ComboBox::from_label("style")
                                .selected_text(cur)
                                .show_ui(ui, |ui| {
                                    for (name, style) in ArtStyle::ALL {
                                        let r = ui.selectable_label(cfg.art_style == *style, *name);
                                        if r.clicked() { cfg.art_style = style.clone(); changed = true; }
                                        else if r.hovered() { hover_art = Some(style.clone()); }
                                    }
                                });
                            if art_resp.inner.is_some() { any_hovered = true; }
                            if let Some(v) = hover_art {
                                any_hovered = true;
                                if cfg.art_style != v {
                                    if preview.is_none() { *preview = Some(cfg.clone()); }
                                    cfg.art_style = v; cfg.needs_rebuild = true;
                                }
                            }
                            let pd = cfg.art_depth;
                            ui.add(egui::Slider::new(&mut cfg.art_depth, 1..=9).text("depth"));
                            if cfg.art_depth != pd { changed = true; }
                            ui.add(egui::Slider::new(&mut cfg.art_anim, 0.0..=2.0).text("anim speed").step_by(0.05));
                            ui.horizontal(|ui| {
                                if ui.button("New seed").clicked() { cfg.art_seed = rand::random::<u64>(); changed = true; }
                                if ui.button("Regenerate").clicked() { changed = true; }
                            });
                        }
                    });

                ui.add_space(6.0);
                if ui.button("Reset to defaults").clicked() {
                    *cfg = FlexCfg::default(); *preview = None;
                }
                ui.add_space(4.0);
                ui.label("Copy code:");
                ui.horizontal(|ui| {
                    if ui.button("Bevy").clicked() {
                        ui.ctx().copy_text(emit_bevy_code(&cfg.root));
                    }
                    if ui.button("HTML/CSS").clicked() {
                        ui.ctx().copy_text(emit_html_css(&cfg.root));
                    }
                    if ui.button("Tailwind").clicked() {
                        ui.ctx().copy_text(emit_tailwind(&cfg.root));
                    }
                    if ui.button("SwiftUI").clicked() {
                        ui.ctx().copy_text(emit_swiftui(&cfg.root));
                    }
                });
            });
        });

    if changed {
        *preview = None;
        cfg.needs_rebuild = true;
    } else if !any_hovered
        && let Some(saved) = preview.take() {
            *cfg = saved;
            while !path_valid(&cfg.root, &cfg.selected) && !cfg.selected.is_empty() {
                cfg.selected.pop();
            }
            cfg.needs_rebuild = true;
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
    if !cfg.needs_rebuild { return; }
    cfg.needs_rebuild = false;
    for e in &roots { commands.entity(e).despawn(); }
    art.exprs.clear();
    art.handles.clear();
    if cfg.bg_mode == BgMode::RandomArt {
        let n = count_leaves(&cfg.root);
        let (base, depth, style) = (cfg.art_seed, cfg.art_depth, cfg.art_style.clone());
        for i in 0..n {
            let iseed = base.wrapping_add((i as u64).wrapping_mul(0x9e3779b97f4a7c15));
            let exprs = ArtExprs::generate(iseed, depth);
            let pixels = render_art(&style, &exprs, iseed, 0.0);
            let image = Image::new(
                Extent3d { width: ART_SZ, height: ART_SZ, depth_or_array_layers: 1 },
                TextureDimension::D2, pixels, TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            art.handles.push(images.add(image));
            art.exprs.push(exprs);
        }
    }
    spawn_viz(&mut commands, &cfg, &art);
}

fn spawn_viz(commands: &mut Commands, cfg: &FlexCfg, art: &ArtState) {
    let viz_root = commands.spawn((VizRoot, Node {
        width: Val::Percent(100.0), height: Val::Percent(100.0),
        flex_direction: FlexDirection::Row, align_items: AlignItems::Stretch,
        ..default()
    })).id();

    let spacer = commands.spawn(Node { width: Val::Px(PANEL_W), flex_shrink: 0.0, ..default() }).id();
    let area = commands.spawn(Node {
        flex_grow: 1.0, height: Val::Percent(100.0),
        display: Display::Block,
        padding: UiRect::all(Val::Px(16.0)), ..default()
    }).id();
    commands.entity(viz_root).add_children(&[spacer, area]);

    let mut leaf_idx = 0usize;
    spawn_node(commands, area, &cfg.root, cfg, art, &cfg.selected, &[], &mut leaf_idx);
}

fn spawn_node(
    commands: &mut Commands,
    parent_entity: Entity,
    node: &NodeCfg,
    cfg: &FlexCfg,
    art: &ArtState,
    selected_path: &[usize],
    current_path: &[usize],
    leaf_idx: &mut usize,
) {
    let is_selected = current_path == selected_path;
    let is_leaf = node.children.is_empty();

    let (bw, bc) = if is_selected {
        (3.0, Color::srgba(1.0, 0.85, 0.1, 1.0))
    } else {
        (1.5, Color::srgba(0.0, 0.0, 0.0, 0.35))
    };

    let bg_color = if is_leaf {
        if cfg.bg_mode == BgMode::Pastel { pastel(*leaf_idx) } else { Color::WHITE }
    } else {
        Color::srgba(0.11, 0.11, 0.17, 1.0)
    };

    let node_bevy = Node {
        display: Display::Flex,
        flex_direction: node.flex_direction,
        flex_wrap: node.flex_wrap,
        justify_content: node.justify_content,
        align_items: node.align_items,
        align_content: node.align_content,
        row_gap: node.row_gap.to_val(),
        column_gap: node.column_gap.to_val(),
        flex_grow: node.flex_grow,
        flex_shrink: node.flex_shrink,
        flex_basis: node.flex_basis.to_val(),
        align_self: node.align_self,
        width: node.width.to_val(),
        height: node.height.to_val(),
        min_width: node.min_width.to_val(),
        min_height: node.min_height.to_val(),
        max_width: node.max_width.to_val(),
        max_height: node.max_height.to_val(),
        padding: UiRect::all(node.padding.to_val()),
        margin: UiRect::all(node.margin.to_val()),
        border: UiRect::all(Val::Px(bw)),
        overflow: Overflow::clip(),
        ..default()
    };

    if is_leaf {
        let my_idx = *leaf_idx;
        *leaf_idx += 1;
        let info = node_info(node);
        let entity = commands.spawn((
            ArtItemNode(my_idx), node_bevy,
            BackgroundColor(bg_color), BorderColor::all(bc),
        )).id();
        if cfg.bg_mode == BgMode::RandomArt
            && let Some(h) = art.handles.get(my_idx) {
                commands.entity(entity).insert(ImageNode::new(h.clone()));
            }
        let scale = text_scale(node);
        let text_big = commands.spawn((
            Text::new(node.label.clone()),
            TextFont { font_size: (26.0_f32 * scale).clamp(1.0, 52.0), ..default() },
            TextColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
        )).id();
        let text_info = commands.spawn((
            Text::new(info),
            TextFont { font_size: (9.0_f32 * scale).clamp(1.0, 18.0), ..default() },
            TextColor(Color::srgba(0.05, 0.05, 0.1, 0.70)),
        )).id();
        commands.entity(entity).add_children(&[text_big, text_info]);
        commands.entity(parent_entity).add_child(entity);
    } else {
        let entity = commands.spawn((
            node_bevy, BackgroundColor(bg_color), BorderColor::all(bc),
        )).id();
        let cscale = text_scale(node);
        let lbl = commands.spawn((
            Text::new(node.label.clone()),
            TextFont { font_size: (10.0_f32 * cscale).clamp(1.0, 20.0), ..default() },
            TextColor(Color::srgba(0.7, 0.7, 0.9, 0.55)),
            Node { position_type: PositionType::Absolute, top: Val::Px(2.0), left: Val::Px(4.0), ..default() },
        )).id();
        commands.entity(entity).add_child(lbl);
        commands.entity(parent_entity).add_child(entity);
        for (i, child) in node.children.iter().enumerate() {
            let mut child_path = current_path.to_vec();
            child_path.push(i);
            spawn_node(commands, entity, child, cfg, art, selected_path, &child_path, leaf_idx);
        }
    }
}

// ─── Animation ────────────────────────────────────────────────────────────────

fn animate_art(
    mut images: ResMut<Assets<Image>>,
    art: Res<ArtState>,
    cfg: Res<FlexCfg>,
    time: Res<Time>,
    mut last_t: Local<f32>,
) {
    if cfg.art_anim < 1e-4 || cfg.bg_mode != BgMode::RandomArt { return; }
    let t = (time.elapsed_secs() * cfg.art_anim).sin();
    if (t - *last_t).abs() < 1e-4 { return; }
    *last_t = t;
    for (exprs, handle) in art.exprs.iter().zip(art.handles.iter()) {
        if let Some(image) = images.get_mut(handle) {
            image.data = Some(exprs.render(ART_SZ, ART_SZ, t));
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
    any_open: &mut bool,
) -> Option<T> {
    let sel = options.iter().find(|(_, v)| *v == *val).map(|(s, _)| *s).unwrap_or("?");
    let mut hover = None;
    let resp = egui::ComboBox::from_id_salt(label)
        .selected_text(sel).width(130.0)
        .show_ui(ui, |ui| {
            for (name, opt) in options {
                let r = ui.selectable_label(*val == *opt, *name);
                if r.clicked() { *val = *opt; *changed = true; }
                else if r.hovered() { hover = Some(*opt); }
            }
        });
    if resp.inner.is_some() { *any_open = true; }
    hover
}

fn val_row(
    ui: &mut egui::Ui,
    id: &str,
    val: &mut ValCfg,
    changed: &mut bool,
    any_open: &mut bool,
) -> Option<ValCfg> {
    const VARIANTS: &[&str] = &["Auto", "Px", "Percent", "Vw", "Vh"];
    let mut hover = None;
    let mut is_open = false;
    ui.horizontal(|ui| {
        let cur = val.variant();
        let resp = egui::ComboBox::from_id_salt(id).width(72.0).selected_text(cur)
            .show_ui(ui, |ui| {
                for &v in VARIANTS {
                    let r = ui.selectable_label(cur == v, v);
                    if r.clicked() { *val = val.cast(v); *changed = true; }
                    else if r.hovered() { hover = Some(val.cast(v)); }
                }
            });
        if resp.inner.is_some() { is_open = true; }
        if let Some(n) = val.num() {
            let mut n = n;
            let (lo, hi) = if matches!(val, ValCfg::Px(_)) { (0.0_f32, 600.0_f32) } else { (0.0_f32, 100.0_f32) };
            if ui.add(egui::Slider::new(&mut n, lo..=hi).max_decimals(0)).changed() {
                val.set_num(n); *changed = true;
            }
        }
    });
    if is_open { *any_open = true; }
    hover
}

// ─── Node info ────────────────────────────────────────────────────────────────

fn node_info(node: &NodeCfg) -> String {
    format!(
        "g:{} s:{}\nbasis:{} w:{} h:{}",
        ff(node.flex_grow), ff(node.flex_shrink),
        node.flex_basis.variant(), fv(&node.width), fv(&node.height)
    )
}

fn ff(v: f32) -> String {
    if (v - v.round()).abs() < 0.005 { format!("{}", v as i32) } else { format!("{v:.1}") }
}

fn fv(v: &ValCfg) -> String {
    match v {
        ValCfg::Auto => "auto".into(),
        ValCfg::Px(n) => format!("{n:.0}px"),
        ValCfg::Percent(n) => format!("{n:.0}%"),
        ValCfg::Vw(n) => format!("{n:.0}vw"),
        ValCfg::Vh(n) => format!("{n:.0}vh"),
    }
}
