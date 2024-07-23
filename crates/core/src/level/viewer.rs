use std::sync::{atomic::{AtomicI32, AtomicU16, Ordering}, Arc};

use futures::{future, StreamExt};
use proto::types::Dimension;
use util::Vector;

use super::{BoxRegion, PointRegion, Service};

pub struct Viewer {
    pub service: Arc<Service>,
    radius: AtomicU16,

    // The current position of this viewer in chunk coordinates.
    current_x: AtomicI32,
    current_z: AtomicI32
}

impl Viewer {
    pub const fn new(service: Arc<Service>) -> Viewer {
        Viewer {
            service,
            radius: AtomicU16::new(0),
            current_x: AtomicI32::new(0),
            current_z: AtomicI32::new(0)
        }
    }

    /// Updates the position of this viewer.
    pub fn update_position(&self, position: Vector<f32, 2>) {
        // Transform player coordinates to chunk coordinates.
        let chunk_x = (position.x / 16.0).ceil() as i32;
        let chunk_z = (position.y / 16.0).ceil() as i32;

        self.current_x.store(chunk_x, Ordering::Relaxed);
        self.current_z.store(chunk_z, Ordering::Relaxed);

        // Update view if required
        self.on_view_update();
    }

    /// Updates the render distance of this viewer
    #[inline]
    pub fn update_radius(&self, radius: u16) {
        self.radius.store(radius, Ordering::Relaxed);
        self.on_view_update();
    }
    
    fn on_view_update(&self) {
        let x = self.current_x.load(Ordering::Relaxed);
        let z = self.current_z.load(Ordering::Relaxed);

        // // Request the chunk the player is in
        // let stream = self.service.region(BoxRegion::from_bounds(
        //     (x, -4, z), (x, 15, z), Dimension::Overworld
        // ));

        // tokio::spawn(async move {
        //     let fut = stream.take(1).for_each(|res| {
        //         tracing::debug!("{res:?}");

        //         let chunk = res.data;
        //         chunk.serialize_network().unwrap();

        //         future::ready(())
        //     });

        //     fut.await;
        // });
    }
}