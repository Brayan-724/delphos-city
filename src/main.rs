use delphos_math::{IVec2, UVec2};
use delphos_window::sctk::{Region, WaylandSurface};
use delphos_window::wayland::WlFormat;
use delphos_window::{
    DelphosWindowApp, DelphosWindowDraw, DelphosWindowKeyboard, DelphosWindowPointer, OpenWindow,
    Time, sctk,
};
use image::RgbaImage;

fn main() {
    env_logger::init();

    delphos_window::start_window::<DelphosApp>();
}

struct DelphosApp {
    size: UVec2,
    person: RgbaImage,
}

impl DelphosWindowApp for DelphosApp {
    fn setup(_: IVec2, size: IVec2, output: delphos_window::wayland::WlOutput) -> OpenWindow {
        OpenWindow::builder()
            .anchor(sctk::Anchor::TOP)
            .namespace("delphos-city")
            .output(output)
            .size(size.as_u32().set_y(100))
            .build()
    }

    type NewError = ();
    fn new(window: &mut delphos_window::DelphosWindowState) -> Result<Self, Self::NewError> {
        let empty_region = Region::new(&window.compositor).unwrap();

        window
            .layer_surface
            .set_input_region(Some(empty_region.wl_region()));
        window.layer_surface.wl_surface().commit();

        Ok(Self {
            size: UVec2::default(),
            person: asefile::AsepriteFile::read(&include_bytes!("../assets/person.aseprite")[..])
                .unwrap()
                .frame(0)
                .image(),
        })
    }

    fn configure(
        &mut self,
        _: &mut delphos_window::DelphosWindowState,
        _: &delphos_window::ConfigureCtx<'_, Self>,
        configure: sctk::LayerSurfaceConfigure,
        _: u32,
    ) {
        self.size = UVec2::from(configure.new_size);
    }
}

impl DelphosWindowKeyboard for DelphosApp {
    fn press_key(
        &mut self,
        window: &mut delphos_window::DelphosWindowState,
        _: delphos_window::KeyboardCtx<'_, Self>,
        event: sctk::KeyEvent,
    ) {
        if event.keysym == sctk::Keysym::Escape {
            window.exit = true;
        }
    }
}

impl DelphosWindowPointer for DelphosApp {}

impl DelphosWindowDraw for DelphosApp {
    fn draw(
        &mut self,
        window: &mut delphos_window::DelphosWindowState,
        ctx: delphos_window::DrawCtx<'_, Self>,
    ) {
        let time = window.world.resource::<Time>().read();

        let width = self.size.x;
        let height = self.size.y;
        let stride = width as i32 * 4;

        let (buffer, canvas) = window
            .pool
            .create_buffer(width as i32, height as i32, stride, WlFormat::Argb8888)
            .expect("create buffer");

        // Draw to the window:
        {
            let shift = time.elapsed / 3;
            canvas
                .chunks_exact_mut(4)
                .enumerate()
                .for_each(|(index, chunk)| {
                    let x = ((index + shift as usize) % width as usize) as u32;
                    let y = (index / width as usize) as u32;

                    let [r, g, b, a] = if let Some(pixel) =
                        self.person.get_pixel_checked(x / 4, y / 4)
                        && !matches!(pixel.0, [_, _, _, 0])
                    {
                        pixel.0
                    } else {
                        let a = 0x20;
                        let r =
                            u32::min(((width - x) * 0xFF) / width, ((height - y) * 0xFF) / height);
                        let g = u32::min((x * 0xFF) / width, ((height - y) * 0xFF) / height);
                        let b = u32::min(((width - x) * 0xFF) / width, (y * 0xFF) / height);

                        [r as u8, g as u8, b as u8, a as u8]
                    };

                    let r = r as u32;
                    let g = g as u32;
                    let b = b as u32;
                    let a = a as u32;

                    let color = (a << 24) + (r << 16) + (g << 8) + b;

                    let array: &mut [u8; 4] = chunk.try_into().unwrap();
                    *array = color.to_le_bytes();
                });
        }

        // Damage the entire window
        window
            .layer_surface
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        // Request our next frame
        window
            .layer_surface
            .wl_surface()
            .frame(ctx.qh, window.layer_surface.wl_surface().clone());

        // Attach and commit to present.
        buffer
            .attach_to(window.layer_surface.wl_surface())
            .expect("buffer attach");
        window.layer_surface.commit();
    }
}
