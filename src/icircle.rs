use super::IRect;
use glam::IVec2;

/// A circle represented by a center point, in integer coordinates, and a radius.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ICircle {
    point: IVec2,
    radius: u32,
}

impl ICircle {
    #[inline]
    pub fn new<P>(point: P, radius: u32) -> Self
    where
        P: Into<IVec2>,
    {
        Self {
            point: point.into(),
            radius,
        }
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.point.x
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.point.y
    }

    #[inline]
    pub fn point(&self) -> IVec2 {
        self.point
    }

    #[inline]
    pub fn radius(&self) -> u32 {
        self.radius
    }

    #[inline]
    pub fn contains<P>(&self, point: P) -> bool
    where
        P: Into<IVec2>,
    {
        let d = point.into() - self.point;
        d.x * d.x + d.y * d.y <= self.radius as i32 * self.radius as i32
    }

    #[inline]
    pub fn aabb(&self) -> IRect {
        let size = self.radius * 2;
        IRect::centered_at(self.point, size, size)
    }

    #[inline]
    pub fn inner_aabb(&self) -> IRect {
        let size = (self.radius as f32 * 2f32.sqrt()) as u32;
        IRect::centered_at(self.point, size, size)
    }
}

impl From<IRect> for ICircle {
    fn from(rect: IRect) -> Self {
        ICircle::from(&rect)
    }
}
impl From<&IRect> for ICircle {
    fn from(rect: &IRect) -> Self {
        let radius = rect.width().min(rect.height()) / 2;
        let p = (
            rect.x() + rect.width() as i32 / 2,
            rect.y() + rect.height() as i32 / 2,
        );
        Self::new(p, radius)
    }
}

impl IntoIterator for ICircle {
    type Item = (i32, i32);
    type IntoIter = CirclePixelIterator;

    fn into_iter(self) -> Self::IntoIter {
        CirclePixelIterator::new(self)
    }
}

pub struct CirclePixelIterator {
    circle: ICircle,
    x: i32,
    y: i32,
}

impl CirclePixelIterator {
    pub fn new(circle: ICircle) -> Self {
        let y = -(circle.radius as i32);
        let r = circle.radius as i32;
        let x = -((r * r - y * y) as f32).sqrt() as i32;
        Self { circle, x, y }
    }
}

impl Iterator for CirclePixelIterator {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.circle.radius as i32;
        if self.y > r {
            return None;
        }

        let x = self.x;
        self.x += 1;

        let x_len = ((r * r - self.y * self.y) as f32).sqrt() as i32;
        if x > x_len {
            self.y += 1;
            let x_len = ((r * r - self.y * self.y) as f32).sqrt() as i32;
            self.x = -x_len;
            self.next()
        } else {
            let x = self.circle.x() + x;
            let y = self.circle.y() + self.y;
            Some((x, y))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_rect() {
        let rect = IRect::new(0, 0, 10, 10);
        let circle = ICircle::from(rect);
        assert_eq!(circle.x(), 5);
        assert_eq!(circle.y(), 5);
        assert_eq!(circle.radius(), 5);
    }

    #[test]
    fn test_circle_iterator() {
        let mut iter = ICircle::new((0, 0), 2).into_iter();
        assert_eq!(iter.next(), Some((0, -2)));
        assert_eq!(iter.next(), Some((-1, -1)));
        assert_eq!(iter.next(), Some((0, -1)));
        assert_eq!(iter.next(), Some((1, -1)));
        assert_eq!(iter.next(), Some((-2, 0)));
        assert_eq!(iter.next(), Some((-1, 0)));
        assert_eq!(iter.next(), Some((0, 0)));
        assert_eq!(iter.next(), Some((1, 0)));
        assert_eq!(iter.next(), Some((2, 0)));
        assert_eq!(iter.next(), Some((-1, 1)));
        assert_eq!(iter.next(), Some((0, 1)));
        assert_eq!(iter.next(), Some((1, 1)));
        assert_eq!(iter.next(), Some((0, 2)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}
