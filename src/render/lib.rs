//! Render Queue

#![crate_name = "render"]

extern crate cgmath;
extern crate ecs;
extern crate gfx;

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
    bounding_sphere: cgmath::Sphere<Scalar>,
	depth: Depth,
}

pub fn order_opaque<L, T>(a: &Object<L, T>, b: &Object<L, T>) -> Ordering {
    Equal
}

type Index = uint;

pub struct Queue<L, T> {
    pub objects: Vec<Object<L, T>>,
    indices: Vec<Index>,
	context: gfx::batch::Context,
}

impl<L, T> Queue<L, T> {
    fn update(&mut self, order: |&Object<L, T>, &Object<L, T>| -> Ordering) {
        // synchronize indices to have the same length
        let ni = self.indices.len();
        if self.objects.len() > ni {
            self.indices.grow_fn(self.objects.len() - ni, |i| (ni + i) as Index);
        }else
        if self.objects.len() < ni {
            self.indices.retain(|&i| (i as uint) < ni);
        }
        debug_assert_eq!(self.objects.len(), self.indices.len());
        // sort
        let objects = self.objects.as_slice();
        self.indices.sort_by(|&ia, &ib|
            (order)(&objects[ia], &objects[ib])
        );
        // done
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
