/// FerrumCalc – Application Icon
///
/// Generates the window/taskbar icon procedurally as a raw RGBA buffer, so the
/// app carries no binary image asset and needs no PNG decoder at runtime.
///
/// The motif is a flat calculator: a rounded indigo square (matching the app's
/// accent color) with a light display bar near the top and a 3×3 keypad whose
/// right-hand "operator" column is tinted warm orange for a focal accent.

use eframe::egui;

const SIZE: usize = 256;

/// Build the application icon.
pub fn app_icon() -> egui::IconData {
    let mut rgba = vec![0u8; SIZE * SIZE * 4];

    // Background gradient (top → bottom), keypad colors. RGB on a 0..255 scale.
    let bg_top = [108.0, 121.0, 255.0];
    let bg_bottom = [72.0, 84.0, 200.0];
    let screen = [233.0, 236.0, 255.0];
    let key_light = [245.0, 247.0, 255.0];
    let key_op = [255.0, 159.0, 67.0];

    // Keypad layout: 3 columns × 3 rows of rounded square keys.
    let col_x = [72.0, 128.0, 184.0];
    let row_y = [124.0, 168.0, 212.0];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            // Straight-alpha accumulator: [r, g, b (0..255), a (0..1)].
            let mut col = [0.0f32, 0.0, 0.0, 0.0];

            // Rounded-square background, filled with a vertical indigo gradient.
            let bg = rounded_box_coverage(px, py, 128.0, 128.0, 112.0, 112.0, 48.0);
            if bg > 0.0 {
                let t = ((py - 16.0) / 224.0).clamp(0.0, 1.0);
                let r = lerp(bg_top[0], bg_bottom[0], t);
                let g = lerp(bg_top[1], bg_bottom[1], t);
                let b = lerp(bg_top[2], bg_bottom[2], t);
                col = over(col, [r, g, b, bg]);
            }

            // Display bar.
            let sc = rounded_box_coverage(px, py, 128.0, 70.0, 84.0, 26.0, 14.0);
            if sc > 0.0 {
                col = over(col, [screen[0], screen[1], screen[2], 0.97 * sc]);
            }

            // Keypad.
            for (ci, &cx) in col_x.iter().enumerate() {
                let base = if ci == 2 { key_op } else { key_light };
                for &cy in row_y.iter() {
                    let k = rounded_box_coverage(px, py, cx, cy, 22.0, 22.0, 11.0);
                    if k > 0.0 {
                        col = over(col, [base[0], base[1], base[2], 0.95 * k]);
                    }
                }
            }

            let idx = (y * SIZE + x) * 4;
            rgba[idx] = col[0].round().clamp(0.0, 255.0) as u8;
            rgba[idx + 1] = col[1].round().clamp(0.0, 255.0) as u8;
            rgba[idx + 2] = col[2].round().clamp(0.0, 255.0) as u8;
            rgba[idx + 3] = (col[3] * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }

    egui::IconData {
        rgba,
        width: SIZE as u32,
        height: SIZE as u32,
    }
}

/// Linear interpolation between `a` and `b` by `t` in [0, 1].
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Source-over alpha compositing for straight-alpha colors
/// (`[r, g, b (0..255), a (0..1)]`).
fn over(dst: [f32; 4], src: [f32; 4]) -> [f32; 4] {
    let (sa, da) = (src[3], dst[3]);
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return [0.0, 0.0, 0.0, 0.0];
    }
    let blend = |s: f32, d: f32| (s * sa + d * da * (1.0 - sa)) / out_a;
    [
        blend(src[0], dst[0]),
        blend(src[1], dst[1]),
        blend(src[2], dst[2]),
        out_a,
    ]
}

/// Coverage (0..1) of pixel `(px, py)` inside a rounded box centered at
/// `(cx, cy)` with half-extents `(hx, hy)` and corner radius `r`. The 1px
/// transition band gives the edges light anti-aliasing.
fn rounded_box_coverage(px: f32, py: f32, cx: f32, cy: f32, hx: f32, hy: f32, r: f32) -> f32 {
    let qx = (px - cx).abs() - (hx - r);
    let qy = (py - cy).abs() - (hy - r);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    let inside = qx.max(qy).min(0.0);
    let dist = outside + inside - r;
    (0.5 - dist).clamp(0.0, 1.0)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Read the RGBA pixel at (x, y) from the generated icon.
    fn pixel(icon: &egui::IconData, x: usize, y: usize) -> [u8; 4] {
        let i = (y * SIZE + x) * 4;
        [icon.rgba[i], icon.rgba[i + 1], icon.rgba[i + 2], icon.rgba[i + 3]]
    }

    #[test]
    fn test_icon_dimensions() {
        let icon = app_icon();
        assert_eq!(icon.width, SIZE as u32);
        assert_eq!(icon.height, SIZE as u32);
        assert_eq!(icon.rgba.len(), SIZE * SIZE * 4);
    }

    #[test]
    fn test_corner_is_transparent() {
        // The rounded square clears the very corners of the canvas.
        let icon = app_icon();
        assert_eq!(pixel(&icon, 2, 2)[3], 0);
    }

    #[test]
    fn test_background_is_opaque_indigo() {
        // A point above the display bar is plain background: opaque, blue-dominant.
        let icon = app_icon();
        let [r, g, b, a] = pixel(&icon, 128, 40);
        assert_eq!(a, 255);
        assert!(b > r && b > g, "expected indigo, got {r},{g},{b}");
    }

    #[test]
    fn test_operator_key_is_orange() {
        // Center of the top key in the right-hand operator column.
        let icon = app_icon();
        let [r, g, b, a] = pixel(&icon, 184, 124);
        assert!(a > 200);
        assert!(r > g && g > b, "expected warm orange, got {r},{g},{b}");
    }
}
