package com.mindee.bernardledit;

public class NativeLoader {
    private static boolean isLoaded = false;

    public static synchronized void load() {
        if (!isLoaded) {
            System.loadLibrary("bernard_ledit_java");
            isLoaded = true;
        }
    }
}