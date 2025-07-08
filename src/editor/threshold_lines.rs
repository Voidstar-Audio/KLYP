use super::Data;
use nih_plug::nih_dbg;
use nih_plug_vizia::{
    vizia::{prelude::*, vg},
    widgets::{
        param_base::ParamWidgetBase,
        util::{remap_current_entity_y_coordinate, ModifiersExt},
    },
};

pub struct ThresholdLines {
    dragging: bool,
    scrolled_lines: f32,
    softness_param_base: ParamWidgetBase,
    threshold_param_base: ParamWidgetBase,
}

impl View for ThresholdLines {
    /*fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left)
            | WindowEvent::MouseTripleClick(MouseButton::Left) => {
                let mouse_value = self.threshold_param_base.preview_normalized(
                    (1. - remap_current_entity_y_coordinate(cx, cx.mouse().cursory) * 2.0).abs(),
                );

                let actual_value = self.threshold_param_base.unmodulated_plain_value();

                if (actual_value - mouse_value).abs() > 0.15 {
                    return;
                }

                if cx.modifiers().command() {
                    // Ctrl+Click, double click, and right clicks should reset the parameter instead
                    // of initiating a drag operation
                    self.threshold_param_base.begin_set_parameter(cx);
                    self.threshold_param_base.set_normalized_value(
                        cx,
                        self.threshold_param_base.default_normalized_value(),
                    );
                    self.threshold_param_base.end_set_parameter(cx);
                } else {
                    // The `!self.text_input_active` check shouldn't be needed, but the textbox does
                    // not consume the mouse down event. So clicking on the textbox to move the
                    // cursor would also change the slider.
                    self.dragging = true;
                    cx.capture();
                    // NOTE: Otherwise we don't get key up events
                    cx.focus();
                    cx.set_active(true);

                    self.threshold_param_base.begin_set_parameter(cx);
                    // self.threshold_param_base.set_normalized_value(
                    //     cx,
                    //     1. - remap_current_entity_y_coordinate(cx, cx.mouse().cursory),
                    // );
                }
                meta.consume();
            }
            WindowEvent::MouseUp(MouseButton::Left) => {
                if self.dragging {
                    self.dragging = false;

                    cx.release();
                    cx.set_active(false);

                    self.threshold_param_base.end_set_parameter(cx);

                    meta.consume();
                }
            }
            WindowEvent::MouseMove(_, y) => {
                if self.dragging {
                    let value = self
                        .threshold_param_base
                        .preview_normalized((1. - remap_current_entity_y_coordinate(cx, *y) * 2.0).abs());
                    self.threshold_param_base.set_normalized_value(cx, value);
                }
            }
            WindowEvent::MouseScroll(_, scroll_y) => {
                // Loosely adapted from nih-plug's slider
                self.scrolled_lines += scroll_y;

                if self.scrolled_lines.abs() >= 1.0 {
                    let use_finer_steps = cx.modifiers().shift();

                    // Scrolling while dragging needs to be taken into account here
                    if !self.dragging {
                        self.softness_param_base.begin_set_parameter(cx);
                    }

                    let mut current_value = self.softness_param_base.unmodulated_normalized_value();

                    while self.scrolled_lines >= 1.0 {
                        current_value = self
                            .softness_param_base
                            .next_normalized_step(current_value, use_finer_steps);
                        self.softness_param_base
                            .set_normalized_value(cx, current_value);
                        self.scrolled_lines -= 1.0;
                    }

                    while self.scrolled_lines <= -1.0 {
                        current_value = self
                            .softness_param_base
                            .previous_normalized_step(current_value, use_finer_steps);
                        self.softness_param_base
                            .set_normalized_value(cx, current_value);
                        self.scrolled_lines += 1.0;
                    }

                    if !self.dragging {
                        self.softness_param_base.end_set_parameter(cx);
                    }
                }
            }
            _ => {}
        });
    }*/
}

impl ThresholdLines {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {
            dragging: false,
            softness_param_base: ParamWidgetBase::new(cx, Data::params, |params| &params.softness),
            threshold_param_base: ParamWidgetBase::new(cx, Data::params, |params| {
                &params.threshold
            }),
            scrolled_lines: 0.0,
        }
        .build(cx, |cx| {
            ParamWidgetBase::view(
                cx,
                Data::params,
                |params| &params.threshold,
                |cx, threshold| {
                    ParamWidgetBase::view(
                        cx,
                        Data::params,
                        |params| &params.softness,
                        |cx, softness| {
                            Lines {
                                threshold: threshold.make_lens(|p| p.value()),
                                softness: softness.make_lens(|p| p.value()),
                            }
                            .build(cx, |_| {});
                        },
                    )
                },
            );
        })
    }
}

struct Lines<T, S>
where
    T: Lens<Target = f32>,
    S: Lens<Target = f32>,
{
    threshold: T,
    softness: S,
}

impl<T, S> View for Lines<T, S>
where
    T: Lens<Target = f32>,
    S: Lens<Target = f32>,
{
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let threshold = self.threshold.get(cx);
        let softness = self.softness.get(cx);

        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let line_width = cx.scale_factor();

        let color = vg::Color::rgb(208, 10, 10);
        let color_25 = vg::Color { a: 0.25, ..color };

        let top_a = y + (1.0 - threshold) * h / 2.0;
        let top_b = y + (1.0 + threshold) * h / 2.0;

        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();

                path.move_to(x, top_a);
                path.line_to(x + w, top_a);

                path
            },
            &vg::Paint::color(color).with_line_width(line_width),
        );

        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();

                path.move_to(x, top_b);
                path.line_to(x + w, top_b);

                path
            },
            &vg::Paint::color(color).with_line_width(line_width),
        );

        let bottom_a = y + (1.0 - threshold * (1.0 - softness)) * h / 2.0;
        let bottom_b = y + (1.0 + threshold * (1.0 - softness)) * h / 2.0;

        if bottom_a - top_a >= 1.0 {
            canvas.fill_path(
                &{
                    let mut path = vg::Path::new();

                    path.move_to(x, top_a);
                    path.line_to(x + w, top_a);
                    path.line_to(x + w, bottom_a);
                    path.line_to(x, bottom_a);

                    path
                },
                &vg::Paint::color(color_25),
            );
            canvas.fill_path(
                &{
                    let mut path = vg::Path::new();

                    path.move_to(x, top_b);
                    path.line_to(x + w, top_b);
                    path.line_to(x + w, bottom_b);
                    path.line_to(x, bottom_b);

                    path
                },
                &vg::Paint::color(color_25),
            );
        }
    }
}
