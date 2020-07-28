#![no_std]
#![feature(const_btree_new)]
#![feature(option_expect_none)]
#![feature(const_fn)]
extern crate alloc;

#[macro_use]
extern crate nb;

#[macro_use]
pub mod device;

pub(crate) mod events;
mod executor;
pub mod io;
pub mod schemes;

#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
pub use device::DeviceInterrupt;
use embedded_rust_devices::{ComponentConfiguration, Resource, ResourceID};
use nom_uri::Uri;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

pub struct Runtime<'device> {
    resource_ids: BTreeMap<Uri<'device>, ResourceID>,
    resources: Vec<&'device mut dyn Resource>,
    executor: executor::Executor,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    MultipleInitializations,
    ResourceNotFound,
    TaskQueueIsFull,
}

impl<'device> Runtime<'device> {
    pub fn init<F: FnOnce()>(
        heap_bottom: usize,
        heap_size: usize,
        max_events_per_prio: usize,
        resource_configuration: &[ComponentConfiguration],
        init_closure: F,
    ) -> Result<Self, RuntimeError> {
        Self::init_heap(heap_bottom, heap_size);
        events::ERROR_QUEUE
            .try_init_once(|| crossbeam_queue::ArrayQueue::new(max_events_per_prio))
            .or_else(|_| Err(RuntimeError::MultipleInitializations))?;
        events::CRITICAL_QUEUE
            .try_init_once(|| crossbeam_queue::ArrayQueue::new(max_events_per_prio))
            .or_else(|_| Err(RuntimeError::MultipleInitializations))?;
        events::NORMAL_QUEUE
            .try_init_once(|| crossbeam_queue::ArrayQueue::new(max_events_per_prio))
            .or_else(|_| Err(RuntimeError::MultipleInitializations))?;
        let mut rt = Self {
            resources: Vec::with_capacity(32),
            resource_ids: BTreeMap::new(),
            executor: executor::Executor::new(),
        };
        rt.configure(resource_configuration);
        init_closure();
        Ok(rt)
    }
    pub fn get_resource<'uri: 'device>(
        &mut self,
        id: ResourceID,
    ) -> Result<(ResourceID, &mut dyn Resource), RuntimeError> {
        Ok((
            id,
            *self
                .resources
                .get_mut(id.0 as usize)
                .expect("Resource id not found in vector"),
        ))
    }
    pub fn get_resource_from_uri<'uri: 'device>(
        &mut self,
        uri: &'uri Uri,
    ) -> Result<(ResourceID, &mut dyn Resource), RuntimeError> {
        let id = *match self.resource_ids.get_mut(uri) {
            Some(id) => id,
            None => return Err(RuntimeError::ResourceNotFound),
        };
        self.get_resource(id)
    }
    pub fn add_resource<'uri: 'device>(
        &mut self,
        uri: Uri<'uri>,
        resource: &'device mut dyn Resource,
    ) -> ResourceID {
        let id = self.resources.len();
        self.resources.push(resource);
        self.resource_ids.insert(uri, ResourceID(id as u8));
        ResourceID(id as u8)
    }
    /// Creates a new heap with the given bottom and size. The bottom address must be
    /// valid and the memory in the [heap_bottom, heap_bottom + heap_size) range must not
    /// be used for anything else. This function is unsafe because it can cause undefined
    /// behavior if the given address is invalid.
    fn init_heap(heap_bottom: usize, heap_size: usize) {
        unsafe { ALLOCATOR.lock().init(heap_bottom, heap_size) };
    }
    pub fn run(&mut self) -> ! {
        loop {
            while let Some(event) = events::next() {
                self.handle_event(event);
            }
            // TODO: ? waker checken ?
            // TODO: do something  with the result!
            self.executor.run();
            cortex_m::asm::wfi(); // safe power till next interrupt
        }
    }
    pub fn spawn_task(&mut self, task: Task, priority: usize) {
        self.executor.spawn(task, priority)
    }
    fn handle_event(&self, event: events::Event) {
        match event {
            _ => unimplemented!(),
        }
    }
    fn configure(&mut self, configuration: &[ComponentConfiguration]) {}
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}