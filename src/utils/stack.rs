#[derive(Debug)]
pub struct Node<T> {
    pub data: T,
    pub next: Option<Box<Node<T>>>,
}

pub struct Stack<T> {
    pub top: Option<Node<T>>,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Node { data, next: None }
    }
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { top: None }
    }

    pub fn is_empty(self) -> bool {
        match self.top {
            Some(_) => false,
            None => true,
        }
    }

    pub fn push(&mut self, data: T) {
        let mut node = Node::<T>::new(data);
        if let Some(top) = std::mem::replace(&mut self.top, None) {
            node.next = Some(Box::new(top));
        }
        self.top = Some(node);
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(top) = std::mem::replace(&mut self.top, None) {
            self.top = match top.next {
                Some(n) => Some(*n),
                None => None,
            };
            Some(top.data)
        } else {
            None
        }
    }
}
