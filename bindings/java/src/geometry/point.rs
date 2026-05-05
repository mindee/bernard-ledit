#![allow(unsafe_code)]
#![allow(missing_docs)]
#![allow(non_snake_case)]

use bernard_ledit::geometry::point::Point;
use jni::EnvUnowned;
use jni::objects::{JClass, JDoubleArray};
use jni::sys::{jdouble, jdoubleArray, jint, jlong, jstring};

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_newNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    x: jdouble,
    y: jdouble,
) -> jlong {
    let point = Box::new(Point::new(x, y));
    Box::into_raw(point) as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_destroyNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    if handle != 0 {
        unsafe {
            let _ = Box::from_raw(handle as *mut Point);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_addNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    self_handle: jlong,
    addend_handle: jlong,
) -> jlong {
    let p1 = unsafe { &*(self_handle as *mut Point) };
    let p2 = unsafe { &*(addend_handle as *mut Point) };
    let result = Box::new(*p1 + *p2);
    Box::into_raw(result) as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_subNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    self_handle: jlong,
    addend_handle: jlong,
) -> jlong {
    let p1 = unsafe { &*(self_handle as *mut Point) };
    let p2 = unsafe { &*(addend_handle as *mut Point) };
    let result = Box::new(*p1 - *p2);
    Box::into_raw(result) as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_mulNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    self_handle: jlong,
    factor: jdouble,
) -> jlong {
    let p1 = unsafe { &*(self_handle as *mut Point) };
    let result = Box::new(*p1 * factor);
    Box::into_raw(result) as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_divNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    self_handle: jlong,
    denominator: jdouble,
) -> jlong {
    let p1 = unsafe { &*(self_handle as *mut Point) };
    let result = Box::new(*p1 / denominator);
    Box::into_raw(result) as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_getXNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jdouble {
    let p = unsafe { &*(handle as *mut Point) };
    p.x
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_getYNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jdouble {
    let p = unsafe { &*(handle as *mut Point) };
    p.y
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_indexNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    index: jint,
) -> jdouble {
    let p = unsafe { &*(handle as *mut Point) };
    let safe_index = usize::try_from(index).expect("Index from Java must be non-negative");
    p[safe_index]
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_toStringRepresentationNative<
    'local,
>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jstring {
    let outcome = unowned_env.with_env(|env| -> jni::errors::Result<jstring> {
        let p = unsafe { &*(handle as *mut Point) };
        let display_str = p.to_string_representation();
        let jni_str = env.new_string(display_str)?;
        Ok(jni_str.into_raw())
    });
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_toTupleNative<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jdoubleArray {
    let outcome = unowned_env.with_env(|env| -> jni::errors::Result<jdoubleArray> {
        let p = unsafe { &*(handle as *mut Point) };
        let array = env
            .new_double_array(2)
            .expect("Failed to create Java array");
        let buf = [p.x, p.y];
        array.set_region(env, 0, &buf)?;
        Ok(array.into_raw())
    });
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_fromTupleNative<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    tuple: JDoubleArray<'local>,
) -> jlong {
    let outcome = unowned_env.with_env(|env| -> jni::errors::Result<jlong> {
        let mut buf = [0.0; 2];
        tuple.get_region(env, 0, &mut buf)?;

        let point = Box::new(Point::from(<(f64, f64)>::from(buf)));
        Ok(Box::into_raw(point) as jlong)
    });
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_mindee_bernardledit_geometry_Point_cloneNative<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jlong {
    if handle == 0 {
        return 0;
    }

    let p = handle as *const Point;
    let cloned_point = unsafe { *p };
    Box::into_raw(Box::new(cloned_point)) as jlong
}
