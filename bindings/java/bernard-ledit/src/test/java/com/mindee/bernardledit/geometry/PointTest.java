package com.mindee.bernardledit.geometry;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;

public class PointTest {

  @Test
  public void testPointLifecycle() {
    assertDoesNotThrow(
        () -> {
          try (Point point = new Point(3.14, 2.71)) {
            assertEquals(3.14, point.getX(), 0.0001, "X coordinate mismatch");
            assertEquals(2.71, point.getY(), 0.0001, "Y coordinate mismatch");
          }
        });
  }
}
