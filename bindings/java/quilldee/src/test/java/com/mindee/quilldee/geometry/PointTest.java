package com.mindee.quilldee.geometry;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;

public class PointTest {

    @Test
    public void testPointLifecycle() {
        assertDoesNotThrow(() -> {
            try (Point point = new Point(3.14, 2.71)) {
                assertEquals(3.14, point.getX(), 0.0001, "X coordinate mismatch");
                assertEquals(2.71, point.getY(), 0.0001, "Y coordinate mismatch");

            }
        });
    }
}
