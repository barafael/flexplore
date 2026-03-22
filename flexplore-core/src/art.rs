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
