use common::*;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Project {
	pub scenes: Vec<Scene>,
	pub entities: Vec<Entity>,
	pub meshes: Vec<Mesh>,
}

#[derive(Debug, Clone)]
pub struct Scene {
	pub name: String,
	pub entities: Vec<u32>
}

#[derive(Debug, Clone)]
pub struct Entity {
	pub name: String,
	pub mesh_id: u16,

	pub position: Vec3,
	pub rotation: Quat,
	pub scale: Vec3,
}

#[derive(Debug, Clone)]
pub struct Mesh {
	pub positions: Vec<Vec3>,
	pub indices: Vec<u16>,
	pub color_layers: Vec<MeshColorLayer>,
	pub uv_layers: Vec<MeshUvLayer>,
	pub animation_data: Option<MeshAnimationData>,
}

#[derive(Debug, Clone)]
pub struct MeshColorLayer {
	pub name: String,
	pub data: Vec<Vec4>,
}

#[derive(Debug, Clone)]
pub struct MeshUvLayer {
	pub name: String,
	pub data: Vec<Vec2>,
}



#[derive(Debug, Clone)]
pub struct MeshAnimationData {
	pub bones: Vec<MeshBone>,
	pub weights: Vec<MeshWeightVertex>,
	pub animations: Vec<MeshAnimation>,
}

#[derive(Debug, Clone)]
pub struct MeshBone {
	pub name: String,
	pub head: Vec3,
	pub tail: Vec3,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MeshWeightVertex {
	pub indices: [u8; 3],
	pub weights: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct MeshAnimation {
	pub name: String,
	pub fps: f32,
	pub channels: Vec<MeshAnimationChannel>,
}

#[derive(Debug, Clone)]
pub struct MeshAnimationChannel {
	pub bone: String, // TODO: should be an index
	pub frames: Vec<MeshAnimationFrame>,
}

#[derive(Debug, Clone, Copy)]
pub struct MeshAnimationFrame {
	pub position: Vec3,
	pub rotation: Quat,
	pub scale: Vec3,
}



#[derive(Debug, Clone, Copy)]
pub struct SceneRef<'toy> {
	file: &'toy Project,
	scene: &'toy Scene,
}

#[derive(Debug, Clone, Copy)]
pub struct EntityRef<'toy> {
	file: &'toy Project,
	entity: &'toy Entity,
}

impl Project {
	pub fn find_scene(&self, name: &str) -> Option<SceneRef<'_>> {
		self.scenes.iter()
			.find(|e| e.name == name)
			.map(|scene| SceneRef::from(self, scene))
	}

	pub fn find_entity(&self, name: &str) -> Option<EntityRef<'_>> {
		self.entities.iter()
			.find(|e| e.name == name)
			.map(|entity| EntityRef::from(self, entity))
	}

	pub fn scenes(&self) -> impl Iterator<Item=SceneRef<'_>> {
		self.scenes.iter()
			.map(move |entity| SceneRef::from(self, entity))
	}

	pub fn entities(&self) -> impl Iterator<Item=EntityRef<'_>> {
		self.entities.iter()
			.map(move |entity| EntityRef::from(self, entity))
	}

	pub fn entities_with_prefix<'t, 'p: 't>(&'t self, prefix: &'p str) -> impl Iterator<Item=EntityRef<'t>> {
		self.entities()
			.filter(move |entity| entity.name.starts_with(prefix))
	}
}

impl Mesh {
	pub fn color_layer_by_name<'s>(&self, name: &'s str) -> Option<&MeshColorLayer> {
		self.color_layers.iter()
			.find(|l| l.name == name)
	}

	pub fn uv_layer_by_name<'s>(&self, name: &'s str) -> Option<&MeshUvLayer> {
		self.uv_layers.iter()
			.find(|l| l.name == name)
	}
}

impl<'t> SceneRef<'t> {
	pub fn from(file: &'t Project, scene: &'t Scene) -> SceneRef<'t> {
		SceneRef { file, scene }
	}

	pub fn entities(&self) -> impl Iterator<Item=EntityRef<'t>> {
		let file = self.file;

		self.scene.entities.iter()
			.map(move |&id| &file.entities[id as usize - 1])
			.map(move |entity| EntityRef::from(file, entity))
	}

	pub fn entities_with_prefix<'p: 't>(&self, prefix: &'p str) -> impl Iterator<Item=EntityRef<'t>> {
		self.entities()
			.filter(move |entity| entity.name.starts_with(prefix))
	}

	pub fn find_entity(&self, name: &str) -> Option<EntityRef<'t>> {
		self.entities().find(|ent| ent.entity.name == name)
	}
}

impl Deref for SceneRef<'_> {
	type Target = Scene;
	fn deref(&self) -> &Self::Target { self.scene }
}

impl<'t> EntityRef<'t> {
	pub fn from(file: &'t Project, entity: &'t Entity) -> EntityRef<'t> {
		EntityRef { file, entity }
	}

	pub fn mesh(&self) -> Option<&'t Mesh> {
		let mesh_id = self.entity.mesh_id;

		if mesh_id == 0 {
			return None
		}

		self.file.meshes.get(mesh_id as usize - 1)
	}
}

impl Entity {
	pub fn transform(&self) -> Mat3x4 {
		Mat3x4::translate(self.position)
			* self.rotation.to_mat3x4()
			* Mat3x4::scale(self.scale)
	}
}

impl Deref for EntityRef<'_> {
	type Target = Entity;
	fn deref(&self) -> &Self::Target { self.entity }
}

// TODO: entity queries
// TODO: mesh building