//! This program demonstrates how to render a triangle without sending anything to the GPU. This is
//! a not-so-famous technique to reduce the bandwidth and procedurally generate all the required
//! data to perform the render. The trick lives in ordering the GPU to render a certain number of
//! vertices and “spawn” the vertices’ data directly in the vertex shader by using the identifier of
//! the vertex currently being mapped over.
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

extern crate luminance;
extern crate luminance_glfw;

use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::render_state::RenderState;
use luminance::shader::program::Program;
use luminance::tess::{Mode, TessBuilder};
use luminance_glfw::event::{Action, Key, WindowEvent};
use luminance_glfw::surface::{GlfwSurface, Surface, WindowDim, WindowOpt};

const VS: &'static str = include_str!("vs.glsl");
const FS: &'static str = include_str!("fs.glsl");

fn main() {
  let mut surface = GlfwSurface::new(
    WindowDim::Windowed(960, 540),
    "Hello, world!",
    WindowOpt::default(),
  )
  .expect("GLFW surface creation");

  // we don’t use a Vertex type anymore (i.e. attributeless, so we use the unit () type)
  let (program, _) = Program::<(), (), ()>::from_strings(None, VS, None, FS).expect("program creation");

  // yet, we still need to tell luminance to render a certain number of vertices (even if we send no
  // attributes / data); in our case, we’ll just render a triangle, which has three vertices
  let tess = TessBuilder::new(&mut surface)
    .set_vertex_nb(3)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let mut back_buffer = Framebuffer::back_buffer(surface.size());

  'app: loop {
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::FramebufferSize(width, height) => {
          back_buffer = Framebuffer::back_buffer([width as u32, height as u32]);
        }

        _ => (),
      }
    }

    surface
      .pipeline_builder()
      .pipeline(&back_buffer, [0., 0., 0., 0.], |_, shd_gate| {
        shd_gate.shade(&program, |rdr_gate, _| {
          rdr_gate.render(RenderState::default(), |tess_gate| {
            // render the tessellation to the surface the regular way and let the vertex shader’s
            // magic do the rest!
            tess_gate.render(&mut surface, (&tess).into());
          });
        });
      });

    surface.swap_buffers();
  }
}
