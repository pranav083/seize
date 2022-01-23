mod raw;
mod utils;

use std::marker::PhantomData;

pub struct Crystalline<const SLOTS: usize> {
    raw: raw::Crystalline<SLOTS>,
}

impl<const SLOTS: usize> Crystalline<SLOTS> {
    pub fn new() -> Self {
        Self {
            raw: raw::Crystalline::with_threads(1),
        }
    }

    pub fn guard(&self) -> Guard<'_, SLOTS> {
        Guard {
            crystalline: self,
            _not_send: PhantomData,
        }
    }

    pub fn link<T>(&self, value: T) -> Linked<T> {
        Linked {
            value,
            node: self.raw.node_for::<T>(),
        }
    }

    pub fn link_boxed<T>(&self, value: T) -> *mut Linked<T> {
        Box::into_raw(Box::new(self.link(value)))
    }
}

pub struct Shared<'g, T> {
    ptr: *mut Linked<T>,
    guard: PhantomData<&'g T>,
}

impl<T> Shared<'_, T> {
    pub fn as_ptr(&self) -> *mut Linked<T> {
        self.ptr
    }
}

impl<T> Clone for Shared<'_, T> {
    fn clone(&self) -> Self {
        Shared {
            ptr: self.ptr,
            guard: PhantomData,
        }
    }
}

impl<T> Copy for Shared<'_, T> {}

impl<'g, T> Shared<'g, T> {
    pub unsafe fn deref(&self) -> &'g T {
        &(*self.ptr).value
    }
}

pub struct Protect(pub usize);

pub struct Guard<'a, const SLOTS: usize> {
    crystalline: &'a Crystalline<SLOTS>,
    _not_send: PhantomData<*mut ()>,
}

impl<'g, const SLOTS: usize> Guard<'g, SLOTS> {
    pub unsafe fn retire<T>(&self, ptr: *mut Linked<T>, retire: unsafe fn(Link)) {
        self.crystalline.raw.retire(ptr, retire)
    }

    pub fn protect<T>(
        &self,
        op: impl FnMut() -> *mut Linked<T>,
        protect: Protect,
    ) -> Shared<'g, T> {
        Shared {
            ptr: self.crystalline.raw.protect(op, protect.0),
            guard: PhantomData,
        }
    }
}

impl<const SLOTS: usize> Drop for Guard<'_, SLOTS> {
    fn drop(&mut self) {
        unsafe { self.crystalline.raw.clear_all() }
    }
}

pub struct Link {
    node: *mut raw::Node,
}

impl Link {
    pub unsafe fn as_ptr<T>(&mut self) -> *mut Linked<T> {
        self.node as *mut _
    }
}

#[repr(C)]
pub struct Linked<T> {
    node: raw::Node,
    pub value: T,
}

pub unsafe fn retire_boxed<T>(mut link: Link) {
    let _ = Box::from_raw(link.as_ptr::<T>());
}

pub unsafe fn retire_in_place<T>(mut link: Link) {
    let _ = std::ptr::drop_in_place(link.as_ptr::<T>());
}
