#[cfg(target_os = "android")]
mod android;
mod native;
mod offline;
mod registry;

use game_core::{Phase, game::Snapshot};
use game_renderer::{Renderer, SURFACE_LOST};
use native::{AutoreleasePool, NativeTarget};
use offline::{Action, Offline};
use registry::Registry;
use serde::Serialize;
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex, MutexGuard, OnceLock,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};

const INVALID_HANDLE: i32 = -1;
const INVALID_ARGUMENT: i32 = -2;
const BUSY: i32 = -3;
const PANIC: i32 = -4;
#[cfg(not(target_vendor = "apple"))]
const WRONG_PLATFORM: i32 = -5;

#[derive(Clone, Serialize)]
struct Status {
    phase: Phase,
    frames: u64,
    touches: u64,
    selected_lane: Option<u32>,
    surface_generation: u64,
    live_instances: usize,
    live_workers: usize,
    retained_targets: usize,
    pauses: u64,
    last_pause_frame_delta: Option<u64>,
    last_error: Option<String>,
    game: Option<Snapshot>,
    last_action_error: Option<String>,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            phase: Phase::Detached,
            frames: 0,
            touches: 0,
            selected_lane: None,
            surface_generation: 0,
            live_instances: 0,
            live_workers: 0,
            retained_targets: 0,
            pauses: 0,
            last_pause_frame_delta: None,
            last_error: None,
            game: None,
            last_action_error: None,
        }
    }
}

#[derive(Default)]
struct Gate {
    generation: u64,
    active: bool,
    paused: bool,
    destroyed: bool,
    pause_frames: Option<u64>,
    input_epoch: u64,
}

struct Control {
    gate: Mutex<Gate>,
    status: Mutex<Status>,
    sender: SyncSender<Command>,
}

enum Command {
    Attach {
        target: NativeTarget,
        generation: u64,
        width: u32,
        height: u32,
        scale: f32,
        recovery: bool,
    },
    Resize {
        generation: u64,
        width: u32,
        height: u32,
        scale: f32,
    },
    Touch {
        stamp: InputStamp,
        x: f32,
        y: f32,
    },
    Action {
        stamp: InputStamp,
        action: Action,
    },
    Wake,
}

#[derive(Clone, Copy)]
struct InputStamp {
    generation: u64,
    epoch: u64,
    round_id: Option<u64>,
}

impl InputStamp {
    fn capture(gate: &Gate, status: &Status) -> Self {
        Self {
            generation: gate.generation,
            epoch: gate.input_epoch,
            round_id: status.game.as_ref().map(|game| game.round_id),
        }
    }

    fn valid(self, gate: &Gate) -> bool {
        gate.active
            && !gate.paused
            && !gate.destroyed
            && self.generation == gate.generation
            && self.epoch == gate.input_epoch
    }
}

struct Session {
    // Field order is the native lifetime contract: drop Surface before its owner.
    renderer: Renderer,
    _target: NativeTarget,
    generation: u64,
    width: u32,
    height: u32,
    scale: f32,
}

fn release_session(session: &mut Option<Session>) {
    if let Some(Session {
        renderer,
        _target,
        generation,
        ..
    }) = session.take()
    {
        drop(renderer);
        native::trace(&format!(
            "[Rust lifetime] surface_dropped generation={generation}"
        ));
        drop(_target);
        native::trace(&format!(
            "[Rust lifetime] native_reference_released generation={generation}"
        ));
    }
}

static REGISTRY: OnceLock<Mutex<Registry<Arc<Control>>>> = OnceLock::new();
static LIVE_WORKERS: AtomicUsize = AtomicUsize::new(0);

struct WorkerCount;
impl Drop for WorkerCount {
    fn drop(&mut self) {
        LIVE_WORKERS.fetch_sub(1, Ordering::AcqRel);
    }
}

fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn registry() -> &'static Mutex<Registry<Arc<Control>>> {
    REGISTRY.get_or_init(Mutex::default)
}

fn control(handle: u64) -> Result<Arc<Control>, i32> {
    lock(registry()).get(handle).cloned().ok_or(INVALID_HANDLE)
}

fn guarded(action: impl FnOnce() -> Result<(), i32>) -> i32 {
    catch_unwind(AssertUnwindSafe(action)).map_or(PANIC, |result| result.map_or_else(|e| e, |()| 0))
}

