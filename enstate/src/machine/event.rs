#[derive(Debug, Clone, Copy)]
pub enum Event<T> {
    Event(T),
    NoEvent,
}

impl<T> Event<T> {
    pub fn has_occurred(&self) -> bool {
        match self {
            Event::Event(_) => true,
            _ => false,
        }
    }

    pub fn map<B, F: Fn(T) -> B>(self, f: F) -> Event<B> {
        match self {
            Event::Event(x) => Event::Event(f(x)),
            Event::NoEvent => Event::NoEvent,
        }
    }
}

impl<T> From<Option<T>> for Event<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(x) => Event::Event(x),
            None => Event::NoEvent,
        }
    }
}

impl<T> From<Event<T>> for Option<T> {
    fn from(event: Event<T>) -> Self {
        match event {
            Event::Event(x) => Some(x),
            Event::NoEvent => None,
        }
    }
}
