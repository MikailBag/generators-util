//! Defines iterators for the generators.
use core::{
    marker::Unpin,
    ops::{Generator, GeneratorState},
    pin::Pin,
};

/// Simple iterator over Yields. Supported generator must have
/// () as resume argument and () or ! as return value. This iterator
/// does not keep track of the generator state, that's why behavior of polling
/// after finish is unspecified (usually underlying generator will panic).
pub struct SimpleGenIterator<G>(pub(crate) G);

pub trait SupportedReturnValue {}

impl SupportedReturnValue for () {}
impl SupportedReturnValue for ! {}

impl<G> Iterator for SimpleGenIterator<G>
where
    G: Generator<()> + Unpin,
    G::Return: SupportedReturnValue,
{
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.0).resume(()) {
            GeneratorState::Yielded(y) => Some(y),
            GeneratorState::Complete(_) => None,
        }
    }
}

/// Low-level adapter, yielding `GeneratorState`s directly.
/// This iterator is "endless" - it never returns None. This iterator
/// does not keep track of the generator state, that's why behavior of polling
/// after finish is unspecified (usually underlying generator will panic).
pub struct RawGenIterator<G> {
    pub(crate) gen: G,
}

impl<G> Iterator for RawGenIterator<G>
where
    G: Generator<()> + Unpin,
{
    type Item = GeneratorState<G::Yield, G::Return>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Pin::new(&mut self.gen).resume(()))
    }
}

/// High-level fused iterator. When underlying generator returns,
/// all subsequent calls to `next` will return None. Additionally,
/// this iterator stores generator return value.
pub enum FusedGenIterator<G: Generator<()>> {
    /// Generator have not completed yet.
    Generator(G),
    /// Generator completed.
    Completed(G::Return),
}

impl<G: Generator<()> + Unpin> FusedGenIterator<G> {
    pub(crate) fn new(g: G) -> Self {
        FusedGenIterator::Generator(g)
    }

    /// Checks if more values can still be yielded.
    pub fn is_completed(&self) -> bool {
        matches!(self, FusedGenIterator::Generator(_))
    }
}

impl<G: Generator<()> + Unpin> Iterator for FusedGenIterator<G> {
    type Item = G::Yield;
    fn next(&mut self) -> Option<Self::Item> {
        let gen = match self {
            FusedGenIterator::Generator(gen) => Pin::new(gen),
            FusedGenIterator::Completed(_) => return None,
        };
        match gen.resume(()) {
            GeneratorState::Yielded(y) => Some(y),
            GeneratorState::Complete(c) => {
                *self = FusedGenIterator::Completed(c);
                None
            }
        }
    }
}

impl<G: Generator<()> + Unpin> core::iter::FusedIterator for FusedGenIterator<G> {}
