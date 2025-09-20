use core::slice;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use crate::editor::RangePreset;
use crate::transfer;

use super::Data;
use cyma::accumulators::{Accumulator, PeakAccumulator};
use cyma::bus::{Bus, MonoBus};
use nih_plug_vizia::vizia::{prelude::*, vg};
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;
use nih_plug_vizia::widgets::util::{remap_current_entity_y_coordinate, ModifiersExt};

#[derive(Lens)]
pub struct ClippingCurve<R: Lens<Target = RangePreset>> {
    dragging: bool,
    bold: Arc<AtomicBool>,
    scrolled_lines: f32,
    softness_param_base: ParamWidgetBase,
    threshold_param_base: ParamWidgetBase,
    range: R,
}

fn remap<R: Lens<Target = RangePreset>>(cx: &EventContext, y_coord: f32, range: R) -> f32 {
    let size = range.get(cx).raw_scalar();
    1. - remap_current_entity_y_coordinate(cx, y_coord) * size + (size - 1.0)
}

impl<R: Lens<Target = RangePreset>> View for ClippingCurve<R> {
    fn element(&self) -> Option<&'static str> {
        Some("22-clipping-curve")
    }
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left)
            | WindowEvent::MouseTripleClick(MouseButton::Left) => {
                let mouse_value = remap(cx, cx.mouse().cursory, self.range);
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
                    //     1. - remap(cx, cx.mouse().cursory),
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
                self.bold.store(false, std::sync::atomic::Ordering::Relaxed);
            }
            WindowEvent::MouseLeave | WindowEvent::MouseOut => {
                self.bold.store(false, std::sync::atomic::Ordering::Relaxed);
            }
            WindowEvent::MouseMove(_, y) => {
                if self.dragging {
                    let value = self
                        .threshold_param_base
                        .preview_normalized(remap(cx, *y, self.range));
                    self.threshold_param_base.set_normalized_value(cx, value);
                } else {
                    let mouse_value = remap(cx, *y, self.range);
                    let actual_value = self.threshold_param_base.unmodulated_plain_value();

                    self.bold.store(
                        (actual_value - mouse_value).abs() <= 0.15,
                        std::sync::atomic::Ordering::Relaxed,
                    );
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
    }
}

impl<R: Lens<Target = RangePreset>> ClippingCurve<R> {
    pub fn new(cx: &mut Context, bus: Arc<MonoBus>, decay: f32, range: R) -> Handle<Self> {
        let bold = Arc::new(AtomicBool::new(false));
        Self {
            bold: bold.clone(),
            dragging: false,
            softness_param_base: ParamWidgetBase::new(cx, Data::params, |params| &params.softness),
            threshold_param_base: ParamWidgetBase::new(cx, Data::params, |params| {
                &params.threshold
            }),
            scrolled_lines: 0.0,
            range,
        }
        .build(cx, move |cx| {
            let mut accumulator = PeakAccumulator::new(1.0, decay);
            accumulator.set_sample_rate(bus.sample_rate());
            accumulator.set_size(bus.sample_rate() as usize);

            let accumulator = Arc::new(Mutex::new(accumulator));
            let accumulator_c = accumulator.clone();

            let dispatcher_handle = bus.register_dispatcher(move |samples| {
                if let Ok(mut acc) = accumulator_c.lock() {
                    for sample in samples {
                        let _ = acc.accumulate(*sample);
                    }
                }
            });

            ParamWidgetBase::view(
                cx,
                Data::params,
                |params| &params.gain,
                |cx, gain| {
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
                                    InnerCurve {
                                        gain: gain.make_lens(|p| p.value()),
                                        threshold: threshold.make_lens(|p| p.value()),
                                        softness: softness.make_lens(|p| p.value()),
                                        range,
                                        bold: bold.clone(),
                                        bus,
                                        accumulator,
                                        dispatcher_handle,
                                    }
                                    .build(cx, |_| {});
                                },
                            )
                        },
                    )
                },
            );
        })
    }
}

