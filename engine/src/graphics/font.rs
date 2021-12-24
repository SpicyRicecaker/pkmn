use super::State;

use wgpu_glyph::{
    ab_glyph::{self, FontArc},
    GlyphBrush, GlyphBrushBuilder, Section, Text,
};

pub struct FontInterface {
    staging_belt: wgpu::util::StagingBelt,
    glyph_brush: GlyphBrush<()>,
}

impl FontInterface {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // Default font, let's use visitor
        let visitor = ab_glyph::FontArc::try_from_slice(include_bytes!(
            "..\\..\\resources\\visitor2.ttf"
        ))
        .unwrap();
        let glyph_brush = GlyphBrushBuilder::using_font(visitor).build(device, format);
        let staging_belt = wgpu::util::StagingBelt::new(1024);

        Self {
            glyph_brush,
            staging_belt,
        }
    }
    pub fn add_font(&mut self, font: FontArc) {
        self.glyph_brush.add_font(font);
    }
    pub fn finish(&mut self) {
        self.staging_belt.finish()
    }
    #[inline]
    pub fn queue(&mut self, section: Section) {
        self.glyph_brush.queue(section)
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        size: winit::dpi::PhysicalSize<u32>,
        frame: &wgpu::TextureView,
    ) {
        self.glyph_brush
            .draw_queued(
                device,
                &mut self.staging_belt,
                encoder,
                frame,
                size.width,
                size.height,
            )
            .expect("Draw queued");
    }
}

impl State {
    pub fn load_font(&mut self, path: &str) -> Result<(), std::io::Error> {
        let buffer = std::fs::read(path)?;
        let font = ab_glyph::FontArc::try_from_vec(buffer).unwrap();
        self.font_interface.add_font(font);

        Ok(())
    }
    #[inline]
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, color: wgpu::Color, scale: f32) {
        self.font_interface.queue(Section {
            screen_position: (x, y),
            text: vec![Text::new(text)
                .with_color([
                    color.r as f32,
                    color.g as f32,
                    color.b as f32,
                    color.a as f32,
                ])
                .with_scale(scale)],
            ..Section::default()
        });
    }
}
