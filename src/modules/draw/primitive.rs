use super::DrawCommand;
use crate::modules::{GameState, WindowOptions};
use luminance::blending::{Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::depth_test::DepthComparison;
use luminance::pipeline::ShadingGate;
use luminance::render_state::RenderState;
use luminance::shader::program::{Program, Uniform};
use luminance::tess::Tess;
use luminance_derive::UniformInterface;
use nalgebra::Vector2;

#[derive(UniformInterface)]
pub struct PrimitiveShaderInterface {
    pub screen_size: Uniform<[f32; 2]>,
    pub hidpi_factor: Uniform<f32>,
    pub image_size: Uniform<[f32; 2]>,
    pub subimage_offset: Uniform<[f32; 2]>,
    pub subimage_size: Uniform<[f32; 2]>,

    pub depth: Uniform<f32>,
    pub origin: Uniform<[f32; 2]>,
    pub position: Uniform<[f32; 2]>,
    pub rotation: Uniform<f32>,
    pub scale: Uniform<[f32; 2]>,

    pub shape_id: Uniform<i32>,
    pub thickness: Uniform<f32>,
    pub color: Uniform<[f32; 4]>,
    pub radius: Uniform<f32>,

    pub arc_from: Uniform<f32>,
    pub arc_to: Uniform<f32>,

    pub line_from: Uniform<[f32; 2]>,
    pub line_to: Uniform<[f32; 2]>,
}

pub struct PrimitiveProgram<'a> {
    pub program: &'a Program<(), (), PrimitiveShaderInterface>,
    pub tess: &'a Tess,
}

impl<'a> PrimitiveProgram<'a> {
    pub fn render<C, G>(
        &mut self,
        shading_gate: &mut ShadingGate<C>,
        game_state: &G,
        commands: &Vec<DrawCommand>,
    ) where
        C: GraphicsContext,
        G: GameState,
    {
        let WindowOptions { width, height, .. } = game_state.window_options();
        let render_state: RenderState = Default::default();
        let render_state = render_state
            .set_blending(Some((
                Equation::Additive,
                Factor::SrcAlpha,
                Factor::SrcAlphaComplement,
            )))
            .set_depth_test(Some(DepthComparison::Always))
            .set_face_culling(None);

        shading_gate.shade(&self.program, |interface, mut render_gate| {
            interface.screen_size.update([width as f32, height as f32]);
            interface.hidpi_factor.update(2.0);
            interface.subimage_offset.update([0.0, 0.0]);

            interface.scale.update([1.0, 1.0]);

            commands.iter().for_each(|command| {
                interface.shape_id.update(match &command {
                    DrawCommand::Arc { .. } => 0,
                    DrawCommand::Circle { .. } => 1,
                    DrawCommand::Line { .. } => 2,
                    DrawCommand::Rectangle { .. } => 3,
                    DrawCommand::Sprite { .. } => 4,
                    DrawCommand::Text { .. } => 5,
                });
                interface.origin.update([0.5, 0.5]);

                let should_render = match command {
                    DrawCommand::Arc {
                        depth,
                        color,
                        radius,
                        thickness,
                        from,
                        to,
                        position,
                    } => {
                        interface.depth.update(*depth);
                        interface.image_size.update([2.0 * *radius, 2.0 * *radius]);
                        interface
                            .subimage_size
                            .update([2.0 * *radius, 2.0 * *radius]);

                        interface.position.update((*position).into());
                        interface.rotation.update(0.0);

                        interface.color.update(*color);
                        interface.radius.update(*radius);
                        interface.thickness.update(*thickness);
                        interface.arc_from.update(*from);
                        interface.arc_to.update(*to);
                        true
                    }
                    DrawCommand::Circle {
                        depth,
                        color,
                        radius,
                        position,
                    } => {
                        interface.depth.update(*depth);
                        interface.image_size.update([2.0 * *radius, 2.0 * *radius]);
                        interface
                            .subimage_size
                            .update([2.0 * *radius, 2.0 * *radius]);

                        interface.position.update((*position).into());
                        interface.rotation.update(0.0);

                        interface.color.update(*color);
                        interface.radius.update(*radius);
                        true
                    }
                    DrawCommand::Line {
                        depth,
                        color,
                        thickness,
                        from,
                        to,
                    } => {
                        interface.depth.update(*depth);
                        let min = Vector2::new(from.x.min(to.x), from.y.min(to.y))
                            - Vector2::new(*thickness, *thickness);
                        let max = Vector2::new(from.x.max(to.x), from.y.max(to.y))
                            + Vector2::new(*thickness, *thickness);
                        let dimensions = max - min;

                        interface.image_size.update(dimensions.into());
                        interface.subimage_size.update(dimensions.into());
                        interface.position.update(min.into());
                        interface.origin.update([0.0, 0.0]);
                        interface.rotation.update(0.0);
                        interface.color.update(*color);
                        interface.thickness.update(*thickness);
                        interface.line_from.update((from - min).into());
                        interface.line_to.update((to - min).into());
                        true
                    }
                    DrawCommand::Rectangle {
                        depth,
                        color,
                        width,
                        height,
                        position,
                        rotation,
                    } => {
                        interface.depth.update(*depth);
                        interface.image_size.update([*width, *height]);
                        interface.subimage_size.update([*width, *height]);

                        interface.position.update((*position).into());
                        interface.rotation.update(*rotation);

                        interface.color.update(*color);
                        true
                    }
                    DrawCommand::Sprite { .. } => false,
                    DrawCommand::Text { .. } => false,
                };

                if should_render {
                    render_gate.render(&render_state, |mut tess_gate| {
                        tess_gate.render(self.tess);
                    });
                }
            });
        });
    }
}
