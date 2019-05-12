use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use {ErrorCompat, IntoError};

pub trait FutureExt<T, E>
where
    Self: Future<Output = Result<T, E>>,
    Self: Sized,
{
    fn context<C, E2>(self, context: C) -> Context<Self, C, E2>
    where
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat;

    fn with_context<F, C, E2>(self, context: F) -> WithContext<Self, F, E2>
    where
        F: FnOnce() -> C,
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat;
}

impl<Fut, T, E> FutureExt<T, E> for Fut
where
    Self: Future<Output = Result<T, E>>,
{
    fn context<C, E2>(self, context: C) -> Context<Self, C, E2>
    where
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat,
    {
        Context {
            inner: self,
            context: Some(context),
            _e2: PhantomData,
        }
    }

    fn with_context<F, C, E2>(self, context: F) -> WithContext<Self, F, E2>
    where
        F: FnOnce() -> C,
        C: IntoError<E2, Source = E>,
        E2: std::error::Error + ErrorCompat,
    {
        WithContext {
            inner: self,
            context: Some(context),
            _e2: PhantomData,
        }
    }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct Context<Fut, C, E2> {
    inner: Fut,
    context: Option<C>,
    _e2: PhantomData<E2>,
}

impl<Fut, C, E, E2, T> Future for Context<Fut, C, E2>
where
    Fut: Future<Output = Result<T, E>>,
    C: IntoError<E2, Source = E>,
    E2: std::error::Error + ErrorCompat,
{
    type Output = Result<T, E2>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Self::Output> {
        // FIXME doc safety
        unsafe {
            let inner = self.as_mut().map_unchecked_mut(|slf| &mut slf.inner);
            let inner_res = inner.poll(ctx);

            inner_res.map_err(|error| {
                self.get_unchecked_mut()
                    .context
                    .take()
                    .expect("Cannot poll Context after it resolves")
                    .into_error(error)
            })
        }
    }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct WithContext<Fut, F, E2> {
    inner: Fut,
    context: Option<F>,
    _e2: PhantomData<E2>,
}

impl<Fut, F, C, E, E2, T> Future for WithContext<Fut, F, E2>
where
    Fut: Future<Output = Result<T, E>>,
    F: FnOnce() -> C,
    C: IntoError<E2, Source = E>,
    E2: std::error::Error + ErrorCompat,
{
    type Output = Result<T, E2>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Self::Output> {
        // FIXME doc safety
        unsafe {
            let inner = self.as_mut().map_unchecked_mut(|slf| &mut slf.inner);
            let inner_res = inner.poll(ctx);

            inner_res.map_err(|error| {
                let context = self
                    .get_unchecked_mut()
                    .context
                    .take()
                    .expect("Cannot poll Context after it resolves");

                context().into_error(error)
            })
        }
    }
}
