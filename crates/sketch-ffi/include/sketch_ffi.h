#ifndef NATIVE_SKETCH_FFI_H
#define NATIVE_SKETCH_FFI_H
#include <stdint.h>
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif
/* ABI 2. Handles are uniquely owned; free exactly once. Calls (including free) must
 * be serialized by the host, on any thread. Never use a freed handle. All input
 * pointers are borrowed only during the call; outputs use caller-owned storage.
 * Valid pointers must address the stated lengths. No Rust exception unwinds here.
 * Status: 0 success, 1 invalid argument, 2 invalid state, 3 insufficient capacity,
 * 4 I/O error, 5 invalid/unsupported document, 6 caught panic. On panic discard
 * the handle. Failed operations otherwise leave committed state unchanged.
 * Paths are UTF-8 bytes without NUL, length explicit. Save/load are synchronous;
 * host must invoke outside ink delivery and serialize/disable interaction.
 */
typedef struct SketchEditor SketchEditor;
typedef struct { double x, y, timestamp, pressure; uint64_t sample_id; uint32_t has_pressure, kind; } SketchSample;
/* sample kind: 0 actual, 1 predicted (temporary), 2 correction by sample_id.
 * timestamps are monotonic seconds; x/y document-space doubles. Missing pressure
 * gives base width; supplied [0,1] pressure gives width*(0.2+0.8*pressure).
 * Device: 0 mouse, 1 pen, 2 touch. Pointer id is stable through a gesture.
 */
typedef struct { double red, green, blue, alpha, width; } SketchBrush;
typedef struct { double x, y, width, height; } SketchBounds;
typedef struct {
    uint64_t stroke_count, selected_id;
    uint32_t can_undo, can_redo, dirty, interaction; /* interaction 0 idle, 1 ink, 2 resize */
    SketchBounds selection;
} SketchState;
/* Group contiguous commands by stroke_id. Fill one path per stroke with nonzero
 * winding; overlapping subpaths form the stroke without alpha accumulation.
 * verb: 0 move(x1,y1), 1 line(x1,y1), 2 cubic(control1,control2,end), 3 close.
 * Geometry includes transient input/transform. All values are document-space.
 */
typedef struct {
    uint64_t stroke_id;
    uint32_t verb;
    double red, green, blue, alpha;
    double x1, y1, x2, y2, x3, y3;
} SketchPathCommand;
uint32_t sketch_abi_version(void);
uint32_t sketch_editor_create(SketchEditor **out_editor);
void sketch_editor_free(SketchEditor *editor);
uint32_t sketch_editor_state(SketchEditor *editor, SketchState *out_state);
uint32_t sketch_stroke_begin(SketchEditor *editor, uint64_t pointer_id, uint32_t device, SketchBrush brush);
uint32_t sketch_stroke_append(SketchEditor *editor, uint64_t pointer_id, const SketchSample *samples, size_t count);
uint32_t sketch_stroke_commit(SketchEditor *editor, uint64_t pointer_id);
uint32_t sketch_cancel(SketchEditor *editor);
/* Query required count with NULL/0; returns success. Insufficient capacity writes
 * required count and no commands. Copy/state calls never change the document. */
uint32_t sketch_render_copy(SketchEditor *editor, SketchPathCommand *commands, size_t capacity, size_t *out_count);
uint32_t sketch_select_at(SketchEditor *editor, double x, double y, double tolerance);
uint32_t sketch_select_cycle(SketchEditor *editor, int32_t direction); /* +1 next, -1 previous */
uint32_t sketch_select_clear(SketchEditor *editor);
uint32_t sketch_delete_selected(SketchEditor *editor);
uint32_t sketch_resize_begin(SketchEditor *editor);
/* Absolute uniform scale relative to begin, anchored at document-space point.
 * Includes stroke width; finite positive scale only. Repeated updates replace preview. */
uint32_t sketch_resize_update(SketchEditor *editor, double scale, double anchor_x, double anchor_y);
uint32_t sketch_resize_commit(SketchEditor *editor);
uint32_t sketch_undo(SketchEditor *editor);
uint32_t sketch_redo(SketchEditor *editor);
uint32_t sketch_document_new(SketchEditor *editor);
uint32_t sketch_document_save(SketchEditor *editor, const uint8_t *path, size_t length);
uint32_t sketch_document_load(SketchEditor *editor, const uint8_t *path, size_t length);
#ifdef __cplusplus
}
#endif
#endif
