use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rand::{rng};

pub struct SoundProperties {
    pub frequency: f32,
    pub duration: f32,
}

trait Waveform: Send {
    fn sample_phase(&self, phase: f32) -> f32;
}

struct SineWave;

struct SquareWave;

struct SawWave;

impl Waveform for SineWave {
    fn sample_phase(&self, phase: f32) -> f32 {
        (phase * 2.0 * std::f32::consts::PI).sin()
    }
}

impl Waveform for SquareWave {
    fn sample_phase(&self, phase: f32) -> f32 {
        if phase < 0.5 { 1.0 } else { -1.0 }
    }
}

impl Waveform for SawWave {
    fn sample_phase(&self, phase: f32) -> f32 {
        2.0 * phase - 1.0
    }
}

struct Sound {
    pub props: SoundProperties,
    pub waveform: Box<dyn Waveform>,
}

impl Sound {
    fn get_value(&self, phase: f32) -> f32 {
        self.waveform.sample_phase(phase)
    }

    fn frequency(&self) -> f32 {
        self.props.frequency
    }

    fn duration(&self) -> f32 {
        self.props.duration
    }

    fn new<Type: Waveform + 'static> (waveform: Type, frequency: f32, duration: f32) -> Self{
        Self{props: SoundProperties { frequency: frequency, duration: duration }, waveform: Box::new(waveform)}
    }
}

fn main() {
    // let test_sound = Sound::new(SineWave, 400.0, 0.1);

    // 1. Connect to the speakers
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let config = device.default_output_config().unwrap().config();

    // // 2. Get the sample rate (usually 44100 or 48000)
    let sample_rate = config.sample_rate as f32;
    let channels = config.channels as usize;

    let mut phase = 0.0;

    let song: Vec<Sound> = vec![
        Sound::new(SineWave, 400.0, 1.0),
        Sound::new(SineWave, 600.0, 1.0),
        Sound::new(SineWave, 300.0, 1.0),
    ];



    // sine_wave.get_value(0.4);
    //
    let volume = 1.0;
    let mut current_audio_frame = 0.0;
    let mut current_sound_index = 0 as usize;

    // let mut filter1: f32 = 0.0;
    // let mut filter2: f32 = 0.0;
    // let mut filter3: f32 = 0.0;

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for chunk in data.chunks_mut(channels) {
                // let mut rng = rng();

                // let white = rng.random_range(-1.0..1.0);

                // filter1 = (filter1 * 0.99) + (white * 0.01);

                // filter2 = (filter2 * 0.99) + (filter1 * 0.01);

                // filter3 = (filter3 * 0.99) + (filter2 * 0.01);
                // let value = (filter3.clamp(-1.0, 1.0)) * volume;
                //
                if current_sound_index < song.len() {
                    let current_sound = &song[current_sound_index];
                    phase = (phase + (current_sound.frequency() / sample_rate)) % 1.0;
                    let value = current_sound.get_value(phase) * volume;

                    current_audio_frame += 1 as f32;
                    if current_audio_frame/sample_rate > current_sound.duration() {
                        current_sound_index += 1;
                        current_audio_frame = 0.0;
                    }

                    for sample in chunk {
                        *sample = value;
                    }
                } else {
                    current_sound_index = 0;
                }

            }
        },

        |err| { /* handle error */ },
        None,
    ).unwrap();

    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(60000));
    println!("end");

}
