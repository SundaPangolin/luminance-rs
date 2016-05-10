//! Shader programs related types and functions.
//!
//! A shader `Program` is an object representing several operations. It’s a streaming program that
//! will operate on vertices, vertex patches, primitives and/or fragments.
//!
//! > *Note: shader programs don’t have to run on all those objects; they can be ran only on
//! vertices and fragments, for instance*.
//!
//! Creating a shader program is very simple. You need shader `Stage`s representing each step of the
//! processing. Here’s the actual mapping between the shader stage types and the processing unit:
//!
//! - `Stage<TessellationControlShader>`: ran on **tessellation parameters** ;
//! - `Stage<TessellationEvaluationShader>`: ran on **patches** ;
//! - `Stage<VertexShader>`: ran on **vertices** ;
//! - `Stage<GeometryShader>`: ran on **primitives** ;
//! - `Stage<FragmentShader>`: ran on **screen fragments**.
//!
//! You *have* to provide at least a `Stage<VertexShader>` and a `Stage<FragmentShader>`. If you
//! want tessellation processing, you need to provide both a `Stage<TessellationControlShader>` and
//! a `Stage<TessellationEvaluationShader>`. If you want primitives processing, you need to add a
//! `Stage<GeometryShader>`.
//!
//! In order to customize the behavior of your shader programs, you have access to *uniforms*. For
//! more details about them, see the documentation for the type `Uniform` and `Uniformable`. When
//! creating a new shader program, you have to provide code to declare its *uniform interface*. The
//! *uniform interface* refers to a type of your own that will be kept by the shader program and
//! exposed to you when you’ll express the need to update its uniforms. That *uniform interface* is
//! created via a closure you pass. That closure takes as arguments a function used to retrieve
//! `Uniform`s from the program being constructed. That pattern, that can be a bit overwhelming at
//! first, is important to keep things safe and functional. Keep in mind that you can make the
//! closure fail, so that you can notify a `Uniform` lookup failure, for instance.
//!
//! You can create a `Program` with its `new` associated function.
//!
//! # Example
//!
//! ```
//! // assume we have a vertex shader `vs` and fragment shader `fs`
//! let program = Program::new(None, &vs, None, &fs, |get_uni| {
//!   let resolution: Result<Uniform<[f32; 2]>, _> = get_uni("resolution");
//!   let time: Result<Uniform<f32>, _> = get_uni("time");
//!
//!   if let Err(err) = resolution {
//!     return Err(err);
//!   }
//!   let resolution = resolution.unwrap();
//!
//!   if let Err(err) = time {
//!     return Err(err);
//!   }
//!   let time = time.unwrap();
//!
//!   Ok(resolution, time)
//! });
//! ```

use shader::stage::*;
use shader::uniform::{HasUniform, Uniform, Uniformable, UniformName};

/// Trait to implement to provide shader program features.
pub trait HasProgram: HasStage + HasUniform {
  type Program;

  /// Create a new program by linking it with stages.
  fn new_program(tess: Option<(&Self::AStage, &Self::AStage)>, vertex: &Self::AStage, geometry: Option<&Self::AStage>, fragment: &Self::AStage) -> Result<Self::Program, ProgramError>;
  /// Free a program.
  fn free_program(program: &mut Self::Program);
  ///
  fn map_uniform(program: &Self::Program, name: UniformName) -> Result<Self::U, ProgramError>;
  ///
  fn update_uniforms<F>(program: &Self::Program, f: F) where F: Fn();
}

#[derive(Debug)]
pub struct Program<C> where C: HasProgram {
  pub repr: C::Program,
}

impl<C> Drop for Program<C> where C: HasProgram {
  fn drop(&mut self) {
    C::free_program(&mut self.repr)
  }
}

impl<C> Program<C> where C: HasProgram {
  pub fn new(tess: Option<(&Stage<C, TessellationControlShader>, &Stage<C, TessellationEvaluationShader>)>, vertex: &Stage<C, VertexShader>, geometry: Option<&Stage<C, GeometryShader>>, fragment: &Stage<C, FragmentShader>) -> Result<Self, ProgramError> {
    C::new_program(tess.map(|(tcs, tes)| (&tcs.repr, &tes.repr)), &vertex.repr, geometry.map(|g| &g.repr), &fragment.repr).map(|repr| {
      Program {
        repr: repr,
      }
    })
  }

  pub fn uniform<T>(&self, name: &str) -> Result<Uniform<C, T>, ProgramError> where T: Uniformable {
    C::map_uniform(&self.repr, UniformName::StringName(String::from(name))).map(|u| Uniform::new(u))
  }

  pub fn update<F>(&self, f: F) where F: Fn() {
    C::update_uniforms(&self.repr, f)
  }
}

#[derive(Debug)]
pub enum ProgramError {
  LinkFailed(String),
  InactiveUniform(String),
  UniformTypeMismatch(String)
}
