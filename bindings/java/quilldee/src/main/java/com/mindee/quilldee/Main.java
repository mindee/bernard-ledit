package com.mindee.quilldee;

import com.mindee.quilldee.geometry.Point;

public class Main {
    public static void main(String[] args) {
        System.out.println("Testing Rust JNI Bindings...");

        try (Point p1 = new Point(10.5, 20.0);
             Point p2 = new Point(5.0, 5.0)) {

            System.out.println("Point 1: " + p1.toString());
            System.out.println("Point 2: " + p2.toString());

            try (Point p3 = p1.add(p2)) {
                System.out.println("P1 + P2 = " + p3.toString());
                System.out.println("P3 X coordinate: " + p3.getX());
            }

        } catch (Exception e) {
            System.err.println("Error interacting with native code: " + e.getMessage());
        }
    }
}
