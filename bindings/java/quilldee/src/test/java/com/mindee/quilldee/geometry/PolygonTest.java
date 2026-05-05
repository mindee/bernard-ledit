package com.mindee.quilldee.geometry;

import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

public class PolygonTest {

    @Test
    public void testPolygonFromCoordinatesArray() {
        double[] coords = {0.0, 0.0, 4.0, 0.0, 4.0, 3.0};

        try (Polygon polygon = new Polygon(coords)) {
            assertEquals(3, polygon.length(), "Polygon should have 3 points.");
        }
    }

    @Test
    public void testPolygonFromPointsIterable() {
        try (Point p1 = new Point(0.0, 0.0);
             Point p2 = new Point(4.0, 0.0);
             Point p3 = new Point(4.0, 3.0);
             Polygon polygon = new Polygon(Arrays.asList(p1, p2, p3))) {

            assertEquals(3, polygon.length(), "Polygon should have 3 points.");
        }
    }

    @Test
    @SuppressWarnings("resource")
    public void testInvalidCoordinateArrayLengthThrowsException() {
        double[] invalidCoords = {0.0, 0.0, 4.0, 0.0, 4.0};

        IllegalArgumentException exception = assertThrows(
                IllegalArgumentException.class,
                () -> new Polygon(invalidCoords)
        );

        assertEquals("Coordinate array length must be even (x, y pairs).", exception.getMessage());
    }

    @Test
    public void testGetPoint() {
        double[] coords = {1.5, 2.5, 3.5, 4.5};

        try (Polygon polygon = new Polygon(coords)) {
            try (Point p0 = polygon.getPoint(0);
                 Point p1 = polygon.getPoint(1)) {

                assertEquals(1.5, p0.getX(), 0.0001);
                assertEquals(2.5, p0.getY(), 0.0001);

                assertEquals(3.5, p1.getX(), 0.0001);
                assertEquals(4.5, p1.getY(), 0.0001);
            }
        }
    }

    @Test
    public void testGetPoints() {
        double[] coords = {0.0, 0.0, 10.0, 0.0, 5.0, 10.0};

        try (Polygon polygon = new Polygon(coords)) {
            List<Point> points = polygon.getPoints();

            assertEquals(3, points.size());
            assertEquals(5.0, points.get(2).getX(), 0.0001);
            assertEquals(10.0, points.get(2).getY(), 0.0001);
        }
    }

    @Test
    public void testCentroid() {
        double[] coords = {0.0, 0.0, 4.0, 0.0, 4.0, 4.0, 0.0, 4.0};

        try (Polygon polygon = new Polygon(coords);
             Point centroid = polygon.centroid()) {

            assertNotNull(centroid, "Centroid should not be null for a valid polygon.");
            assertEquals(2.0, centroid.getX(), 0.0001, "Centroid X should be 2.0");
            assertEquals(2.0, centroid.getY(), 0.0001, "Centroid Y should be 2.0");
        }
    }

    @Test
    public void testEqualsAndClone() {
        double[] coords1 = {0.0, 0.0, 2.0, 2.0};
        double[] coords2 = {0.0, 0.0, 2.0, 2.0};
        double[] coordsDifferent = {1.0, 1.0, 3.0, 3.0};

        try (Polygon poly1 = new Polygon(coords1);
             Polygon poly2 = new Polygon(coords2);
             Polygon polyDiff = new Polygon(coordsDifferent);
             Polygon clonedPoly = poly1.clone()) {

            assertEquals(poly1, poly2, "Polygons with identical points should be equal.");
            assertNotEquals(poly1, polyDiff, "Polygons with different points should not be equal.");

            assertEquals(poly1, clonedPoly, "Cloned polygon should equal the original.");
            assertNotSame(poly1, clonedPoly, "Cloned polygon should be a different instance.");
        }
    }

    @Test
    public void testToString() {
        double[] coords = {0.0, 0.0, 1.0, 1.0};
        try (Polygon polygon = new Polygon(coords)) {
            String str = polygon.toString();
            assertNotNull(str, "toString representation should not be null");
            assertFalse(str.isEmpty(), "toString representation should not be empty");
        }
    }
}