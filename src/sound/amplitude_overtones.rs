use sound::*;
use std::rc::Rc;
use std::cell::RefCell;

/// Provides time dependent amlitude changes both for the fundamental tone and for overtones.
pub trait AmplitudeOvertonesProvider {
    /// Provides the results of the amplitude calculations for a given overtone.
    /// For the fundamental tone `overtone = 0`.
    fn get(&self, overtone: usize, result: &mut [SampleCalc]) -> SoundResult<()>;
    /// Applies the amplitude function over existing samples for a given overtone.
    /// For the fundamental tone `overtone = 0`. It multiplies each sample with it's new amplitude.
    fn apply(&self, overtone: usize, samples: &mut [SampleCalc]) -> SoundResult<()>;
    /// Resets to the initial state.
    fn restart(&self);
}

/// The `AmplitudeOvertonesJoinable` trait is used to specify the ability of joining
/// amplitude structures (with overtones) together, forming a sequence of them.
pub trait AmplitudeOvertonesJoinable {
    /// Sets the initial amplitude, and resets time.
    fn set_amplitude_start(&self, amplitude: &[SampleCalc]) -> SoundResult<()>;
    // Provides the final amplitude value. (Independent of the current progress phase.)
    // fn get_amplitude_final(&self) -> SampleCalc;
}

/// Amplitude is not changing by time, this function gives the overtone amplitudes too.
#[derive(Debug, Clone)]
pub struct AmplitudeConstOvertones {
    amplitude: RefCell<Vec<SampleCalc>>,
}

