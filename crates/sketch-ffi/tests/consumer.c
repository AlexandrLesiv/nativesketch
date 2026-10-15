#include "consumer_layout.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define CASE "legacy-abi2-conformance"
/* Explicit checks remain enabled even when the compiler receives NDEBUG. */
#define CHECK(condition) do { \
    if (!(condition)) { \
        fprintf(stderr, "FAIL line %d: %s\n", __LINE__, #condition); \
        exit(1); \
    } \
} while (0)

static unsigned char *export_document(SketchEditor *editor, size_t *length,
                                      uint64_t *revision) {
    CHECK(sketch_document_export(editor, NULL, 0, length, revision) == 0);
    unsigned char *bytes = malloc(*length);
    CHECK(bytes != NULL);
    CHECK(sketch_document_export(editor, bytes, *length, length, revision) == 0);
    return bytes;
}

static void same_document(SketchEditor *editor, const unsigned char *expected,
                          size_t length) {
    size_t actual_length = 0;
    uint64_t revision = 0;
    unsigned char *actual = export_document(editor, &actual_length, &revision);
    CHECK(actual_length == length);
    CHECK(memcmp(actual, expected, length) == 0);
    free(actual);
}

static SketchPathCommand *scene(SketchEditor *editor, size_t *count) {
    CHECK(sketch_render_copy(editor, NULL, 0, count) == 0);
    SketchPathCommand *commands = calloc(*count, sizeof(*commands));
    CHECK(commands != NULL);
    CHECK(sketch_render_copy(editor, commands, *count, count) == 0);
    return commands;
}

static void same_scene(SketchEditor *editor, const SketchPathCommand *expected,
                       size_t count) {
    size_t actual_count = 0;
    SketchPathCommand *actual = scene(editor, &actual_count);
    CHECK(actual_count == count);
    for (size_t i = 0; i < count; ++i) {
        CHECK(actual[i].stroke_id == expected[i].stroke_id);
        CHECK(actual[i].verb == expected[i].verb);
        CHECK(actual[i].red == expected[i].red);
        CHECK(actual[i].green == expected[i].green);
        CHECK(actual[i].blue == expected[i].blue);
        CHECK(actual[i].alpha == expected[i].alpha);
        CHECK(actual[i].x1 == expected[i].x1 && actual[i].y1 == expected[i].y1);
        CHECK(actual[i].x2 == expected[i].x2 && actual[i].y2 == expected[i].y2);
        CHECK(actual[i].x3 == expected[i].x3 && actual[i].y3 == expected[i].y3);
    }
    free(actual);
}

static void conformance(const char *fixture) {
    FILE *file = fopen(fixture, "rb");
    CHECK(file != NULL);
    unsigned char golden[4096];
    size_t length = fread(golden, 1, sizeof(golden), file);
    CHECK(feof(file) && !ferror(file) && length > 24);
    CHECK(fclose(file) == 0);
    CHECK(sketch_abi_version() == 2);
    CHECK(sketch_editor_create(NULL) == 1);
    CHECK(sketch_cancel(NULL) == 1);
    SketchEditor *editor = NULL;
    CHECK(sketch_editor_create(&editor) == 0 && editor != NULL);
    SketchBrush brush = {0.2, 0.3, 0.4, 0.5, 6.0};
    CHECK(sketch_stroke_begin(editor, 42, 1, brush) == 0);
    SketchSample samples[] = {
        {10.0, 20.0, 1.0, 0.5, 1, 1, 0},
        {30.0, 40.0, 2.0, 0.7, 2, 1, 0},
        {80.0, 90.0, 3.0, 0.9, 3, 1, 1}
    };
    CHECK(sketch_stroke_append(editor, 42, samples, 3) == 0);
    samples[0].x = 999.0; /* The bridge must have copied borrowed inputs. */
    size_t count = 0;
    SketchPathCommand *before = scene(editor, &count);
    SketchSample invalid[] = {
        {50.0, 60.0, 3.0, 0.5, 3, 1, 0},
        {NAN, 70.0, 4.0, 0.5, 4, 1, 0}
    };
    CHECK(sketch_stroke_append(editor, 42, invalid, 2) == 1);
    same_scene(editor, before, count);
    free(before);
    CHECK(sketch_undo(editor) == 2);
    CHECK(sketch_stroke_commit(editor, 42) == 0);
    same_document(editor, golden, length); /* Predictions cannot be serialized. */
    size_t required = 0;
    uint64_t revision = 0;
    unsigned char sentinel[4096];
    memset(sentinel, 0xa5, sizeof(sentinel));
    CHECK(sketch_document_export(editor, sentinel, length - 1, &required, &revision) == 3);
    CHECK(required == length);
    for (size_t i = 0; i < sizeof(sentinel); ++i) CHECK(sentinel[i] == 0xa5);
    CHECK(sketch_document_mark_saved(editor, revision) == 0);
    SketchState state = {0};
    CHECK(sketch_editor_state(editor, &state) == 0 && !state.dirty);
    CHECK(sketch_stroke_begin(editor, 42, 1, brush) == 0);
    CHECK(sketch_stroke_append(editor, 42, samples + 1, 1) == 0);
    CHECK(sketch_cancel(editor) == 0);
    same_document(editor, golden, length);
    CHECK(sketch_document_import(editor, golden, length - 1) == 5);
    same_document(editor, golden, length);
    CHECK(sketch_undo(editor) == 0);
    CHECK(sketch_editor_state(editor, &state) == 0);
    CHECK(state.stroke_count == 0 && !state.can_undo && state.can_redo);
    CHECK(sketch_redo(editor) == 0);
    same_document(editor, golden, length);
    CHECK(sketch_select_cycle(editor, 1) == 0);
    CHECK(sketch_delete_selected(editor) == 0);
    CHECK(sketch_document_mark_saved(editor, revision) == 2);
    CHECK(sketch_undo(editor) == 0);
    CHECK(sketch_document_mark_saved(editor, revision) == 0);
    CHECK(sketch_document_new(editor) == 0);
    CHECK(sketch_document_mark_saved(editor, revision) == 2);
    CHECK(sketch_document_import(editor, golden, length) == 0);
    CHECK(sketch_document_mark_saved(editor, revision) == 2);
    unsigned char original[4096];
    memcpy(original, golden, length);
    memset(golden, 0, length); /* Imported document must own its bytes. */
    same_document(editor, original, length);
    sketch_editor_free(editor);
    sketch_editor_free(NULL);
}

int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "--list") == 0) {
        puts(CASE);
        return 0;
    }
    if (argc == 2 && strcmp(argv[1], "--negative-oracle") == 0) {
        /* Deliberately wrong expectation proves actual linked failures propagate. */
        CHECK(sketch_cancel(NULL) == 0);
        return 0;
    }
    if (argc != 4 || strcmp(argv[1], "--run") || strcmp(argv[2], CASE)) {
        fputs("Unknown case or arguments\n", stderr);
        return 2;
    }
    conformance(argv[3]);
    puts("PASS " CASE);
    return 0;
}
