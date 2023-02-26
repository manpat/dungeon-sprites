#![feature(let_chains)]

use toybox::prelude::*;

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
	selected_cell_begin: Vec2i,
	selected_cell_end: Vec2i,

	preview_background: Color,
}


#[derive(Default)]
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
	canvas.draw_cell_rect(state.selected_cell_begin, state.selected_cell_end, Color::rgb(1.0, 0.2, 0.2));

	let Some(hovered_cell) = canvas.hovered_cell() else {
		return
	};

	if ui.is_item_clicked() {
		state.selected_cell_begin = hovered_cell;
		state.selected_cell_end = hovered_cell + Vec2i::splat(1);
	}

	if ui.is_mouse_dragging(imgui::MouseButton::Left) {
		let Vec2i{x, y} = hovered_cell - state.selected_cell_begin + Vec2i::splat(1);
		let cell_delta = Vec2i::new(x.max(1), y.max(1));

		state.selected_cell_end = state.selected_cell_begin + cell_delta;
	}

	// Draw hovered cell
	canvas.draw_cell_rect(hovered_cell, hovered_cell + Vec2i::splat(1), Color::grey_a(1.0, 0.5));
}


pub fn ui_sprite_basic_preview(ui: &imgui::Ui<'_>, atlas: gfx::TextureKey, state: &SpriteEditorState) {
	let canvas = TextureCanvasBuilder::new(atlas)
		.widget_size(Vec2::splat(300.0))
		.display_range(state.selected_cell_begin * 16, state.selected_cell_end * 16)
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
	widget_size: Option<Vec2>,
	display_range: Option<(Vec2i, Vec2i)>,
}

impl TextureCanvasBuilder {
	pub fn new(atlas: gfx::TextureKey) -> Self {
		TextureCanvasBuilder {
			atlas,
			widget_size: None,
			display_range: None,
		}
	}

	pub fn widget_size(mut self, size: Vec2) -> Self {
		self.widget_size = Some(size);
		self
	}

	pub fn display_range(mut self, start: Vec2i, end: Vec2i) -> Self {
		self.display_range = Some((start, end));
		self
	}

	pub fn build<'imgui>(self, ui: &'imgui imgui::Ui<'_>) -> TextureCanvas<'imgui> {
		let (uv_start, uv_end) = match self.display_range {
			Some((start, end)) => (start.to_vec2() / TEX_SIZE, end.to_vec2() / TEX_SIZE),
			None => (Vec2::zero(), Vec2::splat(1.0)),
		};

		// let aspect = ...

		let widget_size = match self.widget_size {
			Some(size) => size,
			None => {
				let [w, h] = ui.content_region_avail();
				Vec2::splat(w.min(h))
			}
		};

		let widget_start = Vec2::from(ui.cursor_screen_pos());
		let widget_end = widget_start + widget_size;

		ui.invisible_button("Texture canvas", widget_size.to_array());

		TextureCanvas {
			ui,
			draw_list: ui.get_window_draw_list(),

			atlas: self.atlas,

			widget_start,
			widget_end,
			widget_size,

			uv_start,
			uv_end,
		}
	}
}



pub struct TextureCanvas<'imgui> {
	ui: &'imgui imgui::Ui<'imgui>,
	draw_list: imgui::draw_list::DrawListMut<'imgui>,

	atlas: gfx::TextureKey,

	widget_start: Vec2,
	widget_end: Vec2,
	widget_size: Vec2,
	
	uv_start: Vec2,
	uv_end: Vec2,
}

impl TextureCanvas<'_> {
	pub fn is_empty(&self) -> bool {
		let Vec2{x, y} = self.uv_end - self.uv_start;
		x < 0.001 || y < 0.001
	}

	pub fn fill(&self, color: Color) {
		self.draw_list.add_rect(self.widget_start.to_array(), self.widget_end.to_array(), color.to_tuple())
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
		self.draw_list.add_image(texture_id, self.widget_start.to_array(), self.widget_end.to_array())
			.uv_min(flip_y(self.uv_start).to_array())
			.uv_max(flip_y(self.uv_end).to_array())
			.build();
	}

	pub fn draw_cell_rect(&self, start_cell: Vec2i, end_cell: Vec2i, color: impl Into<Color>) {
		self.draw_pixel_rect(start_cell * 16, end_cell * 16, color);
	}

	pub fn draw_pixel_rect(&self, start_px: Vec2i, end_px: Vec2i, color: impl Into<Color>) {
		if start_px == end_px {
			return;
		}

		let color = color.into();

		let start_norm = start_px.to_vec2() / TEX_SIZE;
		let end_norm = end_px.to_vec2() / TEX_SIZE;

		let uv_size = self.uv_end - self.uv_start;
		let start_viewport = (start_norm - self.uv_start) / uv_size;
		let end_viewport = (end_norm - self.uv_start) / uv_size;

		let start_widget = start_viewport * self.widget_size + self.widget_start;
		let end_widget = end_viewport * self.widget_size + self.widget_start;

		self.draw_list.with_clip_rect_intersect(self.widget_start.to_array(), self.widget_end.to_array(), || {
			self.draw_list.add_rect(start_widget.to_array(), end_widget.to_array(), color.to_tuple()).build();
		});
	}

	pub fn hovered_pixel(&self) -> Option<Vec2i> {
		if !self.ui.is_item_hovered() {
			return None;
		}

		let mouse_pos = self.ui.io().mouse_pos;

		let diff = Vec2::from(mouse_pos) - self.widget_start;
		let hovered_pixel = (diff * TEX_SIZE / self.widget_size).to_vec2i();
		Some(hovered_pixel)
	}

	pub fn hovered_cell(&self) -> Option<Vec2i> {
		self.hovered_pixel()
			.map(|pos_px| pos_px / 16)
	}
}