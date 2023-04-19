use super::{IRect, Region};
use glam::IVec2;
use num_traits::{NumCast, Unsigned};
use std::ops::{Index, IndexMut, Range};

#[derive(Debug, Clone)]
pub struct IPolygon {
    points: Vec<IVec2>,
}

impl IPolygon {
    #[inline]
    pub fn new(points: Vec<IVec2>) -> Self {
        Self { points }
    }

    #[inline]
    pub fn points(&self) -> &[IVec2] {
        &self.points
    }

    pub fn is_clockwise(&self) -> bool {
        let mut sum = 0;
        for i in 0..self.points.len() {
            let p1 = self.points[i];
            let p2 = self.points[(i + 1) % self.points.len()];
            sum += (p2.x - p1.x) * (p2.y + p1.y);
        }
        sum > 0
    }
}

impl From<IRect> for IPolygon {
    #[inline]
    fn from(rect: IRect) -> Self {
        IPolygon::from(&rect)
    }
}

impl From<&IRect> for IPolygon {
    #[inline]
    fn from(rect: &IRect) -> Self {
        Self {
            points: vec![
                rect.min(),
                IVec2::new(rect.max().x, rect.min().y),
                rect.max(),
                IVec2::new(rect.min().x, rect.max().y),
            ],
        }
    }
}

impl<U: Unsigned + NumCast + Copy> From<Region<U>> for IPolygon {
    #[inline]
    fn from(region: Region<U>) -> Self {
        IPolygon::from(&region)
    }
}

impl<U: Unsigned + NumCast + Copy> From<&Region<U>> for IPolygon {
    #[inline]
    fn from(region: &Region<U>) -> Self {
        Self {
            points: vec![
                region.point(),
                IVec2::new(region.end_point().x, region.point().y),
                region.end_point(),
                IVec2::new(region.point().x, region.end_point().y),
            ],
        }
    }
}

impl Index<usize> for IPolygon {
    type Output = IVec2;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.points[index]
    }
}

impl Index<Range<usize>> for IPolygon {
    type Output = [IVec2];

    #[inline]
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.points[index]
    }
}

impl IndexMut<usize> for IPolygon {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.points[index]
    }
}
