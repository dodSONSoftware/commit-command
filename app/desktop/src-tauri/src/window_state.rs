use gitfleet_core::application::WindowPlacement;
use tauri::{PhysicalPosition, PhysicalSize, Runtime, WebviewWindow};

fn fit(saved: WindowPlacement, area: WindowPlacement) -> WindowPlacement {
    let width = saved.width.min(area.width);
    let height = saved.height.min(area.height);
    WindowPlacement {
        x: saved.x.clamp(area.x, area.x + (area.width - width) as i32),
        y: saved
            .y
            .clamp(area.y, area.y + (area.height - height) as i32),
        width,
        height,
    }
}
pub fn restore<R: Runtime>(window: &WebviewWindow<R>, saved: WindowPlacement) -> tauri::Result<()> {
    let monitors = window.available_monitors()?;
    let areas: Vec<_> = monitors
        .iter()
        .map(|m| {
            let r = m.work_area();
            WindowPlacement {
                x: r.position.x,
                y: r.position.y,
                width: r.size.width,
                height: r.size.height,
            }
        })
        .collect();
    let area = areas
        .iter()
        .find(|a| {
            saved.x >= a.x
                && saved.y >= a.y
                && i64::from(saved.x) < i64::from(a.x) + i64::from(a.width)
                && i64::from(saved.y) < i64::from(a.y) + i64::from(a.height)
        })
        .or_else(|| areas.first());
    if let Some(area) = area {
        let p = fit(saved, *area);
        window.set_size(PhysicalSize::new(p.width, p.height))?;
        window.set_position(PhysicalPosition::new(p.x, p.y))?;
    }
    Ok(())
}
pub fn capture<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<Option<WindowPlacement>> {
    if window.is_maximized()? || window.is_minimized()? || window.is_fullscreen()? {
        return Ok(None);
    }
    let position = window.outer_position()?;
    let size = window.inner_size()?;
    Ok(Some(WindowPlacement {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    }))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_geometry_fits_changed_display_and_preserves_valid_position() {
        let area = WindowPlacement {
            x: -1600,
            y: 25,
            width: 1600,
            height: 875,
        };
        let saved = WindowPlacement {
            x: 5000,
            y: -3000,
            width: 2000,
            height: 1200,
        };
        assert_eq!(fit(saved, area), area);
        let saved = WindowPlacement {
            x: -1500,
            y: 100,
            width: 1200,
            height: 700,
        };
        assert_eq!(fit(saved, area), saved);
    }
}
