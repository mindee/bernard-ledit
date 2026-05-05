package com.mindee.bernardledit;

public class NativeLoader {
  private static boolean isLoaded;

  public static synchronized void load() {
    if (!isLoaded) {
      System.loadLibrary("bernard_ledit_java");
      isLoaded = true;
    }
  }
}
