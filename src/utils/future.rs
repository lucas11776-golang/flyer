pub struct SendFuture<F>(pub F);

unsafe impl<F> Send for SendFuture<F> {}


impl<F: std::future::Future> std::future::Future for SendFuture<F> {
    type Output = F::Output;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        return unsafe {
            self.map_unchecked_mut(|s| &mut s.0).poll(cx)
        };
    }
}