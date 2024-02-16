use std::sync::{atomic::{AtomicI32, AtomicU16, Ordering}, Arc};

use util::Vector;

use super::Service;

pub struct Viewer {
    service: Arc<Service>,
    radius: AtomicU16,

    // The current position of this viewer in chunk coordinates.
    current_x: AtomicI32,
    current_z: AtomicI32
}

impl Viewer {
    pub fn update_position(&self, position: Vector<f32, 2>) {
        // Transform player coordinates to chunk coordinates.
        let chunk_x = (position.x / 16.0).ceil() as i32;
        let chunk_z = (position.y / 16.0).ceil() as i32;

        self.current_x.store(chunk_x, Ordering::Relaxed);
        self.current_z.store(chunk_z, Ordering::Relaxed);

        // Update view if required
        self.on_view_update();
    }

    #[inline]
    pub fn update_radius(&self, radius: u16) {
        self.radius.store(radius, Ordering::Relaxed);
        self.on_view_update();
    }
    
    fn on_view_update(&self) {
        todo!()
    }
}