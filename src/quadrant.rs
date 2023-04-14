use super::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quadrant {
    BottomLeft = 0,
    BottomRight = 1,
    TopRight = 2,
    TopLeft = 3,
}

impl Quadrant {
    #[inline]
    pub fn for_point<P>(point: P, center: i32) -> Quadrant
    where
        P: Into<Point>,
    {
        let point = point.into();
        if point.x() < center {
            if point.y() >= center {
                Quadrant::TopLeft
            } else {
                Quadrant::BottomLeft
            }
        } else if point.y() >= center {
            Quadrant::TopRight
        } else {
            Quadrant::BottomRight
        }
    }
}

#[cfg(test)]
mod test {
    use super::Quadrant;

    #[test]
    fn test_for_point() {
        assert_eq!(Quadrant::for_point((0, 0), 1), Quadrant::BottomLeft);
        assert_eq!(Quadrant::for_point((1, 0), 1), Quadrant::BottomRight);
        assert_eq!(Quadrant::for_point((0, 1), 1), Quadrant::TopLeft);
        assert_eq!(Quadrant::for_point((1, 1), 1), Quadrant::TopRight);
    }
}
