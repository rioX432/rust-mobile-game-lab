use game_core::Probe;
use game_renderer::{Renderer, SURFACE_LOST};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

const FRAME_INTERVAL: Duration = Duration::from_millis(16);

enum UserEvent {
    Initialized(u64, Result<Renderer, String>),
}

struct App {
    renderer: Option<Renderer>,
    window: Option<Arc<Window>>,
    instance: wgpu::Instance,
    proxy: EventLoopProxy<UserEvent>,
    generation: u64,
    initializing: bool,
    active: bool,
    occluded: bool,
    paused: bool,
    hidden: bool,
    probe: Probe,
    cursor: Option<PhysicalPosition<f64>>,
    frames: u64,
    next_frame: Instant,
    error: Option<String>,
    smoke: Option<Smoke>,
}

struct Smoke {
    deadline: Instant,
    stage: u8,
    wait_until: Instant,
    stopped_at: u64,
    expected_size: PhysicalSize<u32>,
    resized: bool,
}

impl App {
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: String) {
        eprintln!("desktop error: {error}");
        self.error = Some(error);
        event_loop.exit();
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) {
        if self.initializing || self.renderer.is_some() || !self.active {
            return;
        }
        let Some(window) = &self.window else { return };
        let surface = match self.instance.create_surface(Arc::clone(window)) {
            Ok(surface) => surface,
            Err(error) => {
                self.fail(event_loop, format!("create surface: {error}"));
                return;
            }
        };
        let size = window.inner_size();
        let instance = self.instance.clone();
        let proxy = self.proxy.clone();
        self.generation += 1;
        let generation = self.generation;
        self.initializing = true;
        // Window/surface creation stays on the event thread; GPU initialization does not block it.
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                pollster::block_on(Renderer::new(&instance, surface, size.width, size.height))
            }))
            .unwrap_or_else(|_| Err("GPU initialization panicked".into()));
            let _ = proxy.send_event(UserEvent::Initialized(generation, result));
        });
    }

    fn can_draw(&self) -> bool {
        self.active
            && !self.paused
            && !self.occluded
            && !self.hidden
            && self.renderer.is_some()
            && self.window.as_ref().is_some_and(|w| {
                let size = w.inner_size();
                size.width > 0 && size.height > 0
            })
    }

    fn touch(&mut self, position: PhysicalPosition<f64>, source: &str) {
        if !self.can_draw() {
            return;
        }
        let window = self.window.as_ref().unwrap();
        let size = window.inner_size();
        let scale = window.scale_factor() as f32;
        let accepted = self.probe.touch(
            position.x as f32 / scale,
            position.y as f32 / scale,
            size.width as f32 / scale,
            size.height as f32 / scale,
        );
        println!(
            "input source={source} accepted={accepted} touches={} selected_lane={:?}",
            self.probe.touches, self.probe.selected_lane
        );
        window.request_redraw();
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(size.width, size.height);
        }
        println!("resize physical={}x{}", size.width, size.height);
        if let Some(smoke) = &mut self.smoke
            && size == smoke.expected_size
        {
            smoke.resized = true;
        }
        self.next_frame = Instant::now();
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        if !self.can_draw() {
            return;
        }
        let window = self.window.as_ref().unwrap();
        let size = window.inner_size();
        let scene = self
            .probe
            .scene(size.width, size.height, window.scale_factor() as f32);
        let renderer = self.renderer.as_mut().unwrap();
        let before = renderer.presented_frames();
        window.pre_present_notify();
        match renderer.render(&scene) {
            Ok(()) => self.frames += renderer.presented_frames() - before,
            Err(error) if error == SURFACE_LOST => {
                println!("surface lost: recreating renderer");
                self.renderer = None;
                self.initialize(event_loop);
            }
            Err(error) => self.fail(event_loop, error),
        }
        self.next_frame = Instant::now() + FRAME_INTERVAL;
    }

    fn advance_smoke(&mut self, event_loop: &ActiveEventLoop) {
        let Some(smoke) = &self.smoke else { return };
        let now = Instant::now();
        if now >= smoke.deadline {
            self.fail(
                event_loop,
                "smoke timed out before completing real presentations".into(),
            );
            return;
        }
        let stage = smoke.stage;
        let window = Arc::clone(self.window.as_ref().unwrap());
        match stage {
            0 if self.frames >= 3 => {
                let size = window.inner_size();
                // Inject through the same local-coordinate path as mouse/touch events.
                self.touch(
                    PhysicalPosition::new(size.width as f64 * 0.8, size.height as f64 * 0.5),
                    "smoke",
                );
                if self.probe.touches != 1 || self.probe.selected_lane != Some(2) {
                    self.fail(event_loop, "smoke input did not reach Probe".into());
                    return;
                }
                let expected = PhysicalSize::new(size.width + 96, size.height + 48);
                let smoke = self.smoke.as_mut().unwrap();
                smoke.expected_size = expected;
                smoke.stage = 1;
                let immediate = window.request_inner_size(expected);
                if let Some(size) = immediate {
                    self.resize(size);
                }
            }
            1 if smoke.resized && self.frames >= 6 => {
                self.paused = true;
                let smoke = self.smoke.as_mut().unwrap();
                smoke.stage = 2;
                smoke.stopped_at = self.frames;
                smoke.wait_until = now + Duration::from_millis(200);
                println!("smoke pause frames={}", self.frames);
            }
            2 if now >= smoke.wait_until => {
                if self.frames != smoke.stopped_at {
                    self.fail(event_loop, "frames advanced while paused".into());
                    return;
                }
                self.paused = false;
                self.hidden = true;
                window.set_visible(false);
                let smoke = self.smoke.as_mut().unwrap();
                smoke.stage = 3;
                smoke.wait_until = now + Duration::from_millis(200);
                println!("smoke hide frames={}", self.frames);
            }
            3 if now >= smoke.wait_until => {
                if self.frames != smoke.stopped_at {
                    self.fail(event_loop, "frames advanced while hidden".into());
                    return;
                }
                self.hidden = false;
                window.set_visible(true);
                self.smoke.as_mut().unwrap().stage = 4;
                self.next_frame = now;
            }
            4 if self.frames >= smoke.stopped_at + 3 => {
                // Exercise zero-size suppression on the real renderer without relying on the OS
                // accepting a zero-sized native window.
                let renderer = self.renderer.as_mut().unwrap();
                let before = renderer.presented_frames();
                renderer.resize(0, 0);
                if let Err(error) = renderer.render(&self.probe.scene(0, 0, 1.0)) {
                    self.fail(event_loop, error);
                    return;
                }
                if renderer.presented_frames() != before {
                    self.fail(event_loop, "zero-size frame was presented".into());
                    return;
                }
                let size = window.inner_size();
                renderer.resize(size.width, size.height);
                let smoke = self.smoke.as_mut().unwrap();
                smoke.stage = 5;
                smoke.wait_until = now + Duration::from_secs(2);
                println!(
                    "smoke zero-size skipped; restoring {}x{}",
                    size.width, size.height
                );
            }
            5 if self.frames >= smoke.stopped_at + 6 && now >= smoke.wait_until => {
                println!(
                    "SMOKE_OK presented_frames={} touches={} selected_lane={:?} resize={} pause_no_frames=true hide_no_frames=true zero_size_skipped=true",
                    self.frames, self.probe.touches, self.probe.selected_lane, smoke.resized
                );
                self.smoke.as_mut().unwrap().stage = 6;
                event_loop.exit();
            }
            _ => {}
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.active = true;
        if self.window.is_none() {
            match event_loop.create_window(
                Window::default_attributes()
                    .with_title("Rust Mobile Game Lab — Probe (Space: pause)")
                    .with_inner_size(LogicalSize::new(720.0, 480.0)),
            ) {
                Ok(window) => self.window = Some(Arc::new(window)),
                Err(error) => {
                    self.fail(event_loop, error.to_string());
                    return;
                }
            }
        }
        self.next_frame = Instant::now();
        self.initialize(event_loop);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.active = false;
        self.generation += 1;
        self.renderer = None;
        println!("suspended frames={}", self.frames);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        let UserEvent::Initialized(generation, result) = event;
        self.initializing = false;
        if generation != self.generation || !self.active {
            // Finish releasing the previous surface before starting another initializer.
            drop(result);
            self.initialize(event_loop);
            return;
        }
        match result {
            Ok(mut renderer) => {
                let window = self.window.as_ref().unwrap();
                let size = window.inner_size();
                renderer.resize(size.width, size.height);
                self.renderer = Some(renderer);
                self.next_frame = Instant::now();
                println!(
                    "GPU ready physical={}x{} scale={}",
                    size.width,
                    size.height,
                    window.scale_factor()
                );
            }
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if !self.window.as_ref().is_some_and(|w| w.id() == id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::ScaleFactorChanged { .. } => {
                self.resize(self.window.as_ref().unwrap().inner_size());
            }
            WindowEvent::Occluded(occluded) => {
                self.occluded = occluded;
                self.next_frame = Instant::now();
                println!("occluded={occluded} frames={}", self.frames);
            }
            WindowEvent::CursorMoved { position, .. } => self.cursor = Some(position),
            WindowEvent::CursorLeft { .. } => self.cursor = None,
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(position) = self.cursor {
                    self.touch(position, "mouse");
                }
            }
            WindowEvent::Touch(touch) if touch.phase == TouchPhase::Started => {
                self.touch(touch.location, "touch");
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state.is_pressed() && !event.repeat =>
            {
                match event.logical_key {
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    Key::Named(NamedKey::Space) => {
                        self.paused = !self.paused;
                        self.next_frame = Instant::now();
                        println!("paused={} frames={}", self.paused, self.frames);
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => self.redraw(event_loop),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            self.advance_smoke(event_loop);
        }
        let now = Instant::now();
        let mut wake_at = self.smoke.as_ref().map(|s| s.deadline);
        if let Some(smoke) = &self.smoke
            && (smoke.stage == 2 || smoke.stage == 3)
        {
            wake_at = Some(smoke.wait_until.min(smoke.deadline));
        }
        if self.can_draw() {
            if now >= self.next_frame {
                self.window.as_ref().unwrap().request_redraw();
                self.next_frame = now + FRAME_INTERVAL;
            }
            wake_at = Some(wake_at.map_or(self.next_frame, |t| t.min(self.next_frame)));
        }
        event_loop.set_control_flow(wake_at.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.generation += 1;
        self.renderer = None;
        self.window = None;
        println!("desktop exit presented_frames={}", self.frames);
    }
}

fn main() -> Result<(), String> {
    let smoke = match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => false,
        [arg] if arg == "--smoke" => true,
        _ => return Err("usage: game-desktop [--smoke]".into()),
    };
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .map_err(|e| e.to_string())?;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
        Box::new(event_loop.owned_display_handle()),
    ));
    let now = Instant::now();
    let mut app = App {
        renderer: None,
        window: None,
        instance,
        proxy: event_loop.create_proxy(),
        generation: 0,
        initializing: false,
        active: false,
        occluded: false,
        paused: false,
        hidden: false,
        probe: Probe::default(),
        cursor: None,
        frames: 0,
        next_frame: now,
        error: None,
        smoke: smoke.then_some(Smoke {
            deadline: now + Duration::from_secs(20),
            stage: 0,
            wait_until: now,
            stopped_at: 0,
            expected_size: PhysicalSize::new(0, 0),
            resized: false,
        }),
    };
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;
    if let Some(error) = app.error {
        return Err(error);
    }
    if app.smoke.as_ref().is_some_and(|s| s.stage != 6) {
        return Err("smoke exited before all checks completed".into());
    }
    Ok(())
}
