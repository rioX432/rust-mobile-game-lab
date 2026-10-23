#ifndef GAME_RUNTIME_H
#define GAME_RUNTIME_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Calls enqueue bounded work. Negative values are errors: -1 invalid handle,
 * -2 invalid argument, -3 busy, -4 panic, -5 wrong platform. */
uint64_t mg_create(uint64_t seed);
int32_t mg_destroy(uint64_t handle);
/* Caller supplies a valid CAMetalLayer for this call. Rust retains it until
 * its Surface is dropped. GPU initialization runs on the Rust worker. */
int32_t mg_attach_metal(uint64_t handle, void *layer, uint32_t width, uint32_t height, float scale);
int32_t mg_resize(uint64_t handle, uint32_t width, uint32_t height, float scale);
int32_t mg_touch(uint64_t handle, float x_pixels, float y_pixels);
int32_t mg_pause(uint64_t handle, uint32_t paused);
int32_t mg_detach(uint64_t handle);
/* Actions: 0 = Probe reset (argument ignored), 1 = mode (0 Probe / 1 Race),
 * 2 = Start/next round (argument 0), 3 = Choose (argument 0..2),
 * 4 = host early Close (argument 0). Start/Close act as host 1; Choose/touch
 * act as viewer 2. Mode changes and Surface recreation preserve the room.
 * Return 0 means queued, not applied. Inspect status.last_action_error for
 * game/lifecycle rejection; status.last_error is reserved for GPU/worker errors.
 * status.game is null in Probe, otherwise the common Rust Snapshot.
 * Choose/touch require an active, unpaused Surface and retain their round.
 * Choices/touches invalidated by pause, detach, destroy or mode changes are discarded. */
int32_t mg_action(uint64_t handle, uint32_t action, uint32_t argument);
/* Writes UTF-8 JSON with trailing NUL; returns byte count excluding NUL.
 * buffer must be writable for capacity bytes. No Rust pointer crosses ABI. */
int32_t mg_status(uint64_t handle, char *buffer, uint32_t capacity);
#ifdef __cplusplus
}
#endif
#endif