fn enqueue(control: &Control, command: Command) -> Result<(), i32> {
    control.sender.try_send(command).map_err(|_| BUSY)
}

fn invalidate(control: &Control, destroyed: bool) {
    // Initialization and drawable acquisition never hold this gate. An existing
    // submit/present transaction completes before invalidation returns.
    let mut gate = lock(&control.gate);
    gate.generation += 1;
    gate.active = false;
    gate.destroyed |= destroyed;
    let mut status = lock(&control.status);
    status.surface_generation = gate.generation;
    status.phase = if destroyed {
        Phase::Destroyed
    } else {
        Phase::Detached
    };
}

fn frame_transaction<R, F>(
    control: &Control,
    generation: u64,
    renderer: &mut R,
    acquire: impl FnOnce(&mut R) -> Result<Option<F>, String>,
    submit: impl FnOnce(&mut R, F) -> u64,
) -> Result<bool, String> {
    {
        let gate = lock(&control.gate);
        if !gate.active || gate.paused || gate.destroyed || gate.generation != generation {
            return Ok(false);
        }
    }
    let Some(frame) = acquire(renderer)? else {
        return Ok(false);
    };
    let gate = lock(&control.gate);
    if gate.active && !gate.paused && !gate.destroyed && gate.generation == generation {
        let count = submit(renderer, frame);
        lock(&control.status).frames += count;
        Ok(true)
    } else {
        drop(gate);
        drop(frame);
        Ok(false)
    }
}

#[cfg(any(target_vendor = "apple", target_os = "android"))]
fn attach(
    handle: u64,
    target: NativeTarget,
    width: u32,
    height: u32,
    scale: f32,
) -> Result<(), i32> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err(INVALID_ARGUMENT);
    }
    let control = control(handle)?;
    invalidate(&control, false);
    let gate = lock(&control.gate);
    if gate.destroyed {
        return Err(INVALID_HANDLE);
    }
    let generation = gate.generation;
    lock(&control.status).phase = Phase::Initializing;
    match enqueue(
        &control,
        Command::Attach {
            target,
            generation,
            width,
            height,
            scale,
            recovery: false,
        },
    ) {
        Ok(()) => Ok(()),
        Err(error) => {
            let mut status = lock(&control.status);
            status.phase = Phase::Detached;
            status.last_error = Some("attachment queue is full or worker stopped".into());
            Err(error)
        }
    }
}

fn status_json(handle: u64) -> Result<String, i32> {
    let control = control(handle)?;
    let mut status = lock(&control.status).clone();
    status.live_instances = lock(registry()).len();
    status.live_workers = LIVE_WORKERS.load(Ordering::Acquire);
    status.retained_targets = NativeTarget::retained_count();
    serde_json::to_string(&status).map_err(|_| PANIC)
}

