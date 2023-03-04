use crate::math::vector::Vec2i;
use crate::math::aabb::Aabb2;

/// A Half Open 2D Range - that is inclusive of min and exclusive of max
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Aabb2i {
	pub min: Vec2i,
	pub max: Vec2i,
}

impl Aabb2i {
	pub fn new(a: Vec2i, b: Vec2i) -> Aabb2i {
		let min = Vec2i::new(a.x.min(b.x), a.y.min(b.y));
		let max = Vec2i::new(a.x.max(b.x), a.y.max(b.y));
		Aabb2i { min, max }
	}

	pub fn new_empty() -> Aabb2i {
		Aabb2i::default()
	}

	pub fn around_point(center: Vec2i, extents: Vec2i) -> Aabb2i {
		Aabb2i::new(center - extents, center + extents)
	}

	pub fn from_min_point(min: Vec2i, size: Vec2i) -> Aabb2i {
		Aabb2i::new(min, min + size)
	}

	pub fn to_aabb2(&self) -> Aabb2 {
		Aabb2::new(self.min.to_vec2(), self.max.to_vec2())
	}

	pub fn scale(&self, factor: i32) -> Aabb2i {
		Aabb2i::new(self.min * factor, self.max * factor)
	}

	pub fn union(&self, rhs: &Aabb2i) -> Aabb2i {
		if self.is_empty() { return *rhs; }
		if rhs.is_empty() { return *self; }

		let min_rect = Aabb2i::new(self.min, rhs.min);
		let max_rect = Aabb2i::new(self.max, rhs.max);
		Aabb2i::new(min_rect.min, max_rect.max)
	}

	pub fn is_empty(&self) -> bool {
		self.min.x >= self.max.x
		|| self.min.y >= self.max.y
	}

	pub fn contains_point(&self, point: Vec2i) -> bool {
		self.min.x <= point.x && point.x < self.max.x
		&& self.min.y <= point.y && point.y < self.max.y
	}

	pub fn size(&self) -> Vec2i {
		self.max - self.min
	}
}




#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new_aabb_wraps_points() {
		let start = Vec2i::new(0, 1);
		let end = Vec2i::new(2, 3);
		assert_eq!(Aabb2i::new(start, end), Aabb2i::new(end, start));
		assert!(Aabb2i::new(start, end).contains_point(start));
		assert!(Aabb2i::new(end, start).contains_point(start));
		assert!(!Aabb2i::new(start, end).contains_point(end));
		assert!(!Aabb2i::new(end, start).contains_point(end));
	}

	#[test]
	fn is_empty() {
		assert!(!Aabb2i::new(Vec2i::splat(0), Vec2i::splat(1)).is_empty());
		assert!(Aabb2i::new(Vec2i::splat(0), Vec2i::splat(0)).is_empty());
		assert!(Aabb2i::default().is_empty());
	}

	#[test]
	fn from_min_point() {
		assert_eq!(Aabb2i::from_min_point(Vec2i::splat(0), Vec2i::new(2, 3)).size(), Vec2i::new(2, 3));
		assert_eq!(Aabb2i::from_min_point(Vec2i::splat(0), Vec2i::new(2, -3)).size(), Vec2i::new(2, 3));
	}

	#[test]
	fn union() {
		let a = Aabb2i::from_min_point(Vec2i::zero(), Vec2i::splat(1));
		let b = Aabb2i::from_min_point(Vec2i::splat(2), Vec2i::splat(1));
		let aub = a.union(&b);
		let bua = b.union(&a);

		assert_eq!(aub, bua);
		assert_eq!(aub, Aabb2i::new(Vec2i::zero(), Vec2i::splat(3)));
	}
}