#![feature(let_chains)]

use toybox::prelude::*;
use serde::{Serialize, Deserialize};


fn main() -> Result<(), Box<dyn std::error::Error>> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let mut engine = toybox::Engine::new("dungeon-sprites")?;

	let mut gfx = engine.gfx.resource_context(None);
	let atlas = load_texture(&mut gfx, "assets/atlas.png")?;
	let shader = gfx.new_simple_shader(
		include_str!("shaders/tex_3d.vert.glsl"),
		include_str!("shaders/textured.frag.glsl"),
	)?;

	let vao = gfx.new_vao();

	let mut sprite_editor_state = SpriteEditorState::default();
	sprite_editor_state.preview_background = Color::black();
	
	engine.imgui.set_input_enabled(true);
	engine.imgui.set_visible(true);

	'main: loop {
		engine.process_events();
		if engine.should_quit() {
			break 'main
		}

		let ui = engine.imgui.frame();

		ui_atlas_editor(ui, atlas, &mut sprite_editor_state);

		ui.same_line();

		ui.group(|| {
			ui_sprite_basic_preview(ui, atlas, &sprite_editor_state);

			let mut editable_color = sprite_editor_state.preview_background.to_vec4().to_array();
			if imgui::ColorEdit::new("Preview BG Color", &mut editable_color)
				.inputs(false)
				.build(ui)
			{
				sprite_editor_state.preview_background = Color::from(editable_color);
			}
		});

		let mut gfx = engine.gfx.draw_context();

		gfx.set_clear_color(Color::grey(0.02));
		gfx.clear(gfx::ClearMode::ALL);

		gfx.bind_vao(vao);
		gfx.bind_shader(shader);
		gfx.bind_texture(0, atlas);

		gfx.draw_arrays(gfx::DrawMode::Triangles, 6);

		engine.end_frame();
	}

	Ok(())
}




use std::path::Path;

pub fn load_texture(gfx: &mut gfx::ResourceContext<'_>, path: impl AsRef<Path>) -> Result<gfx::TextureKey, Box<dyn std::error::Error>> {
	let image = image::open(path)?.flipv().into_rgba8().into_flat_samples();
	let image_size = Vec2i::new(image.layout.width as i32, image.layout.height as i32);
	let texture_format = gfx::TextureFormat::srgba();

	let texture = gfx.new_texture(image_size, texture_format);

	{
		let mut texture = gfx.resources.textures.get_mut(texture);
		texture.upload_rgba8_raw(&image.samples);
	}

	Ok(texture)
}



#[derive(Default)]
pub struct SpriteEditorState {
	selected_cells: Aabb2i,
	drag_start_cell: Vec2i,

	preview_background: Color,
}


#[derive(Default, Serialize, Deserialize)]
pub struct Sprite {
	name: String,

	pixel_start: Vec2i,
	pixel_end: Vec2i,
}


pub fn ui_atlas_editor(ui: &imgui::Ui<'_>, atlas: gfx::TextureKey, state: &mut SpriteEditorState) {
	let canvas = TextureCanvasBuilder::new(atlas)
		.build(ui);

	canvas.fill(state.preview_background);
	canvas.draw_texture();

	// Draw Selection
	canvas.draw_cell_rect(state.selected_cells, Color::rgb(1.0, 0.2, 0.2));

	let Some(hovered_cell) = canvas.hovered_cell() else {
		return
	};

	let hovered_cell_range = Aabb2i::from_min_point(hovered_cell, Vec2i::splat(1));

	if ui.is_item_clicked() {
		state.drag_start_cell = hovered_cell;
		state.selected_cells = hovered_cell_range;
	}

	if ui.is_mouse_dragging(imgui::MouseButton::Left) {
		let start_range = Aabb2i::from_min_point(state.drag_start_cell, Vec2i::splat(1));
		state.selected_cells = start_range.union(&hovered_cell_range);
	}

	// Draw hovered cell
	canvas.draw_cell_rect(hovered_cell_range, Color::grey_a(1.0, 0.5));
}


pub fn ui_sprite_basic_preview(ui: &imgui::Ui<'_>, atlas: gfx::TextureKey, state: &SpriteEditorState) {
	let canvas = TextureCanvasBuilder::new(atlas)
		.widget_size(Vec2::splat(300.0))
		.display_range(state.selected_cells.scale(16))
		.build(ui);

	canvas.fill(state.preview_background);
	if canvas.is_empty() {
		return;
	}

	canvas.draw_texture();
}




const TEX_SIZE: f32 = 128.0;


