use crate::Antiderivative;
use std::f64::consts::PI;

/// First-order Antiderivative Antialiasing (ADAA)
#[derive(Default, Clone)]
pub struct Processor {
    x1: f64,
    x2: f64,
    ad1_x1: f64,
    ad2_x1: f64,
    d2: f64,
    s: f64,
}

const TOL: f64 = 1.0e-5;
const TOL_SOFTNESS: f64 = 1.0e-5;

impl Processor {
    pub fn process(&mut self, x: f64, s: f64, antiderivative: &Antiderivative) -> f64 {
        use Antiderivative::*;
        match antiderivative {
            Off => self.func(x, s),
            FirstDegree => {
                let ad1_x = self.func_ad1(x, s);

                let func = self.func(0.5 * (x + self.x1), s);
                let derivative_diff = (ad1_x - self.ad1_x1) / (x - self.x1);

                let y = if (s - self.s).abs() > TOL_SOFTNESS || (x - self.x1).abs() < TOL {
                    func
                } else {
                    derivative_diff
                };

                self.ad1_x1 = ad1_x;
                self.x1 = x;
                self.s = s;

                y
            }
            SecondDegree => {
                let ad2_x0 = self.func_ad2(x, s);

                let d1 = if (x - self.x1).abs() < TOL {
                    self.func_ad1(0.5 * (x + self.x1), s)
                } else {
                    (ad2_x0 - self.ad2_x1) / (x - self.x1)
                };

                let y = if (s - self.s).abs() > TOL_SOFTNESS {
                    self.func(0.5 * (x + self.x1), s)
                } else if (x - self.x2).abs() < TOL {
                    let x_bar = 0.5 * (x + self.x2);
                    let delta = x_bar - self.x1;

                    if delta.abs() < TOL {
                        self.func(0.5 * (x_bar + self.x1), s)
                    } else {
                        2.0 / delta
                            * (self.func_ad1(x_bar, s)
                                + (self.ad2_x1 - self.func_ad2(x_bar, s)) / delta)
                    }
                } else {
                    2.0 / (x - self.x2) * (d1 - self.d2)
                };

                self.d2 = d1;
                self.x2 = self.x1;
                self.x1 = x;
                self.ad2_x1 = ad2_x0;

                y
            }
        }
    }
    #[inline]
    fn func(&mut self, x: f64, s: f64) -> f64 {
        static KNEE_OFFSET: f64 = PI / 2.0 - 1.0;
        let lower_bound = 1.0 - s;
        let upper_bound = 1.0 + s * KNEE_OFFSET;

        if x.abs() < lower_bound {
            x
        } else if x.abs() < upper_bound {
            (1.0 - s + s * (1.0 - (1.0 - x.abs()) / s).sin()) * x.signum()
        } else {
            x.signum()
        }
    }
    #[inline]
    fn func_ad1(&mut self, x: f64, s: f64) -> f64 {
        static KNEE_OFFSET: f64 = PI / 2.0 - 1.0;
        let lower_bound = 1.0 - s;
        let upper_bound = 1.0 + s * KNEE_OFFSET;

        let abs_x = x.abs();

        if abs_x < lower_bound {
            x.powi(2) / 2.0
        } else if abs_x < upper_bound {
            let offset_sin = (1.0 - s).powi(2) / 2.0 - 1.0;

            -s.powi(2) * ((1.0 - abs_x) / s - 1.0).cos() + s * (2.0 - abs_x) + abs_x + offset_sin
        } else {
            let offset_clip = -s.powi(2) * (-PI / 2.0).cos()
                - s * ((s + 1.0) * (PI / 2.0 - 1.0) - PI / 2.0)
                + (1.0 - s).powi(2) / 2.0
                - 1.0;

            abs_x + offset_clip
        }
    }
    #[inline]
    fn func_ad2(&mut self, x: f64, s: f64) -> f64 {
        static KNEE_OFFSET: f64 = PI / 2.0 - 1.0;
        let lower_bound = 1.0 - s;
        let upper_bound = 1.0 + s * KNEE_OFFSET;

        let abs_x = x.abs();

        if abs_x < lower_bound {
            x.powi(3) / 6.0
        } else if abs_x < upper_bound {
            let offset_sin = if s == 0.0 {
                1.0 / 6.0
            } else {
                s.powi(2) * (1.0 - s) + s.powi(3) * (1.0 - 1.0 / s) - (1.0 - s).powi(3) / 6.0
            };

            x.signum()
                * (1.0 / 2.0
                    * (2.0 * s.powi(3) * ((1.0 - abs_x) / s - 1.0).sin()
                        + (s.powi(2) + 2.0 * s - 1.0) * abs_x
                        + (1.0 - s) * x.powi(2)
                        + 2.0 * (1.0 - 1.0 / s) * s.powi(3))
                    - offset_sin)
        } else {
            let offset_clip = if s == 0.0 {
                -1.0 / 6.0
            } else {
                1.0 / 6.0
                    * (6.0 * s.powi(3) * (1.0 + (upper_bound - 1.0) / s).sin() - 5.0 * s.powi(3)
                        + s.powi(2) * (-3.0 * PI * upper_bound + 6.0 * upper_bound + 3.0)
                        + 3.0 * s * (upper_bound - 1.0).powi(2)
                        - 1.0)
            };

            1.0 / 2.0 * x * (s.powi(2) * (3.0 - PI) + abs_x - 1.0) - x.signum() * offset_clip
        }
    }
    // fn func(&mut self, mut x: f64, s: f64) -> f64 {
    //     static KNEE_OFFSET: f64 = PI / 2.0 - 1.0;
    //
    //     if x.abs() < 1.0 {
    //         x
    //     } else {
    //         x.signum()
    //     }
    // }
    // fn func_ad1(&mut self, mut x: f64, s: f64) -> f64 {
    //     static KNEE_OFFSET: f64 = PI / 2.0 - 1.0;
    //
    //     let abs_x = x.abs();
    //
    //     if abs_x < 1.0 {
    //         x.powi(2) / 2.0
    //     } else {
    //         abs_x - 0.5
    //     }
    // }
}
