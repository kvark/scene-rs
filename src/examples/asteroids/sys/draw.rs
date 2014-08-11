use gfx;
use scenegraph::ces;
use w = world;

pub struct System {
	pub frame: gfx::Frame,
	pub meshes: ces::Array<gfx::Mesh>,
	pub states: ces::Array<gfx::DrawState>,
}

impl System {
	pub fn new(frame: gfx::Frame) -> System {
		System {
			frame: frame,
			meshes: ces::Array::new(),
			states: ces::Array::new(),
		}
	}

	fn render(&mut self, ren: &mut gfx::Renderer, drawable: &w::Drawable) {
		let mesh = self.meshes.get(drawable.mesh_id);
		let state = self.states.get(drawable.state_id);
		ren.draw(mesh, drawable.slice, &self.frame, &drawable.program, state).unwrap();
	}
}

impl w::System for System {
	fn process(&mut self, (_, renderer): w::Params, data: &mut w::Components,
	           entities: &mut Vec<w::Entity>) {
		let clear_data = gfx::ClearData {
			color: Some(gfx::Color([0.1, 0.1, 0.1, 0.0])),
			depth: None,
			stencil: None,
		};
		renderer.clear(clear_data, self.frame);
		for ent in entities.iter() {
			ent.draw.map(|d_id| {
				let drawable = data.draw.get_mut(d_id);
				ent.space.map(|s_id| {
					let s = data.space.get(s_id);
					drawable.program.data.transform = [s.pos.x, s.pos.y, s.orient.s, s.scale];
				});
				self.render(renderer, drawable)
			});
		}
	}
}