struct InnerCurve<G, T, S, R, A>
where
    G: Lens<Target = f32>,
    T: Lens<Target = f32>,
    S: Lens<Target = f32>,
    R: Lens<Target = RangePreset>,
    A: Accumulator + 'static,
{
    gain: G,
    threshold: T,
    softness: S,
    range: R,
    bold: Arc<AtomicBool>,
    bus: Arc<MonoBus>,
    accumulator: Arc<Mutex<A>>,
    dispatcher_handle: Arc<dyn Fn(slice::Iter<f32>)>,
}

impl<G, T, S, R, A> View for InnerCurve<G, T, S, R, A>
where
    G: Lens<Target = f32>,
    T: Lens<Target = f32>,
    S: Lens<Target = f32>,
    R: Lens<Target = RangePreset>,
    A: Accumulator + 'static,
{
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        self.bus.update();

        let threshold = self.threshold.get(cx);
        let softness = self.softness.get(cx);
        let range = self.range.get(cx);

        let size = range.raw_scalar();

        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        let offset = (1.0 - 1.0 / size) * h;

        let line_width = cx.scale_factor();

        let padding_scaled = 12.0 * cx.scale_factor();

        // Background
        canvas.fill_path(
            &{
                let mut path = vg::Path::new();
                path.move_to(x, y);
                path.line_to(x + w, y);
                path.line_to(x + w, y + h);
                path.line_to(x, y + h);
                path.close();
                path
            },
            &vg::Paint::color(vg::Color::rgb(219, 221, 229)).with_line_width(line_width),
        );

        // Dots
        canvas.fill_path(
            &{
                let mut path = vg::Path::new();

                (0..=8)
                    .flat_map(move |a| (0..=8).map(move |b| (a, b)))
                    .for_each(|(dx, dy)| {
                        path.circle(x + dx as f32 / 8.0 * w, y + dy as f32 / 8.0 * w, line_width)
                    });
                path
            },
            &vg::Paint::color(vg::Color::rgb(138, 141, 150)),
        );

        let h = h / size;
        let w_scaled = w / size;

        // Clipping Curve

        let in_peak = self.accumulator.lock().unwrap().prev();
        let limit = ((w_scaled.ceil() + padding_scaled) * in_peak) as u32;

        let mut clipping_curve = vg::Path::new();

        clipping_curve.move_to(x, y + h + offset);

        (0..limit).for_each(|i| {
            clipping_curve.line_to(
                x + i as f32,
                y + h * (1.0 - transfer(i as f32 / w_scaled, threshold, softness)) + offset,
            )
        });

        let filled_portion = clipping_curve.clone();
        (limit..(w_scaled.ceil() + padding_scaled) as u32).for_each(|i| {
            clipping_curve.line_to(
                x + i as f32,
                y + h * (1.0 - transfer(i as f32 / w_scaled, threshold, softness)) + offset,
            )
        });

        canvas.stroke_path(
            &clipping_curve,
            &vg::Paint::color(vg::Color::rgb(192, 195, 204)).with_line_width(line_width),
        );
        canvas.stroke_path(
            &filled_portion,
            &vg::Paint::color(vg::Color::rgb(0, 0, 0)).with_line_width(line_width),
        );

        let red = vg::Color::rgb(208, 10, 10);

        let top = y + (1.0 - threshold) * h + offset;
        let bottom = y + (1.0 - threshold * (1.0 - softness)) * h + offset;

        let bold = self.bold.load(std::sync::atomic::Ordering::Relaxed);

        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();

                path.move_to(x - padding_scaled, top);
                path.line_to(x + w + padding_scaled, top);

                path
            },
            &vg::Paint::color(red).with_line_width(line_width * if bold { 2.0 } else { 1.0 }),
        );

        if bottom - top >= 1.0 {
            canvas.fill_path(
                &{
                    let mut path = vg::Path::new();

                    path.move_to(x - padding_scaled, top);
                    path.line_to(x + w + padding_scaled, top);
                    path.line_to(x + w + padding_scaled, bottom);
                    path.line_to(x - padding_scaled, bottom);

                    path
                },
                &vg::Paint::color(vg::Color { a: 0.25, ..red }),
            );
        }
    }
}
