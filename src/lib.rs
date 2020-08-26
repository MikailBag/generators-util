#![feature(generator_trait, never_type)]
#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
mod iter;
mod map;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

pub use iter::{FusedGenIterator, RawGenIterator, SimpleGenIterator};
pub use map::{MapResume, MapReturn, MapYield};

#[cfg(feature = "alloc")]
pub type BoxGenerator<Resume, Yield, Return> =
    Pin<Box<dyn Generator<Resume, Yield = Yield, Return = Return>>>;

/// This trait is automatically implemented for all generators.
pub trait GeneratorExt<R>: core::ops::Generator<R> {
    /// Apply provided function to each yielded value.
    fn map_yield<F, T>(self, func: F) -> MapYield<Self, F>
    where
        F: FnMut(Self::Yield) -> T,
        Self: Sized,
    {
        MapYield { gen: self, func }
    }

    /// Apply provided function to return value.
    fn map_return<F, T>(self, func: F) -> MapReturn<Self, F>
    where
        F: FnMut(Self::Return) -> T,
        Self: Sized,
    {
        MapReturn { gen: self, func }
    }

    /// Apply provided function to resume arguments.
    /// For example, if resume arg implements Default, and you want an
    /// iterator, you can use the following:
    /// ```
    /// #![feature(generator_trait, generators)]
    /// use core::ops::Generator;
    /// use core::marker::Unpin;
    /// use simple_generators_util::GeneratorExt;
    /// fn sum_of_yields<R: Default, G: Generator<R, Yield=u64, Return=()> + Unpin>(gen: G) -> u64 {
    ///    let gen = gen.map_resume(|()| R::default());
    ///    gen.iterate_over_yields().sum()
    /// }
    /// let generator = || {
    ///     for x in 0.. {
    ///         if x * x * x > 100 {
    ///             return;
    ///         }
    ///         yield x * x;
    ///     }    
    /// };
    /// assert_eq!(sum_of_yields(generator), 30);
    /// ```
    fn map_resume<F, T>(self, func: F) -> MapResume<Self, F>
    where
        F: FnMut(T) -> R,
        Self: Sized,
    {
        MapResume { gen: self, func }
    }

    /// `Generator::Resume`, but without explicit pinning.
    fn resume_unpin(&mut self, arg: R) -> GeneratorState<Self::Yield, Self::Return>
    where
        Self: Unpin,
    {
        Pin::new(self).resume(arg)
    }

    /// Utility method to pass a generator by mutable ref.
    fn by_ref(&mut self) -> &mut Self {
        self
    }

    /// Utility method for simple pinning.
    /// Alternatively, you can use macros like `tokio::pin!`
    /// for stack pinning.
    #[cfg(feature = "alloc")]
    fn boxed(self) -> Pin<Box<Self>>
    where
        Self: Sized,
    {
        Box::pin(self)
    }

    /// Converts this generator into iterator over yielded values.
    /// Can be used for simple construction of complex iterators.
    fn iterate_over_yields(self) -> SimpleGenIterator<Self>
    where
        Self: Sized + Unpin,
        Self::Return: iter::SupportedReturnValue,
    {
        SimpleGenIterator(self)
    }

    /// Converts this generator into iterator over `GeneratorState`.
    fn into_raw_iterator(self) -> RawGenIterator<Self>
    where
        Self: Generator<()> + Sized + Unpin,
    {
        RawGenIterator { gen: self }
    }

    /// Convers this iterator into fused iterator. That iterator
    /// also supports retrieving return value.
    fn into_fused_iterator(self) -> FusedGenIterator<Self>
    where
        Self: Generator<()> + Sized + Unpin,
    {
        FusedGenIterator::new(self)
    }
}

impl<R, G: Generator<R>> GeneratorExt<R> for G {}
