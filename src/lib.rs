mod antialiasing;
mod editor;
mod oversampling;
mod preferences;

use crate::{antialiasing::Processor, preferences::Preferences};
use cyma::prelude::*;
use nih_plug::{prelude::*, util::db_to_gain_fast};
use nih_plug_vizia::ViziaState;
use oversampling::Lanczos3Oversampler;
use std::{f32::consts::PI, sync::{Arc, Mutex}};
use util::MINUS_INFINITY_GAIN;

const BLOCK_SIZE: usize = 32;
const MAX_OVERSAMPLING_FACTOR: usize = 3;
const MAX_OVERSAMPLING_TIMES: usize = 2usize.pow(MAX_OVERSAMPLING_FACTOR as u32);
const MAX_OVERSAMPLED_BLOCK_SIZE: usize = BLOCK_SIZE * MAX_OVERSAMPLING_TIMES;

#[inline]
pub fn transfer(mut sample: f32, threshold: f32, softness: f32) -> f32 {
    apply_transfer(&mut sample, &threshold, &softness);
    sample
}

#[inline]
fn apply_transfer(sample: &mut f32, threshold: &f32, softness: &f32) {
    static KNEE_OFFSET: f32 = PI / 2. - 1.;

    *sample /= threshold;

    let abs_sample = sample.abs();

    if abs_sample < 1.0 + softness * KNEE_OFFSET {
        if abs_sample > 1.0 - softness {
            *sample = ((1.0 - softness) + softness * (1.0 - (1.0 - abs_sample) / softness).sin())
                * sample.signum();
        }
    } else {
        *sample = sample.signum();
    }

    *sample *= threshold;
}

#[inline]
fn transfer_curve(softness: f32) -> impl Fn(f32) -> f32 {
    move |sample: f32| {
        static KNEE_OFFSET: f32 = PI / 2. - 1.;

        let abs_sample = sample.abs();

        if abs_sample < 1.0 + softness * KNEE_OFFSET {
            if abs_sample > 1.0 - softness {
                ((1.0 - softness) + softness * (1.0 - (1.0 - abs_sample) / softness).sin())
                    * sample.signum()
            } else {
                sample
            }
        } else {
            sample.signum()
        }
    }
}

pub struct Klyp {
    params: Arc<KlypParams>,
    pre: Arc<MonoBus>,
    post: Arc<MonoBus>,
    processors: Vec<Processor>,
    oversamplers: Vec<Lanczos3Oversampler>,
    scratch_buffers: Box<ScratchBuffers>,
    preferences: Arc<Mutex<Option<Preferences>>>
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum Antiderivative {
    Off,
    FirstDegree,
    SecondDegree,
}

#[derive(Params)]
pub struct KlypParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "threshold"]
    pub threshold: FloatParam,
    #[id = "softness"]
    pub softness: FloatParam,
    #[nested(id_prefix = "aa", group = "oversampling")]
    pub antialiasing: AntialiasingParams,
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
}

#[derive(Params)]
pub struct AntialiasingParams {
    #[id = "oversampling"]
    pub oversampling: IntParam,
    #[id = "antiderivative"]
    pub antiderivative: BoolParam,
}

struct ScratchBuffers {
    gain: [f32; MAX_OVERSAMPLED_BLOCK_SIZE],
    threshold: [f32; MAX_OVERSAMPLED_BLOCK_SIZE],
    softness: [f32; MAX_OVERSAMPLED_BLOCK_SIZE],
}

impl Default for ScratchBuffers {
    fn default() -> Self {
        Self {
            gain: [0.0; MAX_OVERSAMPLED_BLOCK_SIZE],
            threshold: [1.0; MAX_OVERSAMPLED_BLOCK_SIZE],
            softness: [0.0; MAX_OVERSAMPLED_BLOCK_SIZE],
        }
    }
}

impl Default for Klyp {
    fn default() -> Self {
        Self {
            params: Arc::new(KlypParams::default()),
            pre: Arc::new(Default::default()),
            post: Arc::new(Default::default()),
            processors: vec![],
            oversamplers: vec![],
            scratch_buffers: Box::default(),
            preferences: Default::default()
        }
    }
}

