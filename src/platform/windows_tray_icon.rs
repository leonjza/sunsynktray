use gpui_tray::Icon;
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, RECT},
        Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject, DrawTextW,
            GetDC, GetDeviceCaps, ReleaseDC, SelectObject, SetBkMode, SetTextColor,
            ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS,
            DEFAULT_CHARSET, DEFAULT_PITCH, DIB_RGB_COLORS, DT_CENTER, DT_NOPREFIX, DT_SINGLELINE,
            DT_VCENTER, FF_DONTCARE, FW_SEMIBOLD, LOGPIXELSX, OUT_DEFAULT_PRECIS,
        },
        UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON},
    },
};

pub(crate) fn render(value: Option<&str>, symbol: &str) -> Result<Icon, String> {
    let Some(value) = value else {
        return Ok(fallback());
    };
    let size = icon_size();
    let text = compact_value(value);
    let outline = render_mask(size, &text, 1, 1)?;
    let foreground = render_mask(size, &text, 0, 0)?;
    let color = metric_color(value, symbol);
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for (outline, foreground) in outline.iter().zip(foreground.iter()) {
        let text_alpha = *foreground as u16;
        let outline_alpha = *outline as u16;
        let alpha = text_alpha + outline_alpha * (255 - text_alpha) / 255;
        if alpha == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let text_weight = text_alpha;
            let outline_weight = outline_alpha * (255 - text_alpha) / 255;
            let red = (color.0 as u16 * text_weight + 16 * outline_weight) / alpha;
            let green = (color.1 as u16 * text_weight + 16 * outline_weight) / alpha;
            let blue = (color.2 as u16 * text_weight + 16 * outline_weight) / alpha;
            rgba.extend_from_slice(&[red as u8, green as u8, blue as u8, alpha as u8]);
        }
    }
    Icon::from_rgba(rgba, size as u32, size as u32).map_err(|e| e.to_string())
}

fn icon_size() -> i32 {
    let reference = unsafe { GetDC(None) };
    if reference.is_invalid() {
        return 16;
    }
    let dpi = unsafe { GetDeviceCaps(Some(reference), LOGPIXELSX).max(96) } as f32 / 96.0;
    unsafe {
        ReleaseDC(None, reference);
    }
    let base = unsafe { GetSystemMetrics(SM_CXSMICON).max(16) } as f32;
    ((base * dpi).round() as i32).max(16)
}

fn render_mask(size: i32, text: &str, dx: i32, dy: i32) -> Result<Vec<u8>, String> {
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -size,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc.is_invalid() {
        return Err("could not create Windows text context".into());
    }
    let mut bits = std::ptr::null_mut();
    let bitmap = unsafe { CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0) }
        .map_err(|e| e.to_string())?;
    if bits.is_null() {
        unsafe {
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
        }
        return Err("null Windows text buffer".into());
    }
    unsafe {
        std::ptr::write_bytes(bits, 0, (size * size * 4) as usize);
    }
    let old_bitmap = unsafe { SelectObject(dc, bitmap.into()) };
    let font_size = ((size as f32) * 0.62).round().max(8.0) as i32;
    let font = unsafe {
        CreateFontW(
            -font_size,
            0,
            0,
            0,
            FW_SEMIBOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            ANTIALIASED_QUALITY,
            DEFAULT_PITCH.0 as u32 | FF_DONTCARE.0 as u32,
            w!("Segoe UI Semibold"),
        )
    };
    let old_font = unsafe { SelectObject(dc, font.into()) };
    unsafe {
        SetBkMode(dc, windows::Win32::Graphics::Gdi::TRANSPARENT);
        SetTextColor(dc, COLORREF(0x00FFFFFF));
    }
    let mut wide = text.encode_utf16().collect::<Vec<_>>();
    let mut rect = RECT {
        left: dx,
        top: dy,
        right: size + dx,
        bottom: size + dy,
    };
    unsafe {
        DrawTextW(
            dc,
            &mut wide,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );
    }
    let source =
        unsafe { std::slice::from_raw_parts(bits.cast::<u8>(), (size * size * 4) as usize) };
    let mask = source
        .chunks_exact(4)
        .map(|pixel| pixel[0])
        .collect::<Vec<_>>();
    unsafe {
        SelectObject(dc, old_font);
        let _ = DeleteObject(font.into());
        SelectObject(dc, old_bitmap);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(dc);
    }
    Ok(mask)
}

fn metric_color(value: &str, symbol: &str) -> (u8, u8, u8) {
    if symbol == "battery.100" {
        if let Ok(soc) = value.trim().trim_end_matches('%').parse::<f64>() {
            let soc = soc.clamp(0.0, 100.0);
            return if soc >= 80.0 {
                (34, 197, 94)
            } else if soc >= 60.0 {
                (132, 204, 22)
            } else if soc >= 40.0 {
                (234, 179, 8)
            } else {
                (239, 68, 68)
            };
        }
    }
    match symbol {
        "house.fill" => (191, 212, 45),
        "sun.max.fill" => (66, 185, 245),
        _ => (255, 255, 255),
    }
}

fn compact_value(value: &str) -> String {
    value
        .trim()
        .trim_end_matches('%')
        .replace(" kW", "k")
        .replace(" W", "")
}

pub(crate) fn fallback() -> Icon {
    let size = 16;
    let mut rgba = vec![0; size * size * 4];
    for y in 2..14 {
        for x in 4..12 {
            if (x < 8 && y > x / 2 + 2) || (x >= 8 && y < 10 && y > 3) {
                let o = (y * size + x) * 4;
                rgba[o..o + 4].copy_from_slice(&[255, 196, 0, 255]);
            }
        }
    }
    Icon::from_rgba(rgba, size as u32, size as u32).expect("valid fallback icon")
}
