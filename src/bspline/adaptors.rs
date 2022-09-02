use crate::builder::TooFewElements;
use crate::{DiscreteGenerator, Generator, SortedGenerator};

/// DiscreteGenerator Adaptor which repeats its first and last element `n` more times.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct BorderBuffer<G> {
    inner: G,
    n: usize,
}

impl<G> BorderBuffer<G>
where
    G: DiscreteGenerator,
{
    /// Creates a generator which repeats the first and last element of the given generator `n` more times.
    pub fn new(inner: G, n: usize) -> Self {
        BorderBuffer { inner, n }
    }
    /// Maps index from outer to inner values
    fn map_into(&self, index: usize) -> usize {
        if index < self.n {
            return 0;
        }
        if index - self.n >= self.inner.len() {
            return self.inner.len();
        }
        index - self.n
    }
    /// Maps index from inner to outer values
    fn map_from(&self, index: usize) -> usize {
        if index == self.inner.len() {
            return self.len();
        }
        if index == 0 {
            return 0;
        }
        index + self.n
    }
}

impl<G> Generator<usize> for BorderBuffer<G>
where
    G: DiscreteGenerator,
{
    type Output = G::Output;
    fn gen(&self, input: usize) -> Self::Output {
        let clamped = input.max(self.n).min(self.inner.len() + self.n - 1);
        self.inner.gen(clamped - self.n)
    }
}

impl<G> DiscreteGenerator for BorderBuffer<G>
where
    G: DiscreteGenerator,
{
    fn len(&self) -> usize {
        self.inner.len() + 2 * self.n
    }
}

impl<G> SortedGenerator for BorderBuffer<G>
where
    G: SortedGenerator,
{
    fn strict_upper_bound_clamped(&self, element: Self::Output, min: usize, max: usize) -> usize
    where
        Self::Output: PartialOrd + Copy,
    {
        let inner_min = self.map_into(min);
        debug_assert!(max <= self.len());
        let inner_max = self.map_into(max);
        let inner_index = self
            .inner
            .strict_upper_bound_clamped(element, inner_min, inner_max);
        self.map_from(inner_index)
    }
    fn strict_upper_bound(&self, element: Self::Output) -> usize
    where
        Self::Output: PartialOrd + Copy,
    {
        let inner_index = self.inner.strict_upper_bound(element);
        self.map_from(inner_index)
    }
}

/// DiscreteGenerator Adaptor which deletes the first and last element.
///
/// # Panics
///
/// Using this Generator may cause a panic if the underlying generator has less than two elements.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct BorderDeletion<G> {
    inner: G,
}

impl<G> BorderDeletion<G>
where
    G: DiscreteGenerator,
{
    /// Creates a generator ignores the first and last element.
    pub fn new(inner: G) -> Result<Self, TooFewElements> {
        if inner.len() < 2 {
            return Err(TooFewElements::new(inner.len()));
        }
        Ok(BorderDeletion { inner })
    }
}

impl<G> Generator<usize> for BorderDeletion<G>
where
    G: DiscreteGenerator,
{
    type Output = G::Output;
    fn gen(&self, input: usize) -> Self::Output {
        self.inner.gen(input + 1)
    }
}

impl<G> DiscreteGenerator for BorderDeletion<G>
where
    G: DiscreteGenerator,
{
    /// # Panics
    ///
    /// May Panic if the underlying generator has less than two elements.
    fn len(&self) -> usize {
        self.inner.len() - 2
    }
}

impl<G> SortedGenerator for BorderDeletion<G>
where
    G: SortedGenerator,
{
    fn strict_upper_bound_clamped(&self, element: Self::Output, min: usize, max: usize) -> usize
    where
        Self::Output: PartialOrd + Copy,
    {
        debug_assert!(max <= self.len());
        self.inner
            .strict_upper_bound_clamped(element, min + 1, max + 1)
            - 1
    }
}

#[cfg(test)]
mod test {
    use super::{BorderBuffer, BorderDeletion};
    use crate::{DiscreteGenerator, Equidistant, SortedGenerator};

    #[test]
    fn borderdeletion() {
        let del = BorderDeletion::new(Equidistant::normalized(11)).unwrap();
        assert_eq!(del.len(), 9);
        assert_eq!(del.strict_upper_bound_clamped(0.45, 0, del.len()), 4);
        assert_eq!(del.strict_upper_bound_clamped(-1.0, 0, del.len()), 0);
        assert_eq!(del.strict_upper_bound_clamped(10.0, 0, del.len()), 9);
        assert_eq!(del.strict_upper_bound(0.45), 4);
        assert_eq!(del.strict_upper_bound(-1.0), 0);
        assert_eq!(del.strict_upper_bound(10.0), 9);
        assert_eq!(del.strict_upper_bound_clamped(0.8, 1, 5), 5);
        assert_eq!(del.strict_upper_bound_clamped(0.45, 3, 7), 4);
    }

    #[test]
    fn borderbuffer() {
        let buf = BorderBuffer::new(Equidistant::normalized(11), 3);
        assert_eq!(buf.len(), 17);
        assert_eq!(buf.strict_upper_bound_clamped(0.45, 0, buf.len()), 8);
        assert_eq!(buf.strict_upper_bound_clamped(-1.0, 0, buf.len()), 0);
        assert_eq!(buf.strict_upper_bound_clamped(10.0, 0, buf.len()), 17);
        assert_eq!(buf.strict_upper_bound(0.45), 8);
        assert_eq!(buf.strict_upper_bound(-1.0), 0);
        assert_eq!(buf.strict_upper_bound(10.0), 17);
        assert_eq!(buf.strict_upper_bound_clamped(0.8, 1, 5), 5);
        assert_eq!(buf.strict_upper_bound_clamped(0.45, 3, 9), 8);
    }
}
