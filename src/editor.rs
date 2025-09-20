mod curve;
mod threshold_lines;

use astra::prelude::*;
use curve::ClippingCurve;
use cyma::prelude::*;
use nih_plug::params::Param;
use nih_plug::prelude::{Editor, Enum};
use nih_plug::util::db_to_gain;
use nih_plug_vizia::vizia::{image, prelude::*};
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;
use threshold_lines::ThresholdLines;

use crate::KlypParams;

#[derive(Enum, Default, Clone)]
pub enum RangePreset {
    #[name = "0 dB"]
    #[default]
    A,
    #[name = "6 dB"]
    B,
    #[name = "12 dB"]
    C,
}
impl RangePreset {
    pub fn raw_scalar(&self) -> f32 {
        match self {
            Self::A => 1.0,
            Self::B => 2.0,
            Self::C => 4.0,
        }
    }
    fn to_range(&self) -> (f32, f32) {
        let max = self.raw_scalar() * (400.0 / (400.0 - 24.0));
        (-max, max)
    }
}

#[derive(Enum, Default, Clone)]
pub enum DurationPreset {
    #[name = "1s"]
    A,
    #[name = "2s"]
    B,
    #[default]
    #[name = "5s"]
    C,
    #[name = "10s"]
    D,
}
impl DurationPreset {
    fn to_duration(&self) -> f32 {
        match self {
            Self::A => 1.0,
            Self::B => 2.0,
            Self::C => 5.0,
            Self::D => 10.0,
        }
    }
}

#[derive(Lens)]
pub struct Data {
    params: Arc<KlypParams>,
    range: RangePreset,
    duration: DurationPreset,
}

impl Model for Data {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|editor_event, _| match editor_event {
            EditorEvent::UpdateRange(i) => self.range = RangePreset::from_index(*i),
            EditorEvent::UpdateDuration(i) => self.duration = DurationPreset::from_index(*i),
        });
    }
}

pub enum EditorEvent {
    UpdateRange(usize),
    UpdateDuration(usize),
}

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
                dropdown.vis popup {
                    top: -57px;
                }
            "#,
        );

        const MAX_RANGE: f32 = 400.0 / (400.0 - 24.0);

        Data {
            params: params.clone(),
            range: Default::default(),
            duration: Default::default(),
        }
        .build(cx);

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                ZStack::new(cx, |cx| {
                    ClippingCurve::new(cx, pre.clone(), 300.0, Data::range);
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
                                    ParamSelector::new(cx, Data::params, |p| {
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
                .size(Pixels(212.0))
                .child_space(Pixels(12.0));
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
                const TICKS: usize = 2;
                Oscilloscope::new(
                    cx,
                    post.clone(),
                    Data::duration.map(|d| d.to_duration()),
                    Data::range.map(|r| r.to_range()),
                    ValueScaling::Linear,
                );
                Oscilloscope::new(
                    cx,
                    pre.clone(),
                    Data::duration.map(|d| d.to_duration()),
                    Data::range.map(|r| r.to_range()),
                    ValueScaling::Linear,
                )
                .class("overlay");
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    RangePreset::A.to_range(),
                    (-8..=8).map(|x| x as f32 / 8.0).collect::<Vec<_>>(),
                    Orientation::Horizontal,
                );
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    RangePreset::A.to_range(),
                    vec![1.0, 0.75, 0.5, 0.25, 0.0, -0.25, -0.5, -0.75, -1.0],
                    Orientation::Horizontal,
                );
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    Data::range.map(|r| r.to_range()),
                    vec![1.0, -1.0],
                    Orientation::Horizontal,
                )
                .opacity(0.2);
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    Data::duration.map(|r| (0.0, r.to_duration())),
                    (0..=10 * TICKS)
                        .map(|x| x as f32 / TICKS as f32)
                        .collect::<Vec<_>>(),
                    Orientation::Vertical,
                );
                Grid::new(
                    cx,
                    ValueScaling::Linear,
                    Data::duration.map(|r| (0.0, r.to_duration())),
                    (0..=10).map(|x| x as f32).collect::<Vec<_>>(),
                    Orientation::Vertical,
                );
                ThresholdLines::new(cx, Data::range)
                    .color("fg-red")
                    .top(Pixels(12.0))
                    .bottom(Pixels(12.0));
                Binding::new(
                    cx,
                    Data::range.map(|r| r.clone().to_index()),
                    |cx, range| {
                        let range = RangePreset::from_index(range.get(cx));

                        let ticks = match range {
                            RangePreset::A => vec![
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
                            RangePreset::B => vec![
                                (2.00, "6.0 dB"),
                                (1.50, "3.5 dB"),
                                (1.00, "0.0 dB"),
                                (0.50, "-6.0 dB"),
                                (0.00, "-INF dB"),
                                (-0.50, "-6.0 dB"),
                                (-1.00, "0.0 dB"),
                                (-1.50, "3.5 dB"),
                                (-2.00, "6.0 dB"),
                            ],
                            RangePreset::C => vec![
                                (4.00, "12.0 dB"),
                                (3.00, "9.5 dB"),
                                (2.00, "6.0 dB"),
                                (1.00, "0.0 dB"),
                                (0.00, "-INF dB"),
                                (-1.00, "0.0 dB"),
                                (-2.00, "6.0 dB"),
                                (-3.00, "9.5 dB"),
                                (-4.00, "12.0 dB"),
                            ],
                        };

                        UnitRuler::new(
                            cx,
                            range.to_range(),
                            ValueScaling::Linear,
                            ticks,
                            Orientation::Vertical,
                        )
                        .left(Stretch(1.0))
                        .width(Pixels(32.0))
                        .right(Pixels(4.0));
                    },
                );
                Dropdown::new(
                    cx,
                    |cx| {
                        HStack::new(cx, |cx| {
                            Label::new(
                                cx,
                                Data::range.map(|r| RangePreset::variants()[r.clone().to_index()]),
                            )
                            .pointer_events(false);
                            Label::new(cx, "  ").pointer_events(false);
                            Label::new(
                                cx,
                                Data::duration
                                    .map(|r| DurationPreset::variants()[r.clone().to_index()]),
                            )
                            .width(Stretch(1.0))
                            .pointer_events(false);
                            Image::new(cx, "chevron_down.png").pointer_events(false);
                        })
                    },
                    |cx| {
                        VStack::new(cx, |cx| {
                            HStack::new(cx, |cx| {
                                Label::new(cx, "RANGE")
                                    .top(Stretch(1.0))
                                    .bottom(Stretch(1.0));
                                Selector::new(cx, Data::range)
                                    .on_toggle(|cx, i| cx.emit(EditorEvent::UpdateRange(i)));
                            })
                            .height(Auto)
                            .col_between(Stretch(1.0));
                            HStack::new(cx, |cx| {
                                Label::new(cx, "DURATION")
                                    .top(Stretch(1.0))
                                    .bottom(Stretch(1.0));
                                Selector::new(cx, Data::duration)
                                    .on_toggle(|cx, i| cx.emit(EditorEvent::UpdateDuration(i)));
                            })
                            .height(Auto)
                            .col_between(Stretch(1.0));
                        })
                        .child_space(Pixels(8.0))
                        .row_between(Pixels(4.0))
                        .height(Auto);
                    },
                )
                .class("vis")
                .class("ghost")
                .width(Pixels(80.0))
                .left(Pixels(12.0))
                .top(Stretch(1.0))
                .bottom(Pixels(12.0));
            })
            .class("bg-elevated");
        });
    })
}
