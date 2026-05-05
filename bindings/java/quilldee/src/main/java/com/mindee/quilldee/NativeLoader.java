package com.mindee.quilldee;

public class NativeLoader {
    private static boolean isLoaded = false;

    public static synchronized void load() {
        if (!isLoaded) {
            System.loadLibrary("quilldee_java");
            isLoaded = true;
        }
    }
}