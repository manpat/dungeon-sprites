//! Everything relating to interacting with the graphics context, from low-level, more direct access,
//! to higher level utilities.
//!
//! The core of this system is [`system::System`].
//! Other submodules that will likely be used often are [`buffer`] and [`mesh`].

/// Low level bindings to the OpenGL 4.5 API.
pub mod raw {
	include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub mod system;
pub mod draw_context;
pub mod resources;
pub mod resource_context;
pub mod vao;
pub mod buffer;
pub mod texture;
pub mod framebuffer;
pub mod vertex;
pub mod shader;
pub mod query;
pub mod capabilities;
pub mod mesh;

#[doc(inline)] pub use self::system::*;
#[doc(inline)] pub use self::draw_context::*;
#[doc(inline)] pub use self::resource_context::*;
#[doc(inline)] pub use self::resources::*;
#[doc(inline)] pub use self::buffer::*;
#[doc(inline)] pub use self::mesh::*;
#[doc(inline)] pub use self::texture::*;
#[doc(inline)] pub use self::framebuffer::*;
#[doc(inline)] pub use self::vao::*;
#[doc(inline)] pub use self::vertex::*;
#[doc(inline)] pub use self::shader::*;
#[doc(inline)] pub use self::query::*;
#[doc(inline)] pub use self::capabilities::*;

/// The kind of primitive to be generated by a draw call.
/// Used [`DrawContext::draw_arrays`],  [`DrawContext::draw_indexed`], and [`DrawContext::draw_instances_indexed`].
pub enum DrawMode {
	Points,
	Lines,
	LineStrip,
	Triangles,
}

impl DrawMode {
	fn into_gl(self) -> u32 {
		match self {
			DrawMode::Points => raw::POINTS,
			DrawMode::Lines => raw::LINES,
			DrawMode::LineStrip => raw::LINE_STRIP,
			DrawMode::Triangles => raw::TRIANGLES,
		}
	}
}


bitflags::bitflags! {
	/// Which planes should be cleared by [`DrawContext::clear`].
	pub struct ClearMode : u32 {
		const COLOR = 0b001;
		const DEPTH = 0b010;
		const STENCIL = 0b100;
		const ALL = 0b111;
	}
}

impl ClearMode {
	fn into_gl(self) -> u32 {
		let mut gl_value = 0;
		if self.contains(Self::COLOR) { gl_value |= raw::COLOR_BUFFER_BIT }
		if self.contains(Self::DEPTH) { gl_value |= raw::DEPTH_BUFFER_BIT }
		if self.contains(Self::STENCIL) { gl_value |= raw::STENCIL_BUFFER_BIT }
		gl_value
	}
}