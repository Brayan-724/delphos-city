use std::any::type_name;
use std::ops;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Read/Write Counter
pub struct Rwc<D: ?Sized> {
    writers: Rc<AtomicUsize>,
    readers: Rc<AtomicUsize>,
    data: *mut D,
}

impl<D: ?Sized> Clone for Rwc<D> {
    fn clone(&self) -> Self {
        Self {
            writers: self.writers.clone(),
            readers: self.readers.clone(),
            data: self.data,
        }
    }
}

impl<D: ?Sized> Rwc<D> {
    pub fn new(data: Box<D>) -> Self {
        Self {
            writers: Rc::new(AtomicUsize::new(0)),
            readers: Rc::new(AtomicUsize::new(0)),
            data: Box::leak(data),
        }
    }

    pub fn map<R: ?Sized>(&self, f: impl FnOnce(*mut D) -> *mut R) -> Rwc<R> {
        Rwc {
            writers: self.writers.clone(),
            readers: self.readers.clone(),
            data: f(self.data),
        }
    }
}

impl<D: Sized + 'static> Rwc<D> {
    fn checked_read(&self) {
        if let n @ 1.. = self.writers.load(Ordering::Relaxed) {
            log::error!(target: "ecs::rwc", "A reader was created while {n} writers are alive for {}", type_name::<D>());
            if cfg!(debug_assertions) {
                panic!(
                    "A reader was created while {n} writers are alive for {}",
                    type_name::<D>()
                )
            }
        }
    }

    fn checked_write(&self, d: usize) {
        if let n @ 1.. = self.writers.fetch_add(d, Ordering::SeqCst) {
            log::error!(target: "ecs::rwc", "A writer was created while other {n} writers are alive for {}", type_name::<D>());
            if cfg!(debug_assertions) {
                panic!(
                    "A writer was created while other {n} writers are alive for {}",
                    type_name::<D>()
                )
            }
        }
    }

    pub fn read(&self) -> RwcReaderGuard<D> {
        self.readers.fetch_add(1, Ordering::SeqCst);
        self.checked_read();

        RwcReaderGuard {
            readers: self.readers.clone(),
            inner: self.data,
        }
    }

    pub fn write(&self) -> RwcWriterGuard<D> {
        self.checked_write(1);

        RwcWriterGuard {
            writers: self.writers.clone(),
            inner: self.data,
        }
    }
}

impl<D: Sized + 'static> ops::Deref for Rwc<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        self.checked_read();

        unsafe { &*self.data }
    }
}

// ------ ------ OWNED ------ ------

// ------ Reader ------

pub struct RwcReaderGuard<D: ?Sized> {
    readers: Rc<AtomicUsize>,
    inner: *mut D,
}

impl<D: ?Sized> Drop for RwcReaderGuard<D> {
    fn drop(&mut self) {
        self.readers.fetch_sub(1, Ordering::AcqRel);
    }
}

impl<D: ?Sized> ops::Deref for RwcReaderGuard<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

// ------ Writer ------

pub struct RwcWriterGuard<D: ?Sized> {
    writers: Rc<AtomicUsize>,
    inner: *mut D,
}

impl<D: ?Sized> Drop for RwcWriterGuard<D> {
    fn drop(&mut self) {
        self.writers.fetch_sub(1, Ordering::AcqRel);
    }
}

impl<D: ?Sized> ops::Deref for RwcWriterGuard<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl<D: ?Sized> ops::DerefMut for RwcWriterGuard<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}