pub struct TextureCanvasBuilder {
	atlas: gfx::TextureKey,
	widget_size_px: Option<Vec2>,
	display_range: Option<Aabb2i>,
}

impl TextureCanvasBuilder {
	pub fn new(atlas: gfx::TextureKey) -> Self {
		TextureCanvasBuilder {
			atlas,
			widget_size_px: None,
			display_range: None,
		}
	}

	pub fn widget_size(mut self, size: Vec2) -> Self {
		self.widget_size_px = Some(size);
		self
	}

	pub fn display_range(mut self, range: Aabb2i) -> Self {
		self.display_range = Some(range);
		self
	}

	pub fn build<'imgui>(self, ui: &'imgui imgui::Ui<'_>) -> TextureCanvas<'imgui> {
		let uv_range = match self.display_range {
			Some(pixel_range) => pixel_range.to_aabb2().scale(1.0/TEX_SIZE),
			None => Aabb2::new(Vec2::zero(), Vec2::splat(1.0)),
		};

		// let aspect = ...

		let widget_size_px = match self.widget_size_px {
			Some(size) => size,
			None => {
				let [w, h] = ui.content_region_avail();
				Vec2::splat(w.min(h))
			}
		};

		let widget_start = Vec2::from(ui.cursor_screen_pos());
		let widget_end = widget_start + widget_size_px;

		ui.invisible_button("Texture canvas", widget_size_px.to_array());

		TextureCanvas {
			ui,
			draw_list: ui.get_window_draw_list(),

			atlas: self.atlas,

			widget_bounds: Aabb2::new(widget_start, widget_end),
			uv_range,
		}
	}
}



pub struct TextureCanvas<'imgui> {
	ui: &'imgui imgui::Ui<'imgui>,
	draw_list: imgui::draw_list::DrawListMut<'imgui>,

	atlas: gfx::TextureKey,

	widget_bounds: Aabb2,
	uv_range: Aabb2,
}

impl TextureCanvas<'_> {
	pub fn is_empty(&self) -> bool {
		let Vec2{x, y} = self.uv_range.size();
		x < 0.001 || y < 0.001
	}

	pub fn fill(&self, color: Color) {
		self.draw_list.add_rect(self.widget_bounds.min.to_array(), self.widget_bounds.max.to_array(), color.to_tuple())
			.filled(true)
			.build();
	}

	pub fn draw_texture(&self) {
		fn flip_y(uv: Vec2) -> Vec2 {
			Vec2 {
				y: 1.0 - uv.y,
				.. uv
			}
		}

		let texture_id = toybox::imgui_backend::texture_key_to_imgui_id(self.atlas);
		self.draw_list.add_image(texture_id, self.widget_bounds.min.to_array(), self.widget_bounds.max.to_array())
			.uv_min(flip_y(self.uv_range.min).to_array())
			.uv_max(flip_y(self.uv_range.max).to_array())
			.build();
	}

	pub fn draw_cell_rect(&self, cell_range: Aabb2i, color: impl Into<Color>) {
		self.draw_pixel_rect(cell_range.scale(16), color);
	}

	pub fn draw_pixel_rect(&self, pixel_range: Aabb2i, color: impl Into<Color>) {
		if pixel_range.is_empty() {
			return;
		}

		let color = color.into();

		let start_norm = pixel_range.min.to_vec2() / TEX_SIZE;
		let end_norm = pixel_range.max.to_vec2() / TEX_SIZE;

		let start_viewport = self.uv_range.map_to_percentage(start_norm);
		let end_viewport = self.uv_range.map_to_percentage(end_norm);

		let start_widget = self.widget_bounds.map_from_percentage(start_viewport);
		let end_widget = self.widget_bounds.map_from_percentage(end_viewport);

		self.draw_list.with_clip_rect_intersect(self.widget_bounds.min.to_array(), self.widget_bounds.max.to_array(), || {
			self.draw_list.add_rect(start_widget.to_array(), end_widget.to_array(), color.to_tuple()).build();
		});
	}

	pub fn hovered_pixel(&self) -> Option<Vec2i> {
		if !self.ui.is_item_hovered() {
			return None;
		}

		let mouse_pos = self.ui.io().mouse_pos;

		let hovered_pixel = self.widget_bounds.map_to_percentage(Vec2::from(mouse_pos)) * TEX_SIZE;
		Some(hovered_pixel.to_vec2i())
	}

	pub fn hovered_cell(&self) -> Option<Vec2i> {
		self.hovered_pixel()
			.map(|pos_px| pos_px / 16)
	}
}