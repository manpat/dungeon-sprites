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
			SpritePreview::new(atlas)
				.widget_size(Vec2::splat(128.0))
				.display_range(sprite_editor_state.selected_cell_begin, sprite_editor_state.selected_cell_end)
				.background_color(sprite_editor_state.preview_background)
				.build(ui);

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

	cell: Vec2i,
}


pub fn ui_atlas_editor(ui: &imgui::Ui<'_>, atlas: gfx::TextureKey, state: &mut SpriteEditorState) {
	let available_size = ui.content_region_avail();
	let widget_size = available_size[0].min(available_size[1]);
	let widget_pos = Vec2::from(ui.cursor_screen_pos());

	SpritePreview::new(atlas)
		.selection_range(state.selected_cell_begin, state.selected_cell_end)
		.background_color(state.preview_background)
		.build(ui);

	if !ui.is_item_hovered() {
		return;
	}

	let draw_list = ui.get_window_draw_list();

	let mouse_pos = ui.io().mouse_pos;
	draw_list.add_circle(mouse_pos, 5.0, 0xff44ff44).build();

	let diff = Vec2::from(mouse_pos) - widget_pos;
	let hovered_cell = (diff / widget_size * 8.0).to_vec2i();

	if ui.is_item_clicked() {
		state.selected_cell_begin = hovered_cell;
		state.selected_cell_end = hovered_cell + Vec2i::splat(1);
	}

	if ui.is_mouse_dragging(imgui::MouseButton::Left) {
		let Vec2i{x, y} = hovered_cell - state.selected_cell_begin + Vec2i::splat(1);
		let cell_delta = Vec2i::new(x.max(1), y.max(1));

		state.selected_cell_end = state.selected_cell_begin + cell_delta;
	}

	let hovered_cell = hovered_cell.to_vec2();
	let cell_tl = hovered_cell / 8.0 * widget_size + widget_pos;
	let cell_br = (hovered_cell + Vec2::splat(1.0)) / 8.0 * widget_size + widget_pos;

	draw_list.add_rect(cell_tl.to_array(), cell_br.to_array(), 0x88ffffff).build();
}



pub struct SpritePreview {
	atlas: gfx::TextureKey,

	widget_size: Option<Vec2>,

	display_range: Option<(Vec2i, Vec2i)>,
	selection_range: Option<(Vec2i, Vec2i)>,

	bg_color: Color,
}


impl SpritePreview {
	pub fn new(atlas: gfx::TextureKey) -> Self {
		SpritePreview {
			atlas,
			widget_size: None,
			display_range: None,
			selection_range: None,
			bg_color: Color::black(),
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

	pub fn selection_range(mut self, start: Vec2i, end: Vec2i) -> Self {
		self.selection_range = Some((start, end));
		self
	}

	pub fn background_color(mut self, color: Color) -> Self {
		self.bg_color = color;
		self
	}

	pub fn build(self, ui: &imgui::Ui<'_>) {
		let (display_tl, display_br) = match self.display_range {
			Some((start, end)) => (start.to_vec2() / 8.0, end.to_vec2() / 8.0),
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

		let widget_pos = Vec2::from(ui.cursor_screen_pos());
		let widget_end = widget_pos + widget_size;

		ui.invisible_button("Sprite preview", widget_size.to_array());

		let draw_list = ui.get_window_draw_list();

		// Background
		draw_list.add_rect(widget_pos.to_array(), widget_end.to_array(), self.bg_color.to_tuple())
			.filled(true)
			.build();


		let display_size = display_br - display_tl;
		if (display_size.x.abs() < 0.01) || (display_size.y.abs() < 0.01) {
			return;
		}


		fn flip_y(uv: Vec2) -> Vec2 {
			Vec2 {
				y: 1.0 - uv.y,
				.. uv
			}
		}

		// Draw preview
		let texture_id = toybox::imgui_backend::texture_key_to_imgui_id(self.atlas);
		draw_list.add_image(texture_id, widget_pos.to_array(), widget_end.to_array())
			.uv_min(flip_y(display_tl).to_array())
			.uv_max(flip_y(display_br).to_array())
			.build();

		// Draw selection
		if let Some((cell_tl, cell_br)) = self.selection_range
			&& cell_tl != cell_br
		{
			let norm_tl = cell_tl.to_vec2() / 8.0;
			let norm_br = cell_br.to_vec2() / 8.0;

			let norm_tl = (norm_tl - display_tl) / (display_br - display_tl);
			let norm_br = (norm_br - display_tl) / (display_br - display_tl);

			let widget_tl = norm_tl * widget_size + widget_pos;
			let widget_br = norm_br * widget_size + widget_pos;

			draw_list.with_clip_rect_intersect(widget_pos.to_array(), widget_end.to_array(), || {
				draw_list.add_rect(widget_tl.to_array(), widget_br.to_array(), 0xff4444ff).build();
			});
		}
	}
}
