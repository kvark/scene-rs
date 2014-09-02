//! Render Queue

#![crate_name = "render"]

extern crate cgmath;
extern crate ecs;
extern crate gfx;

use std::{iter, num, slice};
use cgmath::Transform;

pub type Depth = u32;

pub type Scalar = f32;

pub type Space = cgmath::Decomposed<
    Scalar,
    cgmath::Vector3<Scalar>,
    cgmath::Quaternion<Scalar>
>;

struct Camera {
    space: Space,
    frustum: cgmath::Frustum<Scalar>,
}

pub struct Object<L, T> {
    batch: gfx::batch::RefBatch<L, T>,
    parameters: T,
    depth: Depth,
}

pub fn order_opaque<L, T>(a: &Object<L, T>, b: &Object<L, T>) -> Ordering {
    (&a.batch, a.depth).cmp(&(&b.batch, b.depth))
}

type Index = uint;

pub struct Queue<L, T> {
    pub objects: Vec<Object<L, T>>,
    indices: Vec<Index>,
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
        self.update();
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
}

pub struct View<L, T> {
    pub frame: gfx::Frame,
    pub camera: Camera,
    queue: Queue<L, T>,
}

impl<L, T: gfx::shade::ShaderParam<L>> View<L, T> {
    pub fn clear(&mut self) {
        self.queue.objects.clear()
    }

    pub fn add(&mut self, batch: gfx::batch::RefBatch<L, T>, data: T,
           world: &Space, _bound: cgmath::Sphere<Scalar>) {
        let view = self.camera.space.concat(world);
        let frustum = &self.camera.frustum;
        let depth_max: Depth = num::Bounded::max_value();
        let depth = (depth_max as f32 * (view.disp.z - frustum.near.d) /
            (frustum.far.d - frustum.near.d)) as Depth;
        //TODO: cull based on `bound`
        self.queue.objects.push(Object {
            batch: batch,
            parameters: data,
            depth: depth,
        });
    }

    pub fn render<'a, C: gfx::CommandBuffer>(&'a mut self, renderer: &mut gfx::Renderer<C>,
                  context: &'a gfx::batch::Context) {
        self.queue.sort(order_opaque);
        for ob in self.queue.objects() {
            renderer.draw((&ob.batch, &ob.parameters, context), &self.frame);
        }
    }
}
