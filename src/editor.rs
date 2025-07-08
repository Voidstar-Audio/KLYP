mod curve;
mod threshold_lines;

use astra::prelude::*;
use curve::ClippingCurve;
use cyma::prelude::*;
use nih_plug::params::Param;
use nih_plug::prelude::Editor;
use nih_plug::util::db_to_gain;
use nih_plug_vizia::vizia::icons::ICON_CHEVRON_DOWN;
use nih_plug_vizia::vizia::{image, prelude::*};
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;
use threshold_lines::ThresholdLines;

use crate::KlypParams;

#[derive(Lens)]
pub struct Data {
    params: Arc<KlypParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (640, 400))
}

pub(crate) fn create(
    params: Arc<KlypParams>,
    editor_state: Arc<ViziaState>,
    pre: Arc<MonoBus>,
    post: Arc<MonoBus>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::None, move |cx, _| {
        let _ = apply_styles(cx);
        cx.load_image(
            "logo.png",
            image::load_from_memory_with_format(
                include_bytes!("editor/logo.png"),
                image::ImageFormat::Png,
            )
            .unwrap(),
            ImageRetentionPolicy::DropWhenUnusedForOneFrame,
        );
        let _ = cx.add_stylesheet(
            r#"
                dropdown {
                    width: 32px;
                }
                dropdown > popup {
                    width: 188px;
                }
            "#,
        );

        Data {
            params: params.clone(),
        }
        .build(cx);

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                ZStack::new(cx, |cx| {
                    ClippingCurve::new(cx, pre.clone(), 300.0);
                    Dropdown::new(
                        cx,
                        |cx| {
                            HStack::new(cx, |cx| {
                                ParamWidgetBase::view(
                                    cx,
                                    Data::params,
                                    |p| &p.antialiasing.oversampling,
                                    |cx, o| {
                                        Label::new(
                                            cx,
                                            o.make_lens(|o| {
                                                o.normalized_value_to_string(
                                                    o.modulated_normalized_value(),
                                                    true,
                                                )
                                            }),
                                        )
                                        .width(Stretch(1.0))
                                        .pointer_events(false);
                                    },
                                );
                                Image::new(cx, "chevron_down.png").pointer_events(false);
                            })
                        },
                        |cx| {
                            VStack::new(cx, |cx| {
                                HStack::new(cx, |cx| {
                                    Label::new(cx, "OVERSAMPLING")
                                        .top(Stretch(1.0))
                                        .bottom(Stretch(1.0));
                                    Selector::new(cx, Data::params, |p| {
                                        &p.antialiasing.oversampling
                                    })
                                    .top(Stretch(1.0))
                                    .bottom(Stretch(1.0));
                                })
                                .height(Auto)
                                .col_between(Stretch(1.0));
                                ParamWidgetBase::view(
                                    cx,
                                    Data::params,
                                    |p| &p.antialiasing.oversampling,
                                    |cx, o| {
                                        HStack::new(cx, |cx| {
                                            Label::new(cx, "ANTIDERIVATIVE")
                                                .top(Stretch(1.0))
                                                .bottom(Stretch(1.0));
                                            ParamSwitch::new(cx, Data::params, |p| {
                                                &p.antialiasing.antiderivative
                                            })
                                            .top(Stretch(1.0))
                                            .bottom(Stretch(1.0));
                                        })
                                        .toggle_class(
                                            "disabled",
                                            o.make_lens(|o| o.unmodulated_plain_value() == 0),
                                        )
                                        .height(Auto)
                                        .col_between(Stretch(1.0));
                                    },
                                );
                            })
                            .child_space(Pixels(8.0))
                            .row_between(Pixels(4.0))
                            .height(Auto);
                        },
                    )
                    .left(Stretch(1.0))
                    .top(Stretch(1.0));
                })
                .child_space(Pixels(12.0))
                .size(Pixels(212.0));
                hdivider(cx);
                VStack::new(cx, |cx| {
                    ParamSlider::new(
                        cx,
                        Data::params,
                        |p| &p.gain,
                        (0..=16).map(|i| {
                            let pos = i as f32 / 16.0;
                            let value = -24 + (pos * 48.0) as i16;
                            let short = value % 6 != 0;
                            SliderTick {
                                pos,
                                label: (!short).then(|| format!("{:}", value)),
                                short,
                            }
                        }),
                    );
                    ParamSlider::new(
                        cx,
                        Data::params,
                        |p| &p.threshold,
                        [
                            (0.0, true),
                            (-1.5, false),
                            (-3.0, false),
                            (-4.5, false),
                            (-6.0, true),
                            (-7.5, false),
                            (-9.0, false),
                            (-10.5, false),
                            (-12.0, true),
                            (-13.5, false),
                            (-15.0, false),
                            (-16.5, false),
                            (-18.0, true),
                            (-19.5, false),
                            (-21.0, false),
                            (-22.5, false),
                            (-24.0, true),
                            (-27.0, false),
                            (-30.0, false),
                            (-33.0, false),
                            (-36.0, true),
                            (-48.0, false),
                            (-100.0, true),
                        ]
                        .iter()
                        .enumerate()
                        .map(|(i, (x, text))| {
                            let pos = params.threshold.preview_normalized(db_to_gain(*x));
                            let short = i % 2 != 0;

                            SliderTick {
                                pos,
                                label: (text).then_some(format!("{:.0}", x)),
                                short,
                            }
                        }),
                    );
                    ParamSlider::new(
                        cx,
                        Data::params,
                        |params| &params.softness,
                        (0..=20).map(|x| {
                            let pos = x as f32 / 20.0;
                            let short = x % 5 != 0;

                            SliderTick {
                                pos,
                                label: (!short).then_some(format!("{:.0}", pos * 100.0)),
                                short,
                            }
                        }),
                    );
                })
                .child_space(Pixels(12.0))
                .row_between(Pixels(8.0))
                .height(Auto);
                hdivider(cx);
                HStack::new(cx, |cx| {
                    Image::new(cx, "voidstar_logo.png").size(Pixels(24.0));
                    Image::new(cx, "logo.png")
                        .height(Pixels(12.0))
                        .width(Pixels(40.0));
                    Label::new(cx, env!("CARGO_PKG_VERSION"))
                        .width(Stretch(1.0))
                        .text_align(TextAlign::Right);
                })
                .height(Stretch(1.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .col_between(Pixels(6.0))
                .child_left(Pixels(12.0))
                .child_right(Pixels(12.0));
            })
            .width(Auto);
            vdivider(cx);
            ZStack::new(cx, |cx| {
                let range = 400.0 / (400.0 - 24.0);
                const DURATION: f32 = 5.0;
                const TICKS: usize = 2;
                Oscilloscope::new(
                    cx,
                    post.clone(),
                    DURATION,
                    (-range, range),
                    ValueScaling::Linear,
                );
                Oscilloscope::new(
                    cx,
                    pre.clone(),
                    DURATION,
                    (-range, range),
                    ValueScaling::Linear,
                )
                .class("overlay");
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    (-range, range),
                    (-8..=8).map(|x| x as f32 / 8.0).collect::<Vec<_>>(),
                    Orientation::Horizontal,
                );
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    (-range, range),
                    vec![1.0, 0.75, 0.5, 0.25, 0.0, -0.25, -0.5, -0.75, -1.0],
                    Orientation::Horizontal,
                );
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    (0.0, DURATION),
                    (0..=DURATION.ceil() as usize * TICKS)
                        .map(|x| x as f32 / TICKS as f32)
                        .collect::<Vec<_>>(),
                    Orientation::Vertical,
                );
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    (0.0, DURATION),
                    (0..=DURATION.ceil() as usize)
                        .map(|x| x as f32)
                        .collect::<Vec<_>>(),
                    Orientation::Vertical,
                );
                ThresholdLines::new(cx)
                    .color("fg-red")
                    .top(Pixels(12.0))
                    .bottom(Pixels(12.0));
                UnitRuler::new(
                    cx,
                    (-range, range),
                    ValueScaling::Linear,
                    vec![
                        (1.00, "0.0 dB"),
                        (0.75, "-2.5 dB"),
                        (0.50, "-6.0 dB"),
                        (0.25, "-12.0 dB"),
                        (0.00, "-INF dB"),
                        (-0.25, "-12.0 dB"),
                        (-0.50, "-6.0 dB"),
                        (-0.75, "-2.5 dB"),
                        (-1.00, "0.0 dB"),
                    ],
                    Orientation::Vertical,
                )
                .left(Stretch(1.0))
                .width(Pixels(32.0))
                .right(Pixels(4.0));
            })
            .class("bg-elevated");
        });
    })
}
