use gfx;
use scenegraph::ces;
use w = world;

pub struct System {
    extents: [f32, ..2],
    pub frame: gfx::Frame,
    pub meshes: ces::Array<gfx::Mesh>,
    pub states: ces::Array<gfx::DrawState>,
}

impl System {
    pub fn new(extents: [f32, ..2], frame: gfx::Frame) -> System {
        System {
            extents: extents,
            frame: frame,
            meshes: ces::Array::new(),
            states: ces::Array::new(),
        }
    }

    fn render<'a>(&mut self, renderer: &mut gfx::Renderer, drawable: &'a w::Drawable,
              param: &'a w::ShaderParam) {
        let mesh = self.meshes.get(drawable.mesh_id);
        let state = self.states.get(drawable.state_id);
        renderer.draw(mesh, drawable.slice, &self.frame,
            (&drawable.program, param), state).unwrap();
    }
}

impl w::System for System {
    fn process(&mut self, &(_, ref mut renderer): w::Params, data: &mut w::Components,
               entities: &mut Vec<w::Entity>) {
        let clear_data = gfx::ClearData {
            color: Some(gfx::Color([0.0, 0.0, 0.1, 0.0])),
            depth: None,
            stencil: None,
        };
        renderer.clear(clear_data, &self.frame);
        let mut param = w::ShaderParam {
            transform: [0.0, 0.0, 0.0, 1.0],
            screen_scale: [1.0 / self.extents[0], 1.0 / self.extents[1], 0.0, 0.0],
        };
        for ent in entities.iter() {
            ent.draw.map(|d_id| {
                let drawable = data.draw.get_mut(d_id);
                match ent.space {
                    Some(s_id) => {
                        let s = data.space.get(s_id);
                        param.transform = [s.pos.x, s.pos.y, s.orient.s, s.scale];
                    }
                    None => ()
                }
                self.render(*renderer, drawable, &param)
            });
        }
    }
}