impl AmplitudeConstOvertones {
    /// custom constructor
    /// It normalizes the amplitudes, so the sum of them will be 1.0.
    /// `overtone_count` is independent of the size of `amplitude`.
    pub fn new(overtone_count: usize,
               amplitude: &[SampleCalc])
               -> SoundResult<AmplitudeConstOvertones> {
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in amplitude.iter().take(overtone_count + 1) {
            if *amplitude_check < 0.0 {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if amplitude_sum == 0.0 {
            return Err(Error::AmplitudeInvalid);
        };
        let mut amplitude_new = vec![0.0; overtone_count + 1]; // fundamental tone included in size
        // normalization
        for (item, amplitude_old) in amplitude_new.iter_mut().zip(amplitude) {
            *item = amplitude_old / amplitude_sum;
        }
        Ok(AmplitudeConstOvertones { amplitude: RefCell::new(amplitude_new) })
    }
}

impl AmplitudeOvertonesProvider for AmplitudeConstOvertones {
    fn get(&self, overtone: usize, result: &mut [SampleCalc]) -> SoundResult<()> {
        let amplitude = self.amplitude.borrow();
        if overtone >= amplitude.len() {
            for item in result.iter_mut() {
                *item = 0.0;
            }
        } else {
            for item in result.iter_mut() {
                *item = amplitude[overtone];
            }
        }
        Ok(())
    }

    fn apply(&self, overtone: usize, samples: &mut [SampleCalc]) -> SoundResult<()> {
        let amplitude = self.amplitude.borrow();
        if overtone >= amplitude.len() {
            for item in samples.iter_mut() {
                *item = 0.0;
            }
        } else {
            for item in samples.iter_mut() {
                *item *= amplitude[overtone];
            }
        }
        Ok(())
    }

    fn restart(&self) {
        // Do nothing, as nothing changes by time.
    }
}

impl AmplitudeOvertonesJoinable for AmplitudeConstOvertones {
    fn set_amplitude_start(&self, amplitude: &[SampleCalc]) -> SoundResult<()> {
        let mut self_amplitude = self.amplitude.borrow_mut();
        // checking the input data
        if amplitude.len() > self_amplitude.len() {
            return Err(Error::OvertoneCountInvalid);
        }
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in amplitude {
            if (*amplitude_check < 0.0) || (*amplitude_check > 1.0) {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if (amplitude_sum == 0.0) || (amplitude_sum > 1.0) {
            return Err(Error::AmplitudeInvalid);
        };
        // Copying input amplitudes and filling the rest with zero.
        let (amp_data, amp_empty) = self_amplitude.split_at_mut(amplitude.len());
        for (item, amplitude) in amp_data.iter_mut().zip(amplitude) {
            *item = *amplitude;
        }
        for item in amp_empty.iter_mut() {
            *item = 0.0;
        }
        Ok(())
    }
}

/// Amplitude is decaying exponentially, also for overtones
/// [Exponential decay](https://en.wikipedia.org/wiki/Exponential_decay)
/// index: 0 = fundamental tone, 1.. = overtones.
#[derive(Debug, Clone)]
pub struct AmplitudeDecayExpOvertones {
    sample_time: SampleCalc,
    amplitude_init: Vec<SampleCalc>, // initial amplitudes
    multiplier: Vec<SampleCalc>,
    amplitude: RefCell<Vec<SampleCalc>>,
}

impl AmplitudeDecayExpOvertones {
    /// custom constructor
    /// It normalizes the amplitudes, so the sum of the starting amplitudes will be 1.0.
    /// `half_life` is the time required to reduce the amplitude to it's half.
    /// `overtone_count` is independent of the size of `amplitude` and `half_life` too.
    pub fn new(sample_rate: SampleCalc,
               overtone_count: usize,
               amplitude: &[SampleCalc],
               half_life: &[SampleCalc])
               -> SoundResult<AmplitudeDecayExpOvertones> {
        let sample_time = try!(get_sample_time(sample_rate));
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in amplitude.iter().take(overtone_count + 1) {
            if *amplitude_check < 0.0 {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if amplitude_sum == 0.0 {
            return Err(Error::AmplitudeInvalid);
        };
        let mut amplitude_new = vec![0.0; overtone_count + 1]; // fundamental tone included in size
        // normalization
        for (item, amplitude_old) in amplitude_new.iter_mut().zip(amplitude) {
            *item = amplitude_old / amplitude_sum;
        }
        for item in half_life {
            if *item <= 0.0 {
                return Err(Error::AmplitudeRateInvalid);
            }
        }
        let mut multiplier = vec![0.0; overtone_count + 1]; // fundamental tone included in size
        let half: SampleCalc = 0.5;
        for (item, hl) in multiplier.iter_mut().zip(half_life) {
            *item = half.powf(sample_time / hl);
        }
        Ok(AmplitudeDecayExpOvertones {
            sample_time: sample_time,
            amplitude_init: amplitude_new.clone(),
            multiplier: multiplier,
            amplitude: RefCell::new(amplitude_new),
        })
    }
}

impl AmplitudeOvertonesProvider for AmplitudeDecayExpOvertones {
    fn get(&self, overtone: usize, result: &mut [SampleCalc]) -> SoundResult<()> {
        let mut amplitude = self.amplitude.borrow_mut();
        if (overtone >= amplitude.len()) || (overtone >= self.multiplier.len()) {
            for item in result.iter_mut() {
                *item = 0.0;
            }
            return Ok(());
        };
        let mut amplitude_overtone = &mut amplitude[overtone];
        for item in result.iter_mut() {
            *amplitude_overtone *= self.multiplier[overtone];
            *item = *amplitude_overtone;
        }
        Ok(())
    }

    fn apply(&self, overtone: usize, samples: &mut [SampleCalc]) -> SoundResult<()> {
        let mut amplitude = self.amplitude.borrow_mut();
        if (overtone >= amplitude.len()) || (overtone >= self.multiplier.len()) {
            for item in samples.iter_mut() {
                *item = 0.0;
            }
            return Ok(());
        };
        let mut amplitude_overtone = &mut amplitude[overtone];
        for item in samples.iter_mut() {
            *amplitude_overtone *= self.multiplier[overtone];
            *item *= *amplitude_overtone;
        }
        Ok(())
    }

    fn restart(&self) {
        for (amplitude, amplitude_init) in self.amplitude
            .borrow_mut()
            .iter_mut()
            .zip(self.amplitude_init.iter()) {
            *amplitude = *amplitude_init;
        }
    }
}

impl AmplitudeOvertonesJoinable for AmplitudeDecayExpOvertones {
    fn set_amplitude_start(&self, amplitude: &[SampleCalc]) -> SoundResult<()> {
        let mut self_amplitude = self.amplitude.borrow_mut();
        // checking the input data
        if amplitude.len() > self_amplitude.len() {
            return Err(Error::OvertoneCountInvalid);
        }
        let mut amplitude_sum: SampleCalc = 0.0;
        for amplitude_check in amplitude {
            if (*amplitude_check < 0.0) || (*amplitude_check > 1.0) {
                return Err(Error::AmplitudeInvalid);
            };
            amplitude_sum += *amplitude_check;
        }
        if (amplitude_sum == 0.0) || (amplitude_sum > 1.0) {
            return Err(Error::AmplitudeInvalid);
        };
        // Copying input amplitudes and filling the rest with zero.
        let (amp_data, amp_empty) = self_amplitude.split_at_mut(amplitude.len());
        for (item, amplitude) in amp_data.iter_mut().zip(amplitude) {
            *item = *amplitude;
        }
        for item in amp_empty.iter_mut() {
            *item = 0.0;
        }
        Ok(())
    }
}

#[doc(hidden)]
#[allow(dead_code)]
struct AmplitudeOvertonesSequenceItem {
    amplitude: Rc<AmplitudeOvertonesProvider>,
    duration: SampleCalc,
}

/// A sequence of amplitude functions with overtones.
#[doc(hidden)]
#[allow(dead_code)]
pub struct AmplitudeOvertonesSequence {
    amplitudes: RefCell<Vec<AmplitudeOvertonesSequenceItem>>,
}
