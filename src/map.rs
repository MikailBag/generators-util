//! Map adapters.
use core::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};
use pin_project::pin_project;

/// Generator adapter that applies provided function to each yielded value.
#[pin_project]
pub struct MapYield<G, F> {
    #[pin]
    pub(crate) gen: G,
    pub(crate) func: F,
}
impl<R, G, F, T> Generator<R> for MapYield<G, F>
where
    G: Generator<R>,
    F: FnMut(G::Yield) -> T,
{
    type Yield = T;
    type Return = G::Return;

    fn resume(self: Pin<&mut Self>, arg: R) -> GeneratorState<Self::Yield, Self::Return> {
        let this = self.project();
        match this.gen.resume(arg) {
            GeneratorState::Complete(c) => GeneratorState::Complete(c),
            GeneratorState::Yielded(y) => GeneratorState::Yielded((this.func)(y)),
        }
    }
}

/// Generator adapter that applies provided function to return value.
#[pin_project]
pub struct MapReturn<G, F> {
    #[pin]
    pub(crate) gen: G,
    pub(crate) func: F,
}

impl<R, G, F, T> Generator<R> for MapReturn<G, F>
where
    G: Generator<R>,
    F: FnMut(G::Return) -> T,
{
    type Yield = G::Yield;
    type Return = T;

    fn resume(self: Pin<&mut Self>, arg: R) -> GeneratorState<Self::Yield, Self::Return> {
        let this = self.project();
        match this.gen.resume(arg) {
            GeneratorState::Complete(c) => GeneratorState::Complete((this.func)(c)),
            GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
        }
    }
}

/// Generator adapter that applies provided function to resume arguments.
#[pin_project]
pub struct MapResume<G, F> {
    #[pin]
    pub(crate) gen: G,
    pub(crate) func: F,
}

impl<ResumeInner, ResumeOuter, G, F> Generator<ResumeOuter> for MapResume<G, F>
where
    G: Generator<ResumeInner>,
    F: FnMut(ResumeOuter) -> ResumeInner,
{
    type Yield = G::Yield;
    type Return = G::Return;

    fn resume(self: Pin<&mut Self>, arg: ResumeOuter) -> GeneratorState<Self::Yield, Self::Return> {
        let this = self.project();
        let arg = (this.func)(arg);
        this.gen.resume(arg)
    }
}
