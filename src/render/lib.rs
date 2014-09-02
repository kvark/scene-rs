//! Render Queue

#![crate_name = "render"]

extern crate cgmath;
extern crate ecs;
extern crate gfx;

use std::{iter, slice};

// normalized depth
pub type Depth = u16;

pub type Scalar = f32;

pub type Transform = cgmath::Decomposed<
    Scalar,
    cgmath::Quaternion<Scalar>,
    cgmath::Point3<Scalar>
>;

struct Camera {
    transform: Transform,
    frustum: cgmath::Frustum<Scalar>,
}

pub struct Object<L, T> {
    batch: gfx::batch::RefBatch<L, T>,
    data: T,
    bounding_sphere: cgmath::Sphere<Scalar>,
    depth: Depth,
}

pub fn order_opaque<L, T>(a: &Object<L, T>, b: &Object<L, T>) -> Ordering {
    (&a.batch, a.depth).cmp(&(&b.batch, b.depth))
}

type Index = uint;

pub struct Queue<L, T> {
    pub objects: Vec<Object<L, T>>,
    indices: Vec<Index>,
    context: gfx::batch::Context,
}

struct ObjectIter<'a, L: 'a, T: 'a> {
    objects: &'a [Object<L, T>],
    id_iter: slice::Items<'a, Index>,
}

impl<'a, L, T> Iterator<&'a Object<L, T>> for ObjectIter<'a, L, T> {
    fn next(&mut self) -> Option<&'a Object<L, T>> {
        self.id_iter.next().map(|&i| &self.objects[i])
    }
}

impl<L, T: gfx::shade::ShaderParam<L>> Queue<L, T> {
    fn is_updated(&self) -> bool {
        self.objects.len() == self.indices.len()
    }

    /// Synchronize indices to have the same length as objects
    pub fn update(&mut self) {
        let ni = self.indices.len();
        if self.objects.len() > ni {
            self.indices.grow_fn(self.objects.len() - ni, |i| (ni + i) as Index);
        }else
        if self.objects.len() < ni {
            self.indices.retain(|&i| (i as uint) < ni);
        }
        debug_assert!(self.is_updated());
    }

    /// Sort objects with the given order
    pub fn sort(&mut self, order: |&Object<L, T>, &Object<L, T>| -> Ordering) {
        debug_assert!(self.is_updated());
        let objects = self.objects.as_slice();
        self.indices.sort_by(|&ia, &ib|
            (order)(&objects[ia], &objects[ib])
        );
    }

    /// Iterate objects in the sorted order
    pub fn objects<'a>(&'a self) -> ObjectIter<'a, L, T>  {
        debug_assert!(self.is_updated());
        ObjectIter {
            objects: self.objects.as_slice(),
            id_iter: self.indices.iter(),
        }
    }

    // render everything to the given frame
    pub fn render<C: gfx::CommandBuffer>(&self, renderer: &mut gfx::Renderer<C>,
                  frame: &gfx::Frame) {
        debug_assert!(self.is_updated());
        for &i in self.indices.iter() {
            let ob = &self.objects[i];
            renderer.draw((&ob.batch, &ob.data, &self.context), frame);
        }
    }
}

pub struct View<L, T> {
    pub frame: gfx::Frame,
    pub camera: Camera,
    pub queue: Queue<L, T>,
}

impl<L, T> View<L, T> {
    fn add(&mut self, batch: gfx::batch::RefBatch<L, T>, bound: cgmath::Sphere<Scalar>) {
        let _depth = 0u;
        //self.queue.objects.push(Object {})
    }
}
