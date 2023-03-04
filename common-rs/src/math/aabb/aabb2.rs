use crate::math::vector::Vec2;

/// A Closed 2D Range - that is min and max count as being inside the bounds of the Aabb2
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Aabb2 {
	pub min: Vec2,
	pub max: Vec2,
}

impl Aabb2 {
	pub fn new(a: Vec2, b: Vec2) -> Aabb2 {
		let min = Vec2::new(a.x.min(b.x), a.y.min(b.y));
		let max = Vec2::new(a.x.max(b.x), a.y.max(b.y));
		Aabb2 { min, max }
	}

	pub fn new_empty() -> Aabb2 {
		Aabb2 {
			min: Vec2::splat(f32::INFINITY),
			max: Vec2::splat(-f32::INFINITY)
		}
	}

	pub fn around_point(center: Vec2, extents: Vec2) -> Aabb2 {
		Aabb2::new(center - extents, center + extents)
	}

	pub fn is_empty(&self) -> bool {
		self.min.x >= self.max.x
		|| self.min.y >= self.max.y
	}

	pub fn scale(&self, factor: f32) -> Aabb2 {
		Aabb2::new(self.min * factor, self.max * factor)
	}

	pub fn contains_point(&self, point: Vec2) -> bool {
		self.min.x <= point.x && point.x <= self.max.x
		&& self.min.y <= point.y && point.y <= self.max.y
	}

	pub fn size(&self) -> Vec2 {
		self.max - self.min
	}

	pub fn map_to_percentage(&self, p: Vec2) -> Vec2 {
		(p - self.min) / self.size()
	}

	pub fn map_from_percentage(&self, p: Vec2) -> Vec2 {
		p * self.size() + self.min
	}
}

