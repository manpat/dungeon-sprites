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
			ui_selection_preview(ui, atlas, &sprite_editor_state);

			{
				let mut editable_color = sprite_editor_state.preview_background.to_vec4().to_array();
				if imgui::ColorEdit::new("Preview BG Color", &mut editable_color)
					.inputs(false)
					.build(ui)
				{
					sprite_editor_state.preview_background = Color::from(editable_color);
				}
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


pub fn ui_atlas_editor(ui: &imgui::Ui<'_>, atlas: gfx::TextureKey, state: &mut SpriteEditorState) {
	let available_size = ui.content_region_avail();
	let widget_size = available_size[0].min(available_size[1]);
	let widget_pos = Vec2::from(ui.cursor_screen_pos());
	let widget_end = widget_pos + Vec2::splat(widget_size);

	ui.invisible_button("Atlas Editor", [widget_size, widget_size]);

	let draw_list = ui.get_window_draw_list();

	// Background
	draw_list.add_rect(widget_pos.to_array(), widget_end.to_array(), state.preview_background.to_tuple())
		.filled(true)
		.build();

	// Draw atlas
	let texture_id = toybox::imgui_backend::texture_key_to_imgui_id(atlas);
	draw_list.add_image(texture_id, widget_pos.to_array(), widget_end.to_array())
		.uv_min([0.0, 1.0])
		.uv_max([1.0, 0.0])
		.build();

	// Draw selection
	{
		let cell_tl = state.selected_cell_begin.to_vec2() / 8.0 * widget_size + widget_pos;
		let cell_br = state.selected_cell_end.to_vec2() / 8.0 * widget_size + widget_pos;

		draw_list.add_rect(cell_tl.to_array(), cell_br.to_array(), 0xff4444ff).build();
	}

	if !ui.is_item_hovered() {
		return;
	}

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

	draw_list.add_text(widget_pos.to_array(), 0xffff00ff, &format!("{mouse_pos:?} {widget_pos:?} {hovered_cell:?}"));

	draw_list.add_rect(cell_tl.to_array(), cell_br.to_array(), 0x88ffffff).build();
}



pub fn ui_selection_preview(ui: &imgui::Ui<'_>, atlas: gfx::TextureKey, state: &SpriteEditorState) {
	let widget_size = 128.0;
	let widget_pos = Vec2::from(ui.cursor_screen_pos());
	let widget_end = widget_pos + Vec2::splat(widget_size);

	ui.invisible_button("Sprite preview", [widget_size, widget_size]);

	let draw_list = ui.get_window_draw_list();

	// Background
	draw_list.add_rect(widget_pos.to_array(), widget_end.to_array(), state.preview_background.to_tuple())
		.filled(true)
		.build();

	let cell_tl = state.selected_cell_begin.to_vec2() / 8.0;
	let cell_br = state.selected_cell_end.to_vec2() / 8.0;

	fn flip_y(uv: Vec2) -> Vec2 {
		Vec2 {
			y: 1.0 - uv.y,
			.. uv
		}
	}

	// Draw preview
	let texture_id = toybox::imgui_backend::texture_key_to_imgui_id(atlas);
	draw_list.add_image(texture_id, widget_pos.to_array(), widget_end.to_array())
		.uv_min(flip_y(cell_tl).to_array())
		.uv_max(flip_y(cell_br).to_array())
		.build();
}