impl Default for KlypParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Pre-Gain",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            threshold: FloatParam::new(
                "Threshold",
                1.0,
                FloatRange::Skewed {
                    min: MINUS_INFINITY_GAIN,
                    max: 1.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            softness: FloatParam::new("Softness", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(2))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            antialiasing: AntialiasingParams {
                oversampling: IntParam::new(
                    "Oversampling",
                    0,
                    IntRange::Linear {
                        min: 0,
                        max: MAX_OVERSAMPLING_FACTOR as i32,
                    },
                )
                .with_value_to_string(Arc::new(|x| format!("{}x", 2u32.pow(x as u32))))
                .with_string_to_value(Arc::new(|x| {
                    x.parse::<i32>().map(|x| x.ilog2() as i32).ok()
                })),
                antiderivative: BoolParam::new("Antiderivative", true),
            },
            editor_state: editor::default_state(),
        }
    }
}

impl Plugin for Klyp {
    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let channels = audio_io_layout
            .main_input_channels
            .map(|c| c.get())
            .unwrap_or(1) as usize;

        self.pre.set_sample_rate(buffer_config.sample_rate);
        self.post.set_sample_rate(buffer_config.sample_rate);

        self.processors = vec![Processor::default(); channels];
        self.oversamplers.resize_with(channels, || {
            Lanczos3Oversampler::new(BLOCK_SIZE, MAX_OVERSAMPLING_FACTOR)
        });

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let gui_open = self.params.editor_state.is_open();

        let oversampling = self.params.antialiasing.oversampling.value() as usize;

        let antiderivative = if self.params.antialiasing.antiderivative.value() {
            match oversampling {
                0 => Antiderivative::Off,
                1 => Antiderivative::FirstDegree,
                2.. => Antiderivative::SecondDegree,
            }
        } else {
            Antiderivative::Off
        };

        let mut latency = 0;

        if let Some(oversampler) = self.oversamplers.first() {
            latency += oversampler.latency(oversampling);
        }

        if antiderivative != Antiderivative::Off {
            latency += 1;
        }

        context.set_latency_samples(latency);

        for (_, mut block) in buffer.iter_blocks(BLOCK_SIZE) {
            let samples = block.samples();
            let samples_upscaled = samples * (1 << oversampling);

            let gain = &mut self.scratch_buffers.gain;
            self.params.gain.smoothed.next_block(gain, samples_upscaled);

            let threshold = &mut self.scratch_buffers.threshold;
            self.params
                .threshold
                .smoothed
                .next_block(threshold, samples_upscaled);

            let softness = &mut self.scratch_buffers.softness;
            self.params
                .softness
                .smoothed
                .next_block(softness, samples_upscaled);

            if gui_open {
                let channels = block.channels() as f32;
                for (i, sample) in block.iter_samples().enumerate() {
                    let gain = unsafe { db_to_gain_fast(*gain.get_unchecked(i)) };
                    if gain.is_infinite() {
                        panic!();
                    }
                    self.pre
                        .send(sample.into_iter().map(|x| *x).sum::<f32>() / channels * gain);
                }
            }

            for (block_channel, (oversampler, processor)) in block
                .into_iter()
                .zip(self.oversamplers.iter_mut().zip(self.processors.iter_mut()))
            {
                for (i, sample) in block_channel.iter_mut().enumerate() {
                    let gain = unsafe { db_to_gain_fast(*gain.get_unchecked(i)) };
                    let threshold = unsafe { threshold.get_unchecked(i) };
                    *sample *= gain;
                    *sample /= threshold;
                }
                oversampler.process(block_channel, oversampling, |upsampled| {
                    for (i, sample) in upsampled.iter_mut().enumerate() {
                        let softness = unsafe { softness.get_unchecked(i) };

                        *sample =
                            processor.process(*sample as f64, *softness as f64, &antiderivative)
                                as f32;
                    }
                });
                for (i, sample) in block_channel.iter_mut().enumerate() {
                    let threshold = unsafe { threshold.get_unchecked(i) };
                    *sample *= threshold;
                }
            }
        }

        if gui_open {
            self.post.send_buffer_summing(buffer);
        }

        return ProcessStatus::Normal;
    }

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
            self.pre.clone(),
            self.post.clone(),
            self.preferences.clone()
        )
    }

    const NAME: &'static str = "KLYP";
    const VENDOR: &'static str = "Voidstar Audio";
    const URL: &'static str = "https://voidstaraudio.com/klyp";
    const EMAIL: &'static str = "223230@pm.me";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
}

impl ClapPlugin for Klyp {
    const CLAP_ID: &'static str = "com.voidstar-audio.klyp";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Intuitive soft clipper with superb sound quality.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Klyp {
    const VST3_CLASS_ID: [u8; 16] = *b"vsKLYP......0100";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Klyp);
nih_export_vst3!(Klyp);
