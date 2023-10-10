use bevy_math::{IVec2, UVec2};

/// Decorate an `IVec2` iterator to only yield positive points,
/// and to covert them to `UVec2`.
pub struct UnsignedPixelIterator<I>
where
    I: Iterator<Item = IVec2>,
{
    inner: I,
}

impl<I> UnsignedPixelIterator<I>
where
    I: Iterator<Item = IVec2>,
{
    #[inline]
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I> Iterator for UnsignedPixelIterator<I>
where
    I: Iterator<Item = IVec2>,
{
    type Item = UVec2;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let i = self.inner.next()?;
            if i.x < 0 || i.y < 0 {
                continue;
            } else {
                return Some(i.as_uvec2());
            }
        }
    }
}
