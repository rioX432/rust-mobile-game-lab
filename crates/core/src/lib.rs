use serde::{Deserialize, Serialize};

pub mod game;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeKind {
    Rectangle,
    Circle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    pub kind: ShapeKind,
    /// Rectangle: x/y/width/height. Circle: center x/y/radius/unused, in logical pixels.
    pub bounds: [f32; 4],
    pub color: [f32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub size: [f32; 2],
    pub shapes: Vec<Shape>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Initializing,
    Running,
    Paused,
    Detached,
    Destroyed,
}

#[derive(Debug, Default)]
pub struct Probe {
    pub selected_lane: Option<u32>,
    pub touches: u64,
}

impl Probe {
    pub fn touch(&mut self, x: f32, y: f32, width: f32, height: f32) -> bool {
        if ![x, y, width, height].iter().all(|v| v.is_finite())
            || width <= 0.0
            || height <= 0.0
            || x < 0.0
            || y < 0.0
            || x >= width
            || y >= height
        {
            return false;
        }
        self.selected_lane = Some(((x / width * 3.0) as u32).min(2));
        self.touches += 1;
        true
    }

    pub fn scene(&self, width: u32, height: u32, scale: f32) -> Scene {
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        let w = width as f32 / scale;
        let h = height as f32 / scale;
        let mut shapes = Vec::new();
        for lane in 0..3 {
            shapes.push(Shape {
                kind: ShapeKind::Rectangle,
                bounds: [
                    lane as f32 * w / 3.0 + 3.0,
                    4.0,
                    (w / 3.0 - 6.0).max(0.0),
                    (h - 8.0).max(0.0),
                ],
                color: if self.selected_lane == Some(lane) {
                    [0.15, 0.45, 0.55, 1.0]
                } else {
                    [0.12, 0.16, 0.25, 1.0]
                },
            });
            shapes.push(Shape {
                kind: ShapeKind::Circle,
                bounds: [(lane as f32 + 0.5) * w / 3.0, h * 0.5, w.min(h) / 14.0, 0.0],
                color: [
                    [0.98, 0.55, 0.38, 1.0],
                    [0.48, 0.85, 0.70, 1.0],
                    [0.70, 0.60, 0.98, 1.0],
                ][lane as usize],
            });
        }
        Scene {
            size: [w, h],
            shapes,
        }
    }
}

pub fn frame_delta(previous_ms: Option<u64>, now_ms: u64) -> f32 {
    previous_ms.map_or(0.0, |last| {
        (now_ms.saturating_sub(last).min(50) as f32) / 1000.0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_uses_local_surface_coordinates_and_rejects_invalid_input() {
        let mut probe = Probe::default();
        assert!(probe.touch(200.0, 100.0, 300.0, 400.0));
        assert_eq!(probe.selected_lane, Some(2));
        assert!(!probe.touch(300.0, 100.0, 300.0, 400.0));
        assert!(!probe.touch(f32::NAN, 0.0, 300.0, 400.0));
        assert!(!probe.touch(0.0, 0.0, 0.0, 0.0));
        assert_eq!(probe.touches, 1);
    }

    #[test]
    fn scene_density_preserves_logical_geometry() {
        let probe = Probe::default();
        assert_eq!(probe.scene(300, 400, 1.0), probe.scene(600, 800, 2.0));
    }

    #[test]
    fn resume_and_clock_changes_do_not_inject_large_delta() {
        assert_eq!(frame_delta(None, 90_000), 0.0);
        assert_eq!(frame_delta(Some(0), 90_000), 0.05);
        assert_eq!(frame_delta(Some(10), 5), 0.0);
    }
}
