#include "sketch_ffi.h"

/* Independent C compiler checks of the frozen, supported 64-bit ABI 2. */
_Static_assert(sizeof(void *) == 8, "64-bit consumer required");
_Static_assert(sizeof(SketchSample) == 48, "sample size");
_Static_assert(_Alignof(SketchSample) == 8, "sample alignment");
_Static_assert(offsetof(SketchSample, sample_id) == 32, "sample identity");
_Static_assert(offsetof(SketchSample, has_pressure) == 40, "pressure presence");
_Static_assert(offsetof(SketchSample, kind) == 44, "sample kind");
_Static_assert(sizeof(SketchBrush) == 40, "brush size");
_Static_assert(offsetof(SketchBrush, width) == 32, "brush width");
_Static_assert(sizeof(SketchBounds) == 32, "bounds size");
_Static_assert(sizeof(SketchState) == 64, "state size");
_Static_assert(offsetof(SketchState, can_undo) == 16, "history flags");
_Static_assert(offsetof(SketchState, selection) == 32, "selection bounds");
_Static_assert(sizeof(SketchPathCommand) == 96, "path size");
_Static_assert(offsetof(SketchPathCommand, verb) == 8, "path verb");
_Static_assert(offsetof(SketchPathCommand, red) == 16, "path color");
_Static_assert(offsetof(SketchPathCommand, x1) == 48, "path coordinates");
_Static_assert(sizeof(SketchSourceText) == 16, "source text");
_Static_assert(sizeof(SketchSourceVersion) == 96, "source version");
_Static_assert(offsetof(SketchSourceVersion, server_generation) == 80, "generation");
_Static_assert(offsetof(SketchSourceVersion, kind) == 88, "source kind");
