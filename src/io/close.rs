use super::{AsyncWrite, Result};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Future for the [`close`](super::AsyncWriteExt::close) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Close<'a, W: ?Sized> {
    writer: &'a mut W,
}

impl<W: ?Sized + Unpin> Unpin for Close<'_, W> {}

impl<'a, W: AsyncWrite + ?Sized + Unpin> Close<'a, W> {
    pub(super) fn new(writer: &'a mut W) -> Self {
        Close { writer }
    }
}

impl<W: AsyncWrite + ?Sized + Unpin> Future for Close<'_, W> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.writer).poll_close(cx)
    }
}
