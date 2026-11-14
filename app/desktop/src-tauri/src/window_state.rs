use gitfleet_core::application::WindowPlacement;
use tauri::{LogicalSize, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow};

pub const MIN_WIDTH: f64 = 960.;
pub const MIN_HEIGHT: f64 = 640.;

#[cfg(test)]
fn fit(saved: WindowPlacement, area: WindowPlacement, scale: f64) -> WindowPlacement {
    fit_minimum(saved, area, scale, MIN_WIDTH, MIN_HEIGHT)
}
fn fit_minimum(
    saved: WindowPlacement,
    area: WindowPlacement,
    scale: f64,
    min_width: f64,
    min_height: f64,
) -> WindowPlacement {
    let minimum: PhysicalSize<u32> = LogicalSize::new(min_width, min_height).to_physical(scale);
    let width = saved.width.max(minimum.width).min(area.width);
    let height = saved.height.max(minimum.height).min(area.height);
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
    restore_minimum(window, saved, MIN_WIDTH, MIN_HEIGHT)
}
pub fn restore_settings<R: Runtime>(
    window: &WebviewWindow<R>,
    saved: WindowPlacement,
) -> tauri::Result<()> {
    restore_minimum(window, saved, 620., 420.)
}
fn restore_minimum<R: Runtime>(
    window: &WebviewWindow<R>,
    saved: WindowPlacement,
    min_width: f64,
    min_height: f64,
) -> tauri::Result<()> {
    let monitors = window.available_monitors()?;
    let areas: Vec<_> = monitors
        .iter()
        .map(|m| {
            let r = m.work_area();
            (
                WindowPlacement {
                    x: r.position.x,
                    y: r.position.y,
                    width: r.size.width,
                    height: r.size.height,
                },
                m.scale_factor(),
            )
        })
        .collect();
    let area = areas
        .iter()
        .find(|(a, _)| {
            saved.x >= a.x
                && saved.y >= a.y
                && i64::from(saved.x) < i64::from(a.x) + i64::from(a.width)
                && i64::from(saved.y) < i64::from(a.y) + i64::from(a.height)
        })
        .or_else(|| areas.first());
    if let Some((area, scale)) = area {
        let p = fit_minimum(saved, *area, *scale, min_width, min_height);
        // The hidden window can start on a different-DPI display. Physical
        // setters convert using that window's current scale before it moves.
        window.set_position(PhysicalPosition::new(p.x, p.y).to_logical::<f64>(*scale))?;
        window.set_size(PhysicalSize::new(p.width, p.height).to_logical::<f64>(*scale))?;
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
        assert_eq!(fit(saved, area, 1.), area);
        let saved = WindowPlacement {
            x: -1500,
            y: 100,
            width: 1200,
            height: 700,
        };
        assert_eq!(fit(saved, area, 1.), saved);
    }

    #[test]
    fn minimum_dimensions_follow_destination_scale_and_fit_small_work_areas() {
        let area = WindowPlacement {
            x: -5120,
            y: 794,
            width: 5120,
            height: 2818,
        };
        let saved = WindowPlacement {
            x: -5000,
            y: 900,
            width: 720,
            height: 470,
        };
        let fitted = fit(saved, area, 2.);
        assert_eq!((fitted.width, fitted.height), (1920, 1280));
        assert_eq!((fitted.x, fitted.y), (saved.x, saved.y));
        let small = WindowPlacement {
            x: 0,
            y: 31,
            width: 800,
            height: 500,
        };
        assert_eq!(fit(saved, small, 1.), small);
        let standard = WindowPlacement {
            x: 0,
            y: 31,
            width: 3840,
            height: 2052,
        };
        let repaired = fit(
            WindowPlacement {
                x: 600,
                y: 155,
                ..saved
            },
            standard,
            1.,
        );
        assert_eq!((repaired.width, repaired.height), (960, 640));
    }
}
