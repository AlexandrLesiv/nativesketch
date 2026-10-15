use crate::{boundary::*, dto};
use sketch_core::application::{EditorQuery, QueryResult};
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_selected_bounds(
    pointer: *mut Runtime,
    handle: *mut Session,
    output: *mut dto::Bounds,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            let session = session(handle, pointer)?;
            let slot = status(value.core.session(session.window))?;
            if slot.app.projection().editor.document.selected == 0 {
                return Err(14);
            }
            let QueryResult::SelectedBounds(bounds) = slot
                .app
                .query(EditorQuery::SelectedBounds)
                .map_err(|_| 2u32)?
            else {
                return Err(2);
            };
            output.write(dto::Bounds {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
            });
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_session_v1_scene(
    pointer: *mut Runtime,
    handle: *mut Session,
    output: *mut *mut Scene,
) -> u32 {
    unsafe {
        runtime(pointer, false, |value| {
            require_output(output)?;
            output.write(std::ptr::null_mut());
            let session = session(handle, pointer)?;
            if session.scene_active.get() {
                return Err(9);
            }
            let QueryResult::Scene(scene) = status(value.core.session(session.window))?
                .app
                .query(EditorQuery::Scene)
                .map_err(|_| 2u32)?
            else {
                return Err(2);
            };
            session.scene_active.set(true);
            output.write(Box::into_raw(Box::new(Scene {
                owner: value.owner,
                active: session.scene_active.clone(),
                value: scene,
            })));
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_scene_v1_info(
    pointer: *mut Scene,
    output: *mut dto::SceneInfo,
) -> u32 {
    unsafe {
        entry(|| {
            require_output(output)?;
            if pointer.is_null() {
                return Err(1);
            }
            let scene = &*pointer;
            if scene.owner != std::thread::current().id() {
                return Err(11);
            }
            output.write(dto::SceneInfo {
                session: scene.value.key.session.get(),
                revision: scene.value.key.revision.0,
                generation: scene.value.key.generation.get(),
                command_count: scene.value.commands.len() as u64,
            });
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_scene_v1_copy(
    pointer: *mut Scene,
    output: *mut dto::Path,
    capacity: u32,
    count: *mut u32,
) -> u32 {
    unsafe {
        entry(|| {
            require_output(count)?;
            if pointer.is_null() {
                return Err(1);
            }
            let scene = &*pointer;
            if scene.owner != std::thread::current().id() {
                return Err(11);
            }
            let needed = u32::try_from(scene.value.commands.len()).map_err(|_| 3u32)?;
            count.write(needed);
            if capacity < needed {
                return Err(3);
            }
            if needed > 0 {
                require_output(output)?;
            }
            for (index, path) in scene.value.commands.iter().enumerate() {
                output.add(index).write(dto::Path {
                    stroke_id: path.stroke_id,
                    verb: path.verb,
                    reserved: 0,
                    red: path.rgba[0],
                    green: path.rgba[1],
                    blue: path.rgba[2],
                    alpha: path.rgba[3],
                    x1: path.points[0],
                    y1: path.points[1],
                    x2: path.points[2],
                    y2: path.points[3],
                    x3: path.points[4],
                    y3: path.points[5],
                });
            }
            Ok(())
        })
    }
}
/// # Safety
/// Valid aligned buffers, unique handles and serialized owner-thread use are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sketch_scene_v1_free(pointer: *mut *mut Scene) -> u32 {
    unsafe {
        entry(|| {
            let handle = read(pointer)?;
            if handle.is_null() {
                return Ok(());
            }
            if (*handle).owner != std::thread::current().id() {
                return Err(11);
            }
            let scene = Box::from_raw(handle);
            scene.active.set(false);
            drop(scene);
            pointer.write(std::ptr::null_mut());
            Ok(())
        })
    }
}
