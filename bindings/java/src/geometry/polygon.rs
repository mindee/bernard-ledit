#![allow(unsafe_code)]
#![allow(missing_docs)]
#![allow(non_snake_case)]

use bernard_ledit::geometry::point::Point;
use bernard_ledit::geometry::polygon::Polygon;
use jni::EnvUnowned;
use jni::objects::{JClass, JDoubleArray, JLongArray};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint, jlong, jlongArray, jstring};

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Polygon_equalsNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle_a: jlong,
    handle_b: jlong,
) -> jboolean {
    if handle_a == handle_b {
        return JNI_TRUE;
    }

    if handle_a == 0 || handle_b == 0 {
        return JNI_FALSE;
    }

    let poly_a = unsafe { &*(handle_a as *const Polygon) };
    let poly_b = unsafe { &*(handle_b as *const Polygon) };

    if poly_a == poly_b {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Polygon_newNative<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handles: JLongArray<'local>,
) -> jlong {
    let outcome = unowned_env.with_env(|env| -> jni::errors::Result<jlong> {
        let len = &handles.len(env).unwrap();

        let mut buf = vec![0; *len];
        handles.get_region(env, 0, &mut buf)?;

        let points_map = buf.into_iter().map(|handle| {
            let p = unsafe { &*(handle as *const Point) };
            *p
        });
        let polygon = Box::new(Polygon::new(points_map));
        Ok(Box::into_raw(polygon) as jlong)
    });

    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Polygon_newNativeFromCoords<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handles: JDoubleArray<'local>,
) -> jlong {
    let outcome = unowned_env.with_env(|env| -> jni::errors::Result<jlong> {
        let len = &handles.len(env).unwrap();

        let mut buf = vec![0.0; *len];
        handles.get_region(env, 0, &mut buf)?;

        let points: Vec<Point> = buf
            .chunks_exact(2)
            .map(|chunk| Point::new(chunk[0], chunk[1]))
            .collect();
        let polygon = Box::new(Polygon::new(points));
        Ok(Box::into_raw(polygon) as jlong)
    });
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Polygon_lengthNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jint {
    let p = unsafe { &*(handle as *mut Polygon) };
    jint::try_from(p.0.len()).expect("Polygon length exceeds maximum JNI integer capacity")
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Polygon_centroidNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jlong {
    let p = unsafe { &*(handle as *mut Polygon) };
    p.centroid()
        .map_or(0, |centroid| Box::into_raw(Box::new(centroid)) as jlong)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Polygon_getPointHandlesNative<
    'local,
>(
    mut env_unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jlongArray {
    let outcome = env_unowned.with_env(|env| -> jni::errors::Result<jlongArray> {
        let p = unsafe { &*(handle as *mut Polygon) };
        let handles =
            p.0.iter()
                .map(|point| Box::into_raw(Box::new(*point)) as jlong)
                .collect::<Vec<_>>();

        let jni_long_array = env.new_long_array(handles.len())?;
        jni_long_array.set_region(env, 0, &handles)?;
        Ok(jni_long_array.into_raw())
    });
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Polygon_toStringRepresentationNative<
    'local,
>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jstring {
    let outcome = unowned_env.with_env(|env| -> jni::errors::Result<jstring> {
        let p = unsafe { &*(handle as *mut Polygon) };
        let display_str = p.to_string_representation();
        let jni_str = env.new_string(display_str)?;
        Ok(jni_str.into_raw())
    });
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Polygon_getPointNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    index: jint,
) -> jlong {
    let p = unsafe { &*(handle as *mut Polygon) };
    let safe_index = usize::try_from(index).expect("Index from Java must be non-negative");
    let point = &p.0[safe_index];
    Box::into_raw(Box::new(*point)) as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Polygon_destroyNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    if handle != 0 {
        unsafe {
            let _ = Box::from_raw(handle as *mut Polygon);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Polygon_cloneNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jlong {
    if handle == 0 {
        return 0;
    }

    let ptr = handle as *const Polygon;
    let cloned_polygon = unsafe { (*ptr).clone() };
    Box::into_raw(Box::new(cloned_polygon)) as jlong
}
