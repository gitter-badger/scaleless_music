#![feature(test)]

extern crate test;
extern crate music;

use test::Bencher;
use music::sound::*;
use music::sound::wave::*;
use music::sound::frequency::*;
use music::sound::amplitude::*;
use music::math::*;
use std::rc::Rc;

const BENCH_SAMPLE_RATE: SampleCalc = 192_000.0;
const BENCH_BUFFER_SIZE: usize = 256;
const SAMPLETIME: SampleCalc = BENCH_BUFFER_SIZE as SampleCalc / BENCH_SAMPLE_RATE;


#[bench]
fn sine_fast_b(bencher: &mut Bencher) {
    let mut rad: f32 = 0.0;
    let mut s: f32 = 0.0;

    bencher.iter(|| {
        rad += 0.001;
        s = rad.sin_fast();
        test::black_box(s);
    });
}

#[bench]
fn sine_b(bencher: &mut Bencher) {
    let mut rad: f32 = 0.0;
    let mut s: f32 = 0.0;

    bencher.iter(|| {
        rad += 0.001;
        s = rad.sin();
        test::black_box(s);
    });
}

#[bench]
fn exp_b(bencher: &mut Bencher) {
    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;

    bencher.iter(|| {
        x += 0.001;
        y = x.exp();
        test::black_box(y);
    });
}

// FrequencyConst
#[bench]
fn freqconst(bencher: &mut Bencher) {
    let mut frequency_buffer: Vec<SampleCalc> = vec![440.0; BENCH_BUFFER_SIZE];
    let frequency = FrequencyConst::new(440.0).unwrap();

    bencher.iter(|| {
        frequency.get(0.0, None, &mut frequency_buffer).unwrap();
    });
}

// AmplitudeConstOvertones
#[bench]
fn ampconst_overtone(bencher: &mut Bencher) {
    let mut amplitude_buffer: Vec<SampleCalc> = vec![0.0; BENCH_BUFFER_SIZE];
    let mut time: SampleCalc = 0.0;
    let amplitude = {
        let overtones_amplitude: Vec<SampleCalc> = vec![1.0, 0.5];
        AmplitudeConstOvertones::new(overtones_amplitude).unwrap()
    };

    bencher.iter(|| {
        amplitude.get(time, 0, &mut amplitude_buffer).unwrap();
        time += SAMPLETIME;
    });
}

// AmplitudeDecayExpOvertones
#[bench]
fn ampdec_overtone(bencher: &mut Bencher) {
    let mut amplitude_buffer: Vec<SampleCalc> = vec![0.0; BENCH_BUFFER_SIZE];
    let mut time: SampleCalc = 0.0;
    let amplitude = {
        let overtones_amplitude: Vec<SampleCalc> = vec![1.0, 0.5];
        let overtones_dec_rate: Vec<SampleCalc> = vec![-1.0, -2.0];
        AmplitudeDecayExpOvertones::new(BENCH_SAMPLE_RATE, overtones_amplitude, overtones_dec_rate)
            .unwrap()
    };

    bencher.iter(|| {
        amplitude.get(time, 0, &mut amplitude_buffer).unwrap();
        time += SAMPLETIME;
    });
}

// FrequencyConst, Note{ AmplitudeDecayExpOvertones with 16 overtones }
#[bench]
fn note_freqconst_ampdec_overtones16(bencher: &mut Bencher) {
    let mut generator_buffer: Vec<SampleCalc> = vec![0.0; BENCH_BUFFER_SIZE];
    let mut frequency_buffer: Vec<SampleCalc> = vec![440.0; BENCH_BUFFER_SIZE];
    let mut time: SampleCalc = 0.0;
    let frequency = FrequencyConst::new(440.0).unwrap();
    let amplitude = {
        let overtones_amplitude: Vec<SampleCalc> = vec![10.0, 1.0, 1.0, 0.95, 0.9, 0.9, 0.86,
                                                        0.83, 0.80, 0.78, 0.76, 0.74, 0.73, 0.72,
                                                        0.71, 0.70];
        let overtones_dec_rate: Vec<SampleCalc> = vec![-1.0, -1.4, -1.9, -2.1, -2.4, -3.0, -3.5,
                                                       -3.7, -3.8, -4.0, -4.2, -4.4, -4.8, -5.3,
                                                       -6.1, -7.0];
        AmplitudeDecayExpOvertones::new(BENCH_SAMPLE_RATE, overtones_amplitude, overtones_dec_rate)
            .unwrap()
    };
    let note = Note::new(BENCH_SAMPLE_RATE, BENCH_BUFFER_SIZE, Rc::new(amplitude), 16).unwrap();

    bencher.iter(|| {
        frequency.get(time, None, &mut frequency_buffer).unwrap();
        note.get(time, &frequency_buffer, &mut generator_buffer).unwrap();
        time += SAMPLETIME;
        // test::black_box(&mut generator_buffer);
    });
}
