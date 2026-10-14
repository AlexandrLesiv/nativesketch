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
/* Additive ABI 2 source metadata policy. This does NOT authenticate metadata,
 * hash bytes, import a document, or change a cache. Before accepting, host must
 * verify expected selected namespace/document (even when current is NULL), request
 * identity, actual SHA256, and document validity. Namespace includes account/library.
 * Text is borrowed UTF-8, not NUL-terminated: IDs 1..256 bytes without controls;
 * parent may be empty (NULL/0 allowed). Hash is exactly 64 lowercase hex bytes.
 * kind: 0 phone draft, 1 server published. generation is draft's server base
 * (0 for phone-only), or published server sequence (>=1). Immutable revision IDs
 * bind hash, parent, kind and generation; publication requires a new revision ID.
 * Current may be NULL. All other pointers must be valid and inputs immutable
 * through the call. Output is caller-owned and disjoint from inputs. No retained
 * pointers/handles/global state; callable on any thread. Status 0/1/6 as above;
 * failure leaves output untouched. No panic unwinds across this boundary.
 * Decisions: 0 accept (advance metadata even if bytes match), 1 duplicate
 * (identical immutable metadata; retain current), 2 stale,
 * 3 conflict (requires explicit user choice), 4 wrong namespace/document.
 * Same revision with contradictory metadata is invalid (status 1). Same published
 * generation with different revisions conflicts even for identical content.
 * Only published versions use generation ordering. Draft transitions require
 * direct-parent evidence with consistent base generation; skipped/unknown ancestry
 * conflicts. Older published snapshots cannot overwrite drafts. No wallclock order.
 */
typedef struct { const uint8_t *bytes; size_t length; } SketchSourceText;
typedef struct {
    SketchSourceText namespace_id, document_id, revision_id, parent_revision_id, content_sha256;
    uint64_t server_generation;
    uint32_t kind;
} SketchSourceVersion;
uint32_t sketch_source_compare(const SketchSourceVersion *current, const SketchSourceVersion *candidate, uint32_t *out_decision);
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
/* Portable persistence for OS document providers and browser blobs. Export is an
 * immutable caller-owned .nsketch snapshot plus editor-local revision identity.
 * NULL/0 queries size successfully; insufficient capacity returns 3, writes only
 * out_count/out_revision, and never writes partial bytes. Export/import/mark_saved
 * require idle state. Limit: 64 MiB. Import validates before replacing state;
 * failure preserves document/history. Success starts a clean document session.
 * Revisions do not repeat across new/import/load within this handle lifetime.
 * Mark saved ONLY after successful durable write; cancelled/failed writes or a
 * browser download initiation must not acknowledge. Stale revision returns 2.
 * All pointers obey the serialized borrowed-input/caller-output rules above.
 */
uint32_t sketch_document_export(SketchEditor *editor, uint8_t *bytes, size_t capacity, size_t *out_count, uint64_t *out_revision);
uint32_t sketch_document_import(SketchEditor *editor, const uint8_t *bytes, size_t length);
uint32_t sketch_document_mark_saved(SketchEditor *editor, uint64_t revision);
#ifdef __cplusplus
}
#endif
#endif
