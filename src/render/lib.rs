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

pub struct Context {
	pub meshes: ecs::Array<gfx::Mesh>,
	pub states: ecs::Array<gfx::DrawState>,
}

pub struct Object {
	mesh_id: ecs::Id<gfx::Mesh>,
	slice: gfx::Slice,
	state_id: ecs::Id<gfx::DrawState>,
	program: gfx::shade::DictionaryProgram,
    bounding_sphere: cgmath::Sphere<Scalar>,
	depth: Depth,
}

pub fn order_opaque(a: &Object, b: &Object) -> Ordering {
    a.mesh_id.cmp(&b.mesh_id)
}

type Index = uint;

pub struct Queue {
    pub objects: Vec<Object>,
    indices: Vec<Index>,
}

impl Queue {
    fn update(&mut self, order: |&Object, &Object| -> Ordering) {
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

pub struct View {
    pub frame: gfx::Frame,
    pub camera: Camera,
	pub queue: Queue,
}

impl View {
	fn add(&mut self, mesh: &gfx::Mesh, slice: gfx::Slice, state: &gfx::DrawState,
		program: gfx::shade::DictionaryProgram, bound: cgmath::Sphere<Scalar>) {
		let _depth = 0u;
		//self.queue.objects.push(Object {})
	}
}
