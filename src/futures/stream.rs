use futures_std::stream::Stream;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use {ErrorCompat, IntoError};

pub trait StreamExt<T, E>
where
    Self: Stream<Item = Result<T, E>>,
    Self: Sized,
{
    fn context<C, E2>(self, context: C) -> Context<Self, C, E2>
    where
        C: IntoError<E2, Source = E> + Clone,
        E2: std::error::Error + ErrorCompat;

    fn with_context<F, C, E2>(self, context: F) -> WithContext<Self, F, E2>
    where
        F: FnMut() -> C,
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat;
}

impl<St, T, E> StreamExt<T, E> for St
where
    Self: Stream<Item = Result<T, E>>,
{
    fn context<C, E2>(self, context: C) -> Context<Self, C, E2>
    where
        C: IntoError<E2, Source = E> + Clone,
        E2: std::error::Error + ErrorCompat,
    {
        Context {
            inner: self,
            context,
            _e2: PhantomData,
        }
    }

    fn with_context<F, C, E2>(self, context: F) -> WithContext<Self, F, E2>
    where
        F: FnMut() -> C,
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat,
    {
        WithContext {
            inner: self,
            context,
            _e2: PhantomData,
        }
    }
}

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Context<St, C, E2> {
    inner: St,
    context: C,
    _e2: PhantomData<E2>,
}

impl<St, C, E, E2, T> Stream for Context<St, C, E2>
where
    St: Stream<Item = Result<T, E>>,
    C: IntoError<E2, Source = E> + Clone,
    E2: std::error::Error + ErrorCompat,
{
    type Item = Result<T, E2>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Option<Self::Item>> {
        // FIXME doc safety
        unsafe {
            let inner = self.as_mut().map_unchecked_mut(|slf| &mut slf.inner);
            match inner.poll_next(ctx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Ready(Some(Ok(v))) => Poll::Ready(Some(Ok(v))),
                Poll::Ready(Some(Err(error))) => {
                    let context = self.context.clone();
                    let error = context.into_error(error);
                    Poll::Ready(Some(Err(error)))
                }
            }
        }
    }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct WithContext<St, F, E2> {
    inner: St,
    context: F,
    _e2: PhantomData<E2>,
}

impl<St, F, C, E, E2, T> Stream for WithContext<St, F, E2>
where
    St: Stream<Item = Result<T, E>>,
    F: FnMut() -> C,
    C: IntoError<E2, Source = E>,
    E2: std::error::Error + ErrorCompat,
{
    type Item = Result<T, E2>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Option<Self::Item>> {
        // FIXME doc safety
        unsafe {
            let inner = self.as_mut().map_unchecked_mut(|slf| &mut slf.inner);
            match inner.poll_next(ctx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Ready(Some(Ok(v))) => Poll::Ready(Some(Ok(v))),
                Poll::Ready(Some(Err(error))) => {
                    let context = &mut self.get_unchecked_mut().context;
                    let error = context().into_error(error);
                    Poll::Ready(Some(Err(error)))
                }
            }
        }
    }
}
