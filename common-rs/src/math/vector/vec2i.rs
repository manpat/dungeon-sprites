use crate::math::vector::Vec2;
use rand_derive2::RandGen;


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, RandGen)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vec2i {
	pub x: i32,
	pub y: i32,
}

impl Vec2i {
	pub const fn new(x: i32, y: i32) -> Vec2i { Vec2i{x, y} }
	pub const fn splat(x: i32) -> Vec2i { Vec2i::new(x, x) }
	pub const fn zero() -> Vec2i { Vec2i::splat(0) }

	pub fn from_tuple(t: (i32,i32)) -> Vec2i { Vec2i::new(t.0, t.1) }
	pub fn to_tuple(self) -> (i32,i32) { (self.x, self.y) }
	pub fn to_array(self) -> [i32; 2] { [self.x, self.y] }
	pub fn to_vec2(self) -> Vec2 { Vec2::new(self.x as f32, self.y as f32) }

	/// Swaps x and y elements.
	pub fn transpose(self) -> Vec2i {
		Vec2i::new(self.y, self.x)
	}

	pub fn length(self) -> f32 {
		((self.x*self.x + self.y*self.y) as f32).sqrt()
	}
}


impl From<[i32; 2]> for Vec2i {
	fn from([x, y]: [i32; 2]) -> Vec2i { Vec2i{x, y} }
}

impl From<(i32, i32)> for Vec2i {
	fn from((x, y): (i32, i32)) -> Vec2i { Vec2i{x, y} }
}