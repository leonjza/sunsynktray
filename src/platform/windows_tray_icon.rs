use gpui_tray::Icon;
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, RECT},
        Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject, DrawTextW,
            GetDC, GetDeviceCaps, ReleaseDC, SelectObject, SetBkMode, SetTextColor, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
            DEFAULT_PITCH, DIB_RGB_COLORS, DT_CENTER, DT_SINGLELINE, DT_VCENTER, FF_DONTCARE,
            FW_SEMIBOLD, LOGPIXELSX, OUT_DEFAULT_PRECIS, TRANSPARENT,
        },
        UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON},
    },
};

pub(crate) fn render(value: Option<&str>, symbol: &str) -> Result<Icon, String> {
    let Some(value) = value else {
        return Ok(fallback());
    };

    let reference = unsafe { GetDC(None) };
    if reference.is_invalid() {
        return Err("could not acquire Windows display context".into());
    }
    let dpi = unsafe { GetDeviceCaps(Some(reference), LOGPIXELSX).max(96) } as f32 / 96.0;
    let base_size = unsafe { GetSystemMetrics(SM_CXSMICON).max(16) } as f32;
    let size = (base_size * dpi).round() as i32;
    unsafe { ReleaseDC(None, reference) };

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
        return Err("could not create Windows icon context".into());
    }
    let mut bits = std::ptr::null_mut();
    let bitmap = unsafe { CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0) }
        .map_err(|error| error.to_string())?;
    if bits.is_null() {
        unsafe {
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
        }
        return Err("Windows returned a null icon buffer".into());
    }
    unsafe { std::ptr::write_bytes(bits, 0, (size * size * 4) as usize) };
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
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32 | FF_DONTCARE.0 as u32,
            w!("Segoe UI"),
        )
    };
    let old_font = unsafe { SelectObject(dc, font.into()) };
    unsafe { SetBkMode(dc, TRANSPARENT) };
    let text = compact_value(value);
    let mut wide = text.encode_utf16().collect::<Vec<_>>();
    let rect = RECT {
        left: 0,
        top: 0,
        right: size,
        bottom: size,
    };
    let flags = DT_CENTER | DT_VCENTER | DT_SINGLELINE;
    let color = metric_color(symbol);
    for (dx, dy) in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
        let mut outline = RECT {
            left: rect.left + dx,
            top: rect.top + dy,
            right: rect.right + dx,
            bottom: rect.bottom + dy,
        };
        unsafe {
            SetTextColor(dc, COLORREF(0x00101010));
            DrawTextW(dc, &mut wide, &mut outline, flags);
        }
    }
    let mut text_rect = rect;
    unsafe {
        SetTextColor(dc, COLORREF(color));
        DrawTextW(dc, &mut wide, &mut text_rect, flags);
    }
    let bgra = unsafe {
        std::slice::from_raw_parts(bits.cast::<u8>(), (size * size * 4) as usize).to_vec()
    };
    unsafe {
        SelectObject(dc, old_font);
        let _ = DeleteObject(font.into());
        SelectObject(dc, old_bitmap);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(dc);
    }

    let mut rgba = Vec::with_capacity(bgra.len());
    for pixel in bgra.chunks_exact(4) {
        let alpha = if pixel[..3].iter().any(|channel| *channel != 0) {
            255
        } else {
            0
        };
        rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], alpha]);
    }
    Icon::from_rgba(rgba, size as u32, size as u32).map_err(|error| error.to_string())
}

pub(crate) fn fallback() -> Icon {
    let size = 20_u32;
    let mut rgba = vec![0_u8; (size * size * 4) as usize];
    for y in 2..18 {
        for x in 4..16 {
            if (x < 10 && y > x / 2 + 2) || (x >= 10 && y < 12 && y > 4) {
                let offset = ((y * size + x) * 4) as usize;
                rgba[offset..offset + 4].copy_from_slice(&[255, 196, 0, 255]);
            }
        }
    }
    Icon::from_rgba(rgba, size, size).expect("fallback tray icon dimensions are valid")
}

fn compact_value(value: &str) -> String {
    value
        .trim()
        .trim_end_matches('%')
        .replace(" kW", "k")
        .replace(" W", "")
}

fn metric_color(symbol: &str) -> u32 {
    match symbol {
        "battery.100" => 0x0080DE4A,
        "house.fill" => 0x00BFD42D,
        "sun.max.fill" => 0x0042B9F5,
        _ => 0x00FFFFFF,
    }
}

#[cfg(test)]
mod tests {
    use super::compact_value;

    #[test]
    fn compact_values_fit_the_tray_icon() {
        assert_eq!(compact_value("78%"), "78");
        assert_eq!(compact_value("1.2 kW"), "1.2k");
        assert_eq!(compact_value("850 W"), "850");
    }
}
