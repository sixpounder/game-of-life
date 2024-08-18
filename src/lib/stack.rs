use std::collections::VecDeque;

#[derive(Debug)]
pub struct WindowedStack<T> {
    size: usize,
    inner: VecDeque<T>
}

impl<T> Default for WindowedStack<T> {
    fn default() -> Self {
        Self::new(10)
    }
}

impl<T> WindowedStack<T> {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            inner: VecDeque::default()
        }
    }

    pub fn push(&mut self, item: T) {
        self.inner.push_back(item);
        if self.inner.len() > self.size {
            self.inner.pop_front();
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop_back()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

