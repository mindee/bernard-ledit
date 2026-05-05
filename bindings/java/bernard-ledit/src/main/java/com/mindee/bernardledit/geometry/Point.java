package com.mindee.bernardledit.geometry;

import java.lang.ref.Cleaner;

public class Point implements AutoCloseable, Cloneable {
    private static final Cleaner CLEANER = Cleaner.create();
    private final Cleaner.Cleanable cleanable;
    static {
        System.loadLibrary("bernard_ledit_java");
    }
    private long nativeHandle;

    /**
     * Native handle getter. Mostly for `Polygon` use.
     * @return Native handle of the Point.
     */
    public long getNativeHandle() {
        return nativeHandle;
    }
    public Point(double x, double y) {
        this.nativeHandle = newNative(x, y);
        final long handleToClean = this.nativeHandle;
        this.cleanable = CLEANER.register(this, () -> destroyNative(handleToClean));
    }


    /**
     * Internal constructor used when Rust returns a new Point handle
     */
    public Point(long nativeHandle) {
        if (nativeHandle == 0) {
            throw new IllegalStateException("Received null pointer from native code.");
        }
        this.nativeHandle = nativeHandle;
        final long handleToClean = this.nativeHandle;
        this.cleanable = CLEANER.register(this, () -> destroyNative(handleToClean));
    }

    /**
     * Maps to Rust: `impl fmt::Display for Point`
     */
    @Override
    public String toString() {
        ensureValid();
        return toStringRepresentationNative(this.nativeHandle);
    }


    @Override
    public Point clone() {
        ensureValid();
        long clonedHandle = cloneNative(this.nativeHandle);
        return new Point(clonedHandle);
    }

    /**
     * Maps to Rust: `impl Add for Point`
     */
    public Point add(Point addend) {
        ensureValid();
        addend.ensureValid();
        return new Point(addNative(this.nativeHandle, addend.nativeHandle));
    }

    /**
     * Maps to Rust: `impl Sub for Point`
     */
    public Point sub(Point subtrahend) {
        ensureValid();
        subtrahend.ensureValid();
        return new Point(subNative(this.nativeHandle, subtrahend.nativeHandle));
    }

    /**
     * Maps to Rust: `impl Mul<f64> for Point`
     */
    public Point mul(double factor) {
        ensureValid();
        return new Point(mulNative(this.nativeHandle, factor));
    }

    /**
     * Maps to Rust: `impl Div<f64> for Point`
     */
    public Point div(double denominator) {
        ensureValid();
        return new Point(divNative(this.nativeHandle, denominator));
    }

    /**
     * Maps to Rust: `impl Drop for Point`
     * @param index Index of the coordinate to retrieve (0 for x, 1 for y).
     * @return The coordinate value.
     */
    public double get(int index) {
        ensureValid();
        if (index < 0 || index > 1) {
            throw new IndexOutOfBoundsException("Rust Point only supports index 0 (x) or 1 (y)");
        }
        return indexNative(this.nativeHandle, index);
    }
    /**
     * Maps to Rust: `impl From<(f64, f64)> for Point`
     */
    public static Point fromTuple(double[] tuple) {
        if (tuple == null || tuple.length != 2) {
            throw new IllegalArgumentException("Tuple array must have exactly 2 elements (x, y)");
        }
        // Internally routes to a native method or just reuses the standard constructor mapping
        return new Point(fromTupleNative(tuple));
    }

    /**
     * Maps to Rust: `impl From<Point> for (f64, f64)`
     */
    public double[] toTuple() {
        ensureValid();
        return toTupleNative(this.nativeHandle);
    }

    /**
     * Exposes Rust `pub x: f64`
     */
    public double getX() {
        ensureValid();
        return getXNative(this.nativeHandle);
    }

    /**
     * Exposes Rust `pub y: f64`
     */
    public double getY() {
        ensureValid();
        return getYNative(this.nativeHandle);
    }

    private void ensureValid() {
        if (this.nativeHandle == 0) {
            throw new IllegalStateException("Native handle is closed/freed.");
        }
    }

    @Override
    public void close() {
        if (this.nativeHandle != 0) {
            this.cleanable.clean();
            this.nativeHandle = 0;
        }
    }

    private static native void destroyNative(long handle);

    private native long newNative(double x, double y);
    private native long addNative(long selfHandle, long addendHandle);
    private native long subNative(long selfHandle, long subtrahendHandle);
    private native long mulNative(long selfHandle, double factor);
    private native long divNative(long selfHandle, double denominator);
    private native String toStringRepresentationNative(long handle);
    private native double indexNative(long handle, int index);
    private static native long fromTupleNative(double[] tuple);
    private native double[] toTupleNative(long handle);
    private native double getXNative(long handle);
    private native double getYNative(long handle);
    private native long cloneNative(long handle);
}