#[unsafe(no_mangle)]
pub extern "C" fn mg_create(seed: u64) -> u64 {
    catch_unwind(AssertUnwindSafe(|| {
        let (sender, receiver) = mpsc::sync_channel(64);
        let control = Arc::new(Control {
            gate: Mutex::default(),
            status: Mutex::default(),
            sender,
        });
        if LIVE_WORKERS
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < 16).then_some(count + 1)
            })
            .is_err()
        {
            return 0;
        }
        let Some(handle) = lock(registry()).insert(control.clone()) else {
            LIVE_WORKERS.fetch_sub(1, Ordering::AcqRel);
            return 0;
        };
        let worker_control = control.clone();
        if thread::Builder::new()
            .name(format!("rust-game-{handle}"))
            .spawn(move || {
                let _count = WorkerCount;
                let result =
                    catch_unwind(AssertUnwindSafe(|| worker(&worker_control, receiver, seed)));
                if result.is_err() {
                    let mut gate = lock(&worker_control.gate);
                    gate.active = false;
                    let mut status = lock(&worker_control.status);
                    status.phase = Phase::Detached;
                    status.last_error =
                        Some("render worker panicked; destroy and recreate runtime".into());
                }
            })
            .is_err()
        {
            LIVE_WORKERS.fetch_sub(1, Ordering::AcqRel);
            lock(registry()).remove(handle);
            return 0;
        }
        handle
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn mg_destroy(handle: u64) -> i32 {
    guarded(|| {
        let control = lock(registry()).remove(handle).ok_or(INVALID_HANDLE)?;
        invalidate(&control, true);
        let _ = enqueue(&control, Command::Wake);
        Ok(())
    })
}

/// # Safety
/// The caller must supply a live CAMetalLayer, not an arbitrary pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mg_attach_metal(
    handle: u64,
    layer: *mut c_void,
    width: u32,
    height: u32,
    scale: f32,
) -> i32 {
    guarded(|| {
        if layer.is_null() {
            return Err(INVALID_ARGUMENT);
        }
        control(handle)?;
        #[cfg(target_vendor = "apple")]
        {
            // SAFETY: The C contract requires a live layer during this call.
            attach(
                handle,
                unsafe { NativeTarget::retain_layer(layer) },
                width,
                height,
                scale,
            )
        }
        #[cfg(not(target_vendor = "apple"))]
        {
            let _ = (width, height, scale);
            Err(WRONG_PLATFORM)
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mg_detach(handle: u64) -> i32 {
    guarded(|| {
        let control = control(handle)?;
        invalidate(&control, false);
        let _ = enqueue(&control, Command::Wake);
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mg_pause(handle: u64, paused: u32) -> i32 {
    guarded(|| {
        if paused > 1 {
            return Err(INVALID_ARGUMENT);
        }
        let control = control(handle)?;
        let mut gate = lock(&control.gate);
        if gate.destroyed {
            return Err(INVALID_HANDLE);
        }
        let paused = paused != 0;
        let mut status = lock(&control.status);
        if paused && !gate.paused {
            gate.pause_frames = Some(status.frames);
            status.pauses += 1;
        } else if !paused && gate.paused {
            status.last_pause_frame_delta =
                gate.pause_frames.take().map(|start| status.frames - start);
        }
        if paused != gate.paused {
            gate.input_epoch += 1;
        }
        gate.paused = paused;
        if gate.active {
            status.phase = if gate.paused {
                Phase::Paused
            } else {
                Phase::Running
            };
        }
        let _ = enqueue(&control, Command::Wake);
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mg_resize(handle: u64, width: u32, height: u32, scale: f32) -> i32 {
    guarded(|| {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(INVALID_ARGUMENT);
        }
        let control = control(handle)?;
        let gate = lock(&control.gate);
        if gate.destroyed {
            return Err(INVALID_HANDLE);
        }
        enqueue(
            &control,
            Command::Resize {
                generation: gate.generation,
                width,
                height,
                scale,
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mg_touch(handle: u64, x: f32, y: f32) -> i32 {
    guarded(|| {
        if !x.is_finite() || !y.is_finite() {
            return Err(INVALID_ARGUMENT);
        }
        let control = control(handle)?;
        let gate = lock(&control.gate);
        if gate.destroyed {
            return Err(INVALID_HANDLE);
        }
        if !gate.active || gate.paused {
            return Err(BUSY);
        }
        enqueue(
            &control,
            Command::Touch {
                stamp: InputStamp::capture(&gate, &lock(&control.status)),
                x,
                y,
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mg_action(handle: u64, action: u32, argument: u32) -> i32 {
    guarded(|| {
        let action = Action::parse(action, argument)?;
        let control = control(handle)?;
        let gate = lock(&control.gate);
        if gate.destroyed {
            return Err(INVALID_HANDLE);
        }
        if matches!(action, Action::Choose { .. }) && (!gate.active || gate.paused) {
            return Err(BUSY);
        }
        enqueue(
            &control,
            Command::Action {
                stamp: InputStamp::capture(&gate, &lock(&control.status)),
                action,
            },
        )
    })
}

/// # Safety
/// `buffer` must be writable for `capacity` bytes without aliasing concurrent accesses.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mg_status(handle: u64, buffer: *mut u8, capacity: u32) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        if buffer.is_null() || capacity == 0 {
            return Err(INVALID_ARGUMENT);
        }
        let value = status_json(handle)?;
        if value.len() + 1 > capacity as usize {
            return Err(INVALID_ARGUMENT);
        }
        // SAFETY: The caller guarantees a writable capacity-sized buffer. The
        // serialized value is a separate allocation and the length was checked.
        unsafe {
            std::ptr::copy_nonoverlapping(value.as_ptr(), buffer, value.len());
            *buffer.add(value.len()) = 0;
        }
        Ok(value.len() as i32)
    }))
    .map_or(PANIC, |result| result.unwrap_or_else(|e| e))
}

fn process_input(
    control: &Control,
    offline: &mut Offline,
    command: Command,
    metrics: Option<(u32, u32)>,
    now_ms: u64,
) {
    match command {
        Command::Touch { stamp, x, y } => {
            let gate = lock(&control.gate);
            if stamp.valid(&gate)
                && let Some((width, height)) = metrics
            {
                offline.touch(x, y, width, height, stamp.round_id, now_ms);
            }
        }
        Command::Action { stamp, action } => {
            let mut gate = lock(&control.gate);
            if matches!(action, Action::Reset | Action::Mode { .. }) {
                if !gate.destroyed && gate.generation == stamp.generation {
                    offline.apply(action, stamp.round_id, now_ms);
                    if matches!(action, Action::Mode { .. }) {
                        gate.input_epoch += 1;
                    }
                }
            } else if !gate.destroyed
                && gate.generation == stamp.generation
                && !gate.paused
                && (!matches!(action, Action::Choose { .. }) || stamp.valid(&gate))
            {
                offline.apply(action, stamp.round_id, now_ms);
            } else {
                offline.last_action_error =
                    Some("Action invalidated by lifecycle or mode change".into());
            }
        }
        _ => unreachable!("only input commands are dispatched here"),
    }
}

fn now_ms(clock: Instant) -> u64 {
    clock.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}

fn worker(control: &Control, receiver: Receiver<Command>, seed: u64) {
    let clock = Instant::now();
    let mut session: Option<Session> = None;
    let mut offline = Offline::new(seed);
    let mut pending_recovery = None;
    let mut recovery_count = 0u32;
    loop {
        let _pool = AutoreleasePool::new();
        let command = pending_recovery
            .take()
            .map_or_else(|| receiver.recv_timeout(Duration::from_millis(16)), Ok);
        {
            let gate = lock(&control.gate);
            if gate.destroyed {
                break;
            }
            if session
                .as_ref()
                .is_some_and(|s| s.generation != gate.generation)
            {
                drop(gate);
                release_session(&mut session);
            }
        }
        if let Ok(command) = command {
            match command {
                Command::Attach {
                    target,
                    generation,
                    width,
                    height,
                    scale,
                    recovery,
                } => {
                    if lock(&control.gate).generation != generation {
                        continue;
                    }
                    release_session(&mut session);
                    if !recovery {
                        recovery_count = 0;
                    }
                    let result = (|| {
                        let instance = wgpu::Instance::new(
                            wgpu::InstanceDescriptor::new_without_display_handle(),
                        );
                        let surface = target.surface(&instance)?;
                        let renderer =
                            pollster::block_on(Renderer::new(&instance, surface, width, height))?;
                        Ok::<_, String>(Session {
                            renderer,
                            _target: target,
                            generation,
                            width,
                            height,
                            scale,
                        })
                    })();
                    let mut gate = lock(&control.gate);
                    if gate.generation != generation || gate.destroyed {
                        continue;
                    }
                    let mut status = lock(&control.status);
                    match result {
                        Ok(value) => {
                            session = Some(value);
                            gate.active = true;
                            status.phase = if gate.paused {
                                Phase::Paused
                            } else {
                                Phase::Running
                            };
                            status.last_error = None;
                        }
                        Err(error) => {
                            status.phase = Phase::Detached;
                            status.last_error = Some(error);
                        }
                    }
                }
                Command::Resize {
                    generation,
                    width,
                    height,
                    scale,
                } => {
                    let gate = lock(&control.gate);
                    if gate.destroyed || gate.generation != generation {
                        continue;
                    }
                    if let Some(s) = session.as_mut().filter(|s| s.generation == generation) {
                        s.width = width;
                        s.height = height;
                        s.scale = scale;
                        s.renderer.resize(width, height);
                    }
                }
                command @ (Command::Touch { .. } | Command::Action { .. }) => {
                    let metrics = session.as_ref().map(|s| (s.width, s.height));
                    process_input(control, &mut offline, command, metrics, now_ms(clock));
                }
                Command::Wake => {}
            }
        }
        let mut render_failed = false;
        if lock(&control.gate).destroyed {
            break;
        }
        let now_ms = now_ms(clock);
        offline.advance(now_ms);
        {
            let mut status = lock(&control.status);
            status.touches = offline.probe.touches;
            status.selected_lane = offline.probe.selected_lane;
            status.game = offline.snapshot();
            status
                .last_action_error
                .clone_from(&offline.last_action_error);
        }
        if let Some(s) = session.as_mut().filter(|s| s.width > 0 && s.height > 0) {
            let scene = offline.scene(s.width, s.height, s.scale, now_ms);
            let result = frame_transaction(
                control,
                s.generation,
                &mut s.renderer,
                |renderer| renderer.prepare_frame(&scene),
                |renderer, frame| {
                    let before = renderer.presented_frames();
                    renderer.submit_frame(frame);
                    renderer.presented_frames() - before
                },
            );
            if let Err(error) = result {
                render_failed = true;
                let mut gate = lock(&control.gate);
                if gate.generation == s.generation && !gate.destroyed {
                    let mut status = lock(&control.status);
                    status.last_error = Some(error.clone());
                    gate.active = false;
                    status.phase = Phase::Detached;
                    if error == SURFACE_LOST && recovery_count < 3 {
                        recovery_count += 1;
                        status.phase = Phase::Initializing;
                        pending_recovery = Some(Command::Attach {
                            target: s._target.duplicate(),
                            generation: s.generation,
                            width: s.width,
                            height: s.height,
                            scale: s.scale,
                            recovery: true,
                        });
                    }
                }
            }
        }
        if render_failed {
            release_session(&mut session);
        }
    }
    release_session(&mut session);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_fixture() -> (u64, Arc<Control>, Receiver<Command>, Offline) {
        let (sender, receiver) = mpsc::sync_channel(64);
        let mut offline = Offline::new(42);
        offline.apply(Action::Mode { race: true }, None, 0);
        offline.apply(Action::Start, None, 0);
        let control = Arc::new(Control {
            gate: Mutex::new(Gate {
                active: true,
                generation: 1,
                ..Default::default()
            }),
            status: Mutex::new(Status {
                game: offline.snapshot(),
                ..Default::default()
            }),
            sender,
        });
        let handle = lock(registry()).insert(control.clone()).unwrap();
        (handle, control, receiver, offline)
    }

    #[test]
    fn action_acceptance_is_separate_from_application_and_gpu_errors() {
        let (handle, control, receiver, mut offline) = input_fixture();
        lock(&control.status).last_error = Some("GPU failure fixture".into());
        for (action, argument) in [(5, 0), (1, 2), (2, 1), (3, 3), (3, u32::MAX), (4, 1)] {
            assert_eq!(mg_action(handle, action, argument), INVALID_ARGUMENT);
            assert!(receiver.try_recv().is_err());
        }
        assert_eq!(mg_action(handle, 3, 1), 0);
        assert_eq!(offline.snapshot().unwrap().players[1].choice, None);
        process_input(
            &control,
            &mut offline,
            receiver.try_recv().unwrap(),
            None,
            3_000,
        );
        assert_eq!(offline.last_action_error.as_deref(), Some("InvalidPhase"));
        assert_eq!(
            lock(&control.status).last_error.as_deref(),
            Some("GPU failure fixture")
        );
        assert_eq!(mg_action(handle, 0, u32::MAX), 0);
        process_input(
            &control,
            &mut offline,
            receiver.try_recv().unwrap(),
            None,
            3_001,
        );
        assert!(offline.last_action_error.is_none());
        assert_eq!(mg_destroy(handle), 0);
    }

    #[test]
    fn delayed_touch_and_native_choices_are_discarded_after_lifecycle_changes() {
        for native in [false, true] {
            for change in 0..5 {
                let (handle, control, receiver, mut offline) = input_fixture();
                let accepted = if native {
                    mg_action(handle, 3, 2)
                } else {
                    mg_touch(handle, 250.0, 100.0)
                };
                assert_eq!(accepted, 0);
                let command = receiver.try_recv().unwrap();
                match change {
                    0 => {
                        assert_eq!(mg_pause(handle, 1), 0);
                    }
                    1 => {
                        assert_eq!(mg_pause(handle, 1), 0);
                        assert_eq!(mg_pause(handle, 0), 0);
                    }
                    2 => {
                        assert_eq!(mg_detach(handle), 0);
                        lock(&control.gate).active = true;
                    }
                    3 => {
                        assert_eq!(mg_destroy(handle), 0);
                    }
                    _ => {
                        let stamp =
                            InputStamp::capture(&lock(&control.gate), &lock(&control.status));
                        process_input(
                            &control,
                            &mut offline,
                            Command::Action {
                                stamp,
                                action: Action::Mode { race: false },
                            },
                            None,
                            1,
                        );
                        offline.apply(Action::Mode { race: true }, None, 1);
                    }
                }
                process_input(&control, &mut offline, command, Some((300, 400)), 2);
                let snapshot = offline.snapshot().unwrap();
                assert_eq!(snapshot.round_id, 1);
                assert!(snapshot.players.iter().all(|p| p.choice.is_none()));
                assert_eq!(offline.probe.touches, 0);
                if change != 3 {
                    assert_eq!(mg_destroy(handle), 0);
                }
            }
        }
    }

    #[test]
    fn recreated_surface_keeps_the_room_and_accepts_only_new_input() {
        let (handle, control, receiver, mut offline) = input_fixture();
        assert_eq!(mg_touch(handle, 150.0, 100.0), 0);
        process_input(
            &control,
            &mut offline,
            receiver.try_recv().unwrap(),
            Some((300, 400)),
            1,
        );
        let before = offline.snapshot().unwrap();
        assert_eq!(mg_detach(handle), 0);
        assert_eq!(mg_touch(handle, 150.0, 100.0), BUSY);
        assert_eq!(mg_action(handle, 3, 1), BUSY);
        assert_eq!(offline.snapshot().unwrap(), before);
        let _wake = receiver.try_recv().unwrap();
        lock(&control.gate).active = true;
        assert_eq!(mg_action(handle, 3, 2), 0);
        process_input(
            &control,
            &mut offline,
            receiver.try_recv().unwrap(),
            Some((600, 800)),
            2,
        );
        assert_eq!(offline.snapshot().unwrap().players[1].choice, Some(2));
        assert_eq!(mg_destroy(handle), 0);
    }

    #[test]
    fn worker_publishes_seed_snapshot_and_game_error_through_the_existing_status() {
        let handle = mg_create(987);
        assert_eq!(mg_action(handle, 1, 1), 0);
        assert_eq!(mg_action(handle, 4, 0), 0);
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let status: serde_json::Value =
                serde_json::from_str(&status_json(handle).unwrap()).unwrap();
            if status["last_action_error"] == "InvalidPhase" {
                assert_eq!(status["game"]["seed"], 987);
                assert_eq!(status["game"]["phase"], "Waiting");
                assert!(status["last_error"].is_null());
                for key in [
                    "touches",
                    "selected_lane",
                    "frames",
                    "phase",
                    "surface_generation",
                ] {
                    assert!(status.get(key).is_some());
                }
                break;
            }
            assert!(
                Instant::now() < deadline,
                "worker did not publish the action result: {status}"
            );
            thread::yield_now();
        }
        assert_eq!(mg_destroy(handle), 0);
    }

    #[test]
    fn ffi_rejects_invalid_lifetime_and_buffer_contracts() {
        let handle = mg_create(1);
        assert_ne!(handle, 0);
        let mut buffer = [0u8; 1024];
        // SAFETY: buffer is valid and exclusively owned for the full call.
        let length = unsafe { mg_status(handle, buffer.as_mut_ptr(), buffer.len() as u32) };
        assert!(length > 0);
        assert_eq!(buffer[length as usize], 0);
        assert!(serde_json::from_slice::<serde_json::Value>(&buffer[..length as usize]).is_ok());
        // SAFETY: Null is explicitly rejected before any dereference.
        assert_eq!(
            unsafe { mg_status(handle, std::ptr::null_mut(), 10) },
            INVALID_ARGUMENT
        );
        // SAFETY: The one-byte buffer is valid; insufficient capacity is rejected.
        assert_eq!(
            unsafe { mg_status(handle, buffer.as_mut_ptr(), 1) },
            INVALID_ARGUMENT
        );
        assert_eq!(mg_touch(handle, f32::NAN, 0.0), INVALID_ARGUMENT);
        assert_eq!(mg_pause(handle, 2), INVALID_ARGUMENT);
        assert_eq!(mg_resize(handle, 0, 0, 0.0), INVALID_ARGUMENT);
        assert_eq!(mg_detach(handle), 0);
        assert_eq!(mg_destroy(handle), 0);
        assert_eq!(mg_destroy(handle), INVALID_HANDLE);
        assert_eq!(mg_touch(handle, 0.0, 0.0), INVALID_HANDLE);
        assert_eq!(mg_pause(handle, 0), INVALID_HANDLE);
    }

    #[test]
    fn panic_is_contained_at_export_guard() {
        assert_eq!(guarded(|| panic!("contract test")), PANIC);
    }

    #[test]
    fn full_queue_does_not_prevent_immediate_drawing_invalidation() {
        let (sender, _receiver) = mpsc::sync_channel(1);
        let control = Control {
            gate: Mutex::default(),
            status: Mutex::default(),
            sender,
        };
        assert_eq!(enqueue(&control, Command::Wake), Ok(()));
        assert_eq!(enqueue(&control, Command::Wake), Err(BUSY));
        lock(&control.gate).active = true;
        invalidate(&control, false);
        assert!(!lock(&control.gate).active);
    }

    #[test]
    fn lifecycle_calls_finish_while_acquisition_waits_and_discard_the_late_frame() {
        struct Frame {
            control: Arc<Control>,
            discarded: Arc<AtomicUsize>,
        }
        impl Drop for Frame {
            fn drop(&mut self) {
                assert!(self.control.gate.try_lock().is_ok());
                self.discarded.fetch_add(1, Ordering::AcqRel);
            }
        }
        for operation in 0..3 {
            let (sender, _receiver) = mpsc::sync_channel(1);
            let control = Arc::new(Control {
                gate: Mutex::new(Gate {
                    active: true,
                    generation: 1,
                    ..Default::default()
                }),
                status: Mutex::default(),
                sender,
            });
            let handle = lock(registry()).insert(control.clone()).unwrap();
            let (entered_tx, entered_rx) = mpsc::sync_channel(1);
            let (release_tx, release_rx) = mpsc::sync_channel(1);
            let submitted = Arc::new(AtomicUsize::new(0));
            let discarded = Arc::new(AtomicUsize::new(0));
            let worker_control = control.clone();
            let worker_submitted = submitted.clone();
            let worker_discarded = discarded.clone();
            let worker = thread::spawn(move || {
                frame_transaction(
                    &worker_control,
                    1,
                    &mut (),
                    |_| {
                        entered_tx.send(()).unwrap();
                        release_rx.recv().unwrap();
                        Ok(Some(Frame {
                            control: worker_control.clone(),
                            discarded: worker_discarded,
                        }))
                    },
                    |_, _frame| {
                        worker_submitted.fetch_add(1, Ordering::AcqRel);
                        1
                    },
                )
            });
            entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            let (finished_tx, finished_rx) = mpsc::sync_channel(1);
            let operation_thread = thread::spawn(move || {
                let result = match operation {
                    0 => mg_pause(handle, 1),
                    1 => mg_detach(handle),
                    _ => mg_destroy(handle),
                };
                finished_tx.send(result).unwrap();
            });
            assert_eq!(finished_rx.recv_timeout(Duration::from_secs(2)).unwrap(), 0);
            release_tx.send(()).unwrap();
            assert!(!worker.join().unwrap().unwrap());
            operation_thread.join().unwrap();
            assert_eq!(submitted.load(Ordering::Acquire), 0);
            assert_eq!(discarded.load(Ordering::Acquire), 1);
            assert_eq!(lock(&control.status).frames, 0);
            if operation != 2 {
                assert_eq!(mg_destroy(handle), 0);
            }
        }
    }

    #[test]
    fn invalidation_does_not_wait_for_initialization() {
        let (sender, _receiver) = mpsc::sync_channel(1);
        let control = Control {
            gate: Mutex::default(),
            status: Mutex::default(),
            sender,
        };
        lock(&control.status).phase = Phase::Initializing;
        invalidate(&control, false);
        let gate = lock(&control.gate);
        assert_eq!(gate.generation, 1);
        assert!(!gate.active);
        assert_eq!(lock(&control.status).phase, Phase::Detached);
    }
}
