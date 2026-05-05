import pytest

from quilldee.geometry import Point, Polygon


def assert_point(point, x, y):
    assert isinstance(point, Point)
    assert point.x == pytest.approx(x)
    assert point.y == pytest.approx(y)


def test_point_coordinates_are_readable_mutable_and_indexable():
    point = Point(1.5, 2.5)

    assert_point(point, 1.5, 2.5)
    assert point[0] == pytest.approx(1.5)
    assert point[1] == pytest.approx(2.5)

    point.x = 3.5
    point.y = 4.5

    assert_point(point, 3.5, 4.5)
    assert str(point) == "(3.5, 4.5)"


def test_polygon_accepts_points_and_coordinate_tuples():
    polygon = Polygon([Point(0.0, 0.0), (2.0, 0.0), (0.0, 2.0)])

    assert len(polygon) == 3
    assert polygon[0] == Point(0.0, 0.0)
    assert polygon[1] == Point(2.0, 0.0)
    assert polygon[-1] == Point(0.0, 2.0)


def test_polygon_indexing_raises_index_error_for_empty_or_out_of_range_access():
    empty = Polygon([])
    polygon = Polygon([(0.0, 0.0)])

    with pytest.raises(IndexError):
        _ = empty[0]

    with pytest.raises(IndexError):
        _ = empty[-1]

    with pytest.raises(IndexError):
        _ = polygon[1]


def test_polygon_slicing_returns_polygon():
    polygon = Polygon([
        (0.0, 0.0),
        (1.0, 1.0),
        (2.0, 2.0),
        (3.0, 3.0),
        (4.0, 4.0),
    ])

    every_other = polygon[::2]
    assert isinstance(every_other, Polygon)
    assert len(every_other) == 3
    assert every_other[0] == Point(0.0, 0.0)
    assert every_other[1] == Point(2.0, 2.0)
    assert every_other[2] == Point(4.0, 4.0)

    reversed_polygon = polygon[::-1]
    assert isinstance(reversed_polygon, Polygon)
    assert len(reversed_polygon) == 5
    assert reversed_polygon[0] == Point(4.0, 4.0)
    assert reversed_polygon[-1] == Point(0.0, 0.0)

    empty_slice = polygon[10:]
    assert isinstance(empty_slice, Polygon)
    assert len(empty_slice) == 0


def test_polygon_slice_step_zero_raises_value_error():
    polygon = Polygon([(0.0, 0.0), (1.0, 1.0)])

    with pytest.raises(ValueError):
        _ = polygon[::0]


def test_polygon_centroid():
    polygon = Polygon([(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)])
    centroid = polygon.centroid()

    assert_point(centroid, 1.0 / 3.0, 1.0 / 3.0)
    assert Polygon([]).centroid() is None
