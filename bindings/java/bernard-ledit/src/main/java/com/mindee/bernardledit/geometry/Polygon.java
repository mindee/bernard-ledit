package com.mindee.bernardledit.geometry;

import java.lang.ref.Cleaner;
import java.util.ArrayList;
import java.util.List;

public class Polygon implements AutoCloseable, Cloneable {
    private static final Cleaner CLEANER = Cleaner.create();
    private final Cleaner.Cleanable cleanable;
    static {
        System.loadLibrary("bernard_ledit_java");
    }
    private long nativeHandle;

    public Polygon(Iterable<Point> points) {
        List<Long> handleList = new ArrayList<>();
        for (Point p : points) {
            handleList.add(p.getNativeHandle());
        }
        long[] handlesArray = new long[handleList.size()];
        for (int i = 0; i < handleList.size(); i++) {
            handlesArray[i] = handleList.get(i);
        }
        this.nativeHandle = newNative(handlesArray);
        final long handleToClean = this.nativeHandle;
        this.cleanable = CLEANER.register(this, () -> destroyNative(handleToClean));
    }

    /**
     * Internal constructor used when Rust returns a new Polygon handle.
     */
    private Polygon(long nativeHandle) {
        if (nativeHandle == 0) {
            throw new IllegalStateException("Received null pointer from native code.");
        }
        this.nativeHandle = nativeHandle;

        final long handleToClean = this.nativeHandle;
        this.cleanable = CLEANER.register(this, () -> Polygon.destroyNative(handleToClean));
    }

    @Override
    public Polygon clone() {
        ensureValid();
        long clonedHandle = cloneNative(this.nativeHandle);
        return new Polygon(clonedHandle);
    }

    public Polygon(double[] coords) {
        if (coords.length % 2 != 0) {
            throw new IllegalArgumentException("Coordinate array length must be even (x, y pairs).");
        }
        this.nativeHandle = newNativeFromCoords(coords);
        final long handleToClean = this.nativeHandle;
        this.cleanable = CLEANER.register(this, () -> destroyNative(handleToClean));
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null || getClass() != obj.getClass()) return false;
        Polygon other = (Polygon) obj;
        ensureValid();
        other.ensureValid();
        return equalsNative(this.nativeHandle, other.nativeHandle);
    }

    private void ensureValid() {
        if (this.nativeHandle == 0) {
            throw new IllegalStateException("Native handle is closed/freed.");
        }
    }

    /**
     * Exposes a single point from Rust `pub points: Vec<Point>`
     * @param index Index of the point to retrieve.
     * @return Point at the given index.
     */
    public Point getPoint(int index) {
        ensureValid();
        return new Point(getPointNative(this.nativeHandle, index));
    }

    /**
     * Exposes Rust `pub points: Vec<Point>`
     * @return A List of points in the polygon.
     */
    public List<Point> getPoints() {
        ensureValid();
        long[] handles = getPointHandlesNative(this.nativeHandle);
        List<Point> points = new ArrayList<>(handles.length);
        for (long h : handles) {
            points.add(new Point(h));
        }
        return points;
    }
    /**
     * Exposes Rust `len()`
     * @return Number of points in the polygon.
     */
    public int length() {
        ensureValid();
        return lengthNative(this.nativeHandle);
    }

    /**
     * Calculates the weighted centroid of the polygon.
     * @return The centroid Point, or null if the polygon has no points.
     */
    public Point centroid() {
        ensureValid();
        long centroidHandle = centroidNative(this.nativeHandle);
        if (centroidHandle == 0) {
            return null;
        }
        return new Point(centroidHandle);
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
    public void close() {
        if (this.nativeHandle != 0) {
            this.cleanable.clean();
            this.nativeHandle = 0;
        }
    }

    private static native void destroyNative(long handle);

    private native long newNativeFromCoords(double[] coords);
    private native long newNative(long[] handlesArray);
    private native long getPointNative(long handle, int index);
    private native long[] getPointHandlesNative(long handle);
    private native int lengthNative(long handle);
    private native long centroidNative(long handle);
    private native String toStringRepresentationNative(long handle);
    private native boolean equalsNative(long handle, long otherHandle);
    private native long cloneNative(long handle);
}
