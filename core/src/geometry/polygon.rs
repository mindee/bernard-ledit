//! Collection of points.

use crate::geometry::point::Point;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Collection of points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon(pub Vec<Point>);

impl Polygon {
    /// Create a new `Polygon` struct from `Point`s.
    pub fn new<I>(points: I) -> Self
    where
        I: IntoIterator<Item = Point>,
    {
        Self(points.into_iter().collect())
    }

    /// Weighted centroid of a polygon.
    /// Implemented from: <https://math.stackexchange.com/questions/90463/how-can-i-calculate-the-centroid-of-polygon>
    #[must_use]
    pub fn centroid(&self) -> Option<Point> {
        let pts = self.0.as_slice();
        match pts {
            [] => None,
            [p] => Some(*p),
            [p1, p2] => Some((*p1 + *p2) / 2.0),
            [p0, tail @ ..] => {
                let mut weighted_sum_x: f64 = 0.0;
                let mut weighted_sum_y: f64 = 0.0;
                let mut total_weight: f64 = 0.0;
                for window in tail.windows(2) {
                    let pi1 = window[0];
                    let pi2 = window[1];

                    let tri_centroid_x = (p0.x + pi1.x + pi2.x) / 3.0;
                    let tri_centroid_y = (p0.y + pi1.y + pi2.y) / 3.0;

                    // Clippy-suggested optimizations make this unbelievably ugly.
                    #[allow(clippy::suboptimal_flops, clippy::imprecise_flops)]
                    let weight = 0.5
                        * (p0.x * (pi1.y - pi2.y)
                            + pi1.x * (pi2.y - p0.y)
                            + pi2.x * (p0.y - pi1.y));

                    total_weight += weight;
                    weighted_sum_x += weight * tri_centroid_x;
                    weighted_sum_y += weight * tri_centroid_y;
                }
                if total_weight.abs() < f64::EPSILON {
                    // Unfortunately, we'll hit the 9 quadrillionth-precision inaccuracy here 😩
                    #[allow(clippy::cast_precision_loss)]
                    let count = pts.len() as f64;
                    let (sum_x, sum_y) = pts
                        .iter()
                        .fold((0.0, 0.0), |(acc_x, acc_y), p| (acc_x + p.x, acc_y + p.y));
                    return Some(Point::new(sum_x / count, sum_y / count));
                }
                Some(Point::new(
                    weighted_sum_x / total_weight,
                    weighted_sum_y / total_weight,
                ))
            }
        }
    }
    /// The default string representation, explicitly exposed for bindings.
    #[must_use]
    pub fn to_string_representation(&self) -> String {
        let points_str: Vec<String> = self
            .0
            .iter()
            .map(|p| format!("({},{})", p.x, p.y))
            .collect();

        format!("({})", points_str.join(", "))
    }
}
impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;

        let mut iter = self.0.iter();

        if let Some(first) = iter.next() {
            write!(f, "({},{})", first.x, first.y)?;

            for p in iter {
                write!(f, ", ({},{})", p.x, p.y)?;
            }
        }

        write!(f, ")")
    }
}

/// Implement conversion from tuple to `Point`.
impl From<Vec<(f64, f64)>> for Polygon {
    fn from(coords: Vec<(f64, f64)>) -> Self {
        let points = coords.into_iter().map(|(x, y)| Point::new(x, y)).collect();
        Self(points)
    }
}

/// A macro to create a `Polygon` struct using a collection of `Point`s.
#[macro_export]
macro_rules! polygon {
    ($($x:expr),*) => {
        Polygon(vec![$($x),*])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centroid_triangle() {
        let polygon = polygon!(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 1.0)
        );
        assert_eq!(
            polygon.centroid().unwrap(),
            Point::new(1.0 / 3.0, 1.0 / 3.0)
        );
    }

    #[test]
    fn test_centroid_square() {
        let polygon = polygon!(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0)
        );
        assert_eq!(polygon.centroid().unwrap(), Point::new(0.5, 0.5));
    }

    #[test]
    fn test_centroid_empty() {
        let polygon = polygon!();
        assert_eq!(polygon.centroid(), None);
    }

    #[test]
    fn test_centroid_single_point() {
        let single_point = Point::new(3.6, 6.2);
        let polygon = polygon!(single_point);
        assert_eq!(polygon.centroid().unwrap(), single_point);
    }

    #[test]
    fn test_centroid_degenerate() {
        let single_point = Point::new(3.6, 6.2);
        let polygon = polygon!(single_point, single_point);
        assert_eq!(polygon.centroid().unwrap(), single_point);
    }

    #[test]
    fn test_centroid_complex_polygon() {
        let polygon = polygon!(
            Point::new(1.2, 1.4),
            Point::new(8.4, 2.1),
            Point::new(5.3, 4.8),
            Point::new(9.1, 7.6),
            Point::new(2.6, 8.5),
            Point::new(1.2, 1.4)
        );
        assert_eq!(
            polygon.centroid().unwrap(),
            Point::new(455_557.0 / 101_370.0, 494_583.0 / 101_370.0)
        );
    }

    #[test]
    fn test_centroid_concave_l_shape() {
        let polygon = polygon!(
            Point::new(0.0, 0.0),
            Point::new(3.0, 0.0),
            Point::new(3.0, 1.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 3.0),
            Point::new(0.0, 3.0)
        );
        assert_eq!(polygon.centroid().unwrap(), Point::new(1.1, 1.1));
    }

    #[test]
    fn test_centroid_line_segment() {
        let polygon = polygon!(Point::new(0.0, 0.0), Point::new(4.0, 2.0));
        assert_eq!(polygon.centroid().unwrap(), Point::new(2.0, 1.0));
    }

    #[test]
    fn test_centroid_collinear_fallback() {
        let polygon = polygon!(
            Point::new(0.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(4.0, 4.0)
        );
        assert_eq!(polygon.centroid().unwrap(), Point::new(2.0, 2.0));
    }

    #[test]
    fn test_centroid_negative_coordinates() {
        let polygon = polygon!(
            Point::new(-2.0, -2.0),
            Point::new(-0.0, -2.0),
            Point::new(-0.0, -0.0),
            Point::new(-2.0, -0.0)
        );
        assert_eq!(polygon.centroid().unwrap(), Point::new(-1.0, -1.0));
    }

    #[test]
    fn test_polygon_new_from_iterator() {
        let vec_pts = vec![Point::new(1.0, 2.0), Point::new(3.0, 4.0)];
        let polygon = Polygon::new(vec_pts);
        assert_eq!(polygon.0.len(), 2);
    }
    #[test]
    fn test_from_vec_coords() {
        let coords = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let polygon = Polygon::from(coords);

        assert_eq!(polygon.0.len(), 4);
        assert_eq!(polygon.0[0], Point::new(0.0, 0.0));
        assert_eq!(polygon.0[1], Point::new(10.0, 0.0));
        assert_eq!(polygon.0[2], Point::new(10.0, 10.0));
        assert_eq!(polygon.0[3], Point::new(0.0, 10.0));
    }
}
