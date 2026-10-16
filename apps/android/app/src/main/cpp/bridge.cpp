#include "sketch_ffi.h"
#include <jni.h>
#include <new>
#include <vector>
#define JNI(name) extern "C" JNIEXPORT
static SketchEditor *ptr(jlong h) {
  return reinterpret_cast<SketchEditor *>(h);
}
static void fail(JNIEnv *e) {
  e->ThrowNew(e->FindClass("java/lang/IllegalStateException"),
              "Rust engine operation failed");
}
JNI(create)
jlong JNICALL Java_dev_nativesketch_Engine_create(JNIEnv *e, jobject) {
  SketchEditor *p = nullptr;
  if (sketch_abi_version() != 2 || sketch_editor_create(&p)) {
    fail(e);
    return 0;
  }
  return reinterpret_cast<jlong>(p);
}
JNI(free)
void JNICALL Java_dev_nativesketch_Engine_free(JNIEnv *, jobject, jlong h) {
  sketch_editor_free(ptr(h));
}
JNI(state)
jdoubleArray JNICALL
    Java_dev_nativesketch_Engine_state(JNIEnv *e, jobject, jlong h) {
  SketchState s{};
  if (sketch_editor_state(ptr(h), &s)) {
    fail(e);
    return nullptr;
  }
  double v[] = {double(s.stroke_count), double(s.selected_id),
                double(s.can_undo),     double(s.can_redo),
                double(s.dirty),        double(s.interaction),
                s.selection.x,          s.selection.y,
                s.selection.width,      s.selection.height};
  auto a = e->NewDoubleArray(10);
  if (!a)
    return nullptr;
  e->SetDoubleArrayRegion(a, 0, 10, v);
  return a;
}
JNI(action)
jint JNICALL
    Java_dev_nativesketch_Engine_action(JNIEnv *, jobject, jlong h, jint op,
                                        jdouble a, jdouble b, jdouble c) {
  auto p = ptr(h);
  switch (op) {
  case 0:
    return sketch_cancel(p);
  case 1:
    return sketch_stroke_commit(p, (uint64_t)a);
  case 2:
    return sketch_select_at(p, a, b, c);
  case 3:
    return sketch_select_cycle(p, (int32_t)a);
  case 4:
    return sketch_delete_selected(p);
  case 5:
    return sketch_resize_begin(p);
  case 6:
    return sketch_resize_update(p, a, b, c);
  case 7:
    return sketch_resize_commit(p);
  case 8:
    return sketch_undo(p);
  case 9:
    return sketch_redo(p);
  case 10:
    return sketch_document_new(p);
  case 11:
    return sketch_select_clear(p);
  default:
    return 1;
  }
}
JNI(begin)
jint JNICALL
    Java_dev_nativesketch_Engine_begin(JNIEnv *, jobject, jlong h, jint pointer,
                                       jint device, jint color, jdouble width) {
  SketchBrush b{((color >> 16) & 255) / 255., ((color >> 8) & 255) / 255.,
                (color & 255) / 255., ((color >> 24) & 255) / 255., width};
  return sketch_stroke_begin(ptr(h), pointer, device, b);
}
JNI(sample)
jint JNICALL
    Java_dev_nativesketch_Engine_sample(JNIEnv *, jobject, jlong h,
                                        jint pointer, jdouble x, jdouble y,
                                        jdouble time, jdouble pressure,
                                        jboolean has, jlong id) {
  SketchSample s{x, y, time, pressure, (uint64_t)id, (uint32_t)has, 0};
  return sketch_stroke_append(ptr(h), pointer, &s, 1);
}
JNI(render)
jdoubleArray JNICALL
    Java_dev_nativesketch_Engine_render(JNIEnv *e, jobject, jlong h) {
  size_t n = 0;
  if (sketch_render_copy(ptr(h), nullptr, 0, &n)) {
    fail(e);
    return nullptr;
  }
  try {
    std::vector<SketchPathCommand> c(n);
    if (sketch_render_copy(ptr(h), c.data(), n, &n)) {
      fail(e);
      return nullptr;
    }
    std::vector<double> v;
    v.reserve(n * 12);
    uint64_t last = 0;
    double group = 0;
    bool first = true;
    for (auto &p : c) {
      if (first || last != p.stroke_id) {
        ++group;
        last = p.stroke_id;
        first = false;
      }
      v.insert(v.end(), {group, double(p.verb), p.red, p.green, p.blue, p.alpha,
                         p.x1, p.y1, p.x2, p.y2, p.x3, p.y3});
    }
    auto a = e->NewDoubleArray(v.size());
    if (!a)
      return nullptr;
    e->SetDoubleArrayRegion(a, 0, v.size(), v.data());
    return a;
  } catch (const std::bad_alloc &) {
    fail(e);
    return nullptr;
  }
}
JNI(export)
jbyteArray JNICALL
    Java_dev_nativesketch_Engine_export(JNIEnv *e, jobject, jlong h) {
  size_t n = 0;
  uint64_t r = 0;
  if (sketch_document_export(ptr(h), nullptr, 0, &n, &r)) {
    fail(e);
    return nullptr;
  }
  try {
    std::vector<uint8_t> b(n);
    if (sketch_document_export(ptr(h), b.data(), n, &n, &r)) {
      fail(e);
      return nullptr;
    }
    auto a = e->NewByteArray(n);
    if (!a)
      return nullptr;
    e->SetByteArrayRegion(a, 0, n, (jbyte *)b.data());
    return a;
  } catch (const std::bad_alloc &) {
    fail(e);
    return nullptr;
  }
}
JNI(revision)
jlong JNICALL
    Java_dev_nativesketch_Engine_revision(JNIEnv *e, jobject, jlong h) {
  size_t n = 0;
  uint64_t r = 0;
  if (sketch_document_export(ptr(h), nullptr, 0, &n, &r))
    fail(e);
  return r;
}
JNI(importBytes)
jint JNICALL Java_dev_nativesketch_Engine_importBytes(JNIEnv *e, jobject,
                                                      jlong h, jbyteArray a) {
  auto n = e->GetArrayLength(a);
  auto b = e->GetByteArrayElements(a, nullptr);
  if (!b)
    return 1;
  auto s = sketch_document_import(ptr(h), (uint8_t *)b, n);
  e->ReleaseByteArrayElements(a, b, JNI_ABORT);
  return s;
}
JNI(markSaved)
jint JNICALL Java_dev_nativesketch_Engine_markSaved(JNIEnv *, jobject, jlong h,
                                                    jlong r) {
  return sketch_document_mark_saved(ptr(h), r);
}
