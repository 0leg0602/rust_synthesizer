/// @author Oleg
/// time-start: ???
/// time-end: Tue 21 Apr 21:21:40 EDT 2026
/// 
/// For my ISU stage 3 part 2 I decided to create a stand alone program instead of question answer style.
/// My program is a basic music sythesizer.
/// Instead of using a hight level sound generation Library,
/// I decided to try and make sound using cpal a low-level library designed for audio input and output.
/// To create sound using cpal you have to use a separate sound thread,
/// so every variable which I want to have control over in my main thread has to be a Atomic Reference Counter (ARC)
/// 
/// The logic behind playing the songs is simple:
/// There is a songs vector (array) containing a whole bunch of songs,
/// the variable current_song_index correspond to which song currently playing,
/// song 0 is just silence
/// by changin this variable you can switch the song
/// 
/// I made a simple cli to make it easier to list and switch songs,
/// however to make a song you have to edit the source code
/// 
/// Inheritance is not possible in rust you can not create an object which extends another objects 
/// instead you use composition, implementaion and traits to achive simular structure


use std::{io::{self, Write}, sync::{Arc, Mutex, atomic::{AtomicI32, AtomicUsize, Ordering}}};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rand::RngExt;

pub struct SoundProperties {
    pub frequency: f32,
    pub duration: f32,
}

// Abstraction, traits have abstract functions meaning anything that implements this trait must define this funtion.
trait Waveform: Send {
    fn sample_phase(&self, phase: f32) -> f32;
}

struct SineWave;

struct SquareWave;

struct SawWave;

struct WhiteNoise;
struct RedNoise {
    filter1: Mutex<f32>,
    filter2: Mutex<f32>,
}

impl Waveform for SineWave {
    fn sample_phase(&self, phase: f32) -> f32 {
        (phase * 2.0 * std::f32::consts::PI).sin()
    }
}

impl Waveform for SquareWave {
    fn sample_phase(&self, phase: f32) -> f32 {
        if phase < 0.5 { 
            1.0
        } else { 
            -1.0
        }
    }
}

impl Waveform for SawWave {
    fn sample_phase(&self, phase: f32) -> f32 {
        2.0 * phase - 1.0
    }
}

impl Waveform for WhiteNoise {
    fn sample_phase(&self, _phase: f32) -> f32 {
        rand::rng().random_range(-1.0..1.0)
    }
}

impl Waveform for RedNoise {
    fn sample_phase(&self, _phase: f32) -> f32 {
        let mut filter1 = self.filter1.lock().unwrap();
        let mut filter2 = self.filter2.lock().unwrap();

        let white = rand::rng().random_range(-1.0..1.0);
        *filter1 = (*filter1 * 0.99) + (white * 0.01);
        *filter2 = (*filter2 * 0.99) + (*filter1 * 0.01);
        (*filter2).clamp(-1.0, 1.0) * 10.0


    }
}

struct Sound {
    pub props: SoundProperties,
    pub waveform: Box<dyn Waveform + Send + Sync>,
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

    fn new<Type: Waveform + Send + Sync + 'static> (waveform: Type, frequency: f32, duration: f32) -> Self {
        Self{props: SoundProperties { frequency: frequency, duration: duration }, waveform: Box::new(waveform)}
    }
}
// Composition one "Object" (struct) has a field of arrays 
struct Song {
    pub name: &'static str,
    pub sounds: Vec<Sound>,
}

impl Song {
    fn new(name: &'static str, sounds: Vec<Sound>) -> Self {
        Self { name, sounds }
    }
}



fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let config = device.default_output_config().unwrap().config();

    let sample_rate = config.sample_rate as f32;
    let channels = config.channels as usize;

    let mut phase = 0.0;

    let songs = Arc::new(vec![
        Song::new("silence", vec![]),
        Song::new("happy",
            vec![
                Sound::new(SquareWave, 164.0, 0.25),
                Sound::new(SquareWave, 220.0, 0.25),
                Sound::new(SquareWave, 246.0, 0.25),
                Sound::new(SquareWave, 246.0, 0.25),
                Sound::new(SquareWave, 184.0, 0.25),
                Sound::new(SquareWave, 220.0, 0.25),
                Sound::new(SquareWave, 220.0, 0.25),
                Sound::new(SquareWave, 246.0, 0.25),
            ]
        ),
        Song::new("flute",
            vec![
                Sound::new(SineWave, 493.0, 0.25),
                Sound::new(SineWave, 554.0, 0.25),
                Sound::new(SineWave, 622.0, 0.25),
                Sound::new(SineWave, 544.0, 0.25),
                Sound::new(SineWave, 493.0, 0.25),
                Sound::new(SineWave, 554.0, 0.25),
                Sound::new(SineWave, 493.0, 0.25),
                Sound::new(SineWave, 554.0, 0.25),
                Sound::new(SineWave, 622.0, 0.25),
                Sound::new(SineWave, 622.0, 0.25),
                Sound::new(SineWave, 554.0, 0.25),
                Sound::new(SineWave, 493.0, 0.25),
            ]
        ),
        Song::new("warning",
            vec![
                Sound::new(SquareWave, 55.0, 0.5),
                Sound::new(SquareWave, 87.0, 0.5),
                Sound::new(SquareWave, 92.0, 0.5),
                Sound::new(SquareWave, 77.0, 0.5),
                Sound::new(SquareWave, 55.0, 0.5),
                Sound::new(SquareWave, 77.0, 0.5),
                Sound::new(SquareWave, 82.0, 0.5),
                Sound::new(SquareWave, 92.0, 0.5),
            ]
        ),
        Song::new("bass",
            vec![
                Sound::new(SquareWave, 123.0, 0.6),
                Sound::new(SquareWave, 61.0, 0.6),
                Sound::new(SquareWave, 123.0, 0.6),
                Sound::new(SquareWave, 61.0, 0.3),
                Sound::new(SquareWave, 55.0, 0.3),
            ]
        ),
        Song::new("white noise",
            vec![
                Sound::new(WhiteNoise, 0.0, f32::MAX),
            ]
        ),
        Song::new("red noise",
            vec![
                Sound::new(RedNoise{filter1: Mutex::new(0.0), filter2: Mutex::new(0.0)}, 0.0, f32::MAX),
            ]
        ),
        Song::new("René's masterpiece",
            vec![
                Sound::new(SineWave, 68.0, 0.5),
                Sound::new(SquareWave, 105.0, 0.5),
                Sound::new(RedNoise{filter1: Mutex::new(0.0), filter2: Mutex::new(0.0)}, 300.0, 0.3),
                Sound::new(WhiteNoise, 300.0, 0.3),
                Sound::new(SawWave, 160.0, 0.5),
                Sound::new(SineWave, 210.0, 0.5),
                Sound::new(SquareWave, 290.0, 0.5),
                Sound::new(RedNoise{filter1: Mutex::new(0.0), filter2: Mutex::new(0.0)}, 290.0, 0.3),
                Sound::new(WhiteNoise, 290.0, 0.3),
                Sound::new(SawWave, 360.0, 0.5),
                Sound::new(SineWave, 450.0, 0.5),
                Sound::new(SquareWave, 61.0, 1.5),
            ]
        ),
        Song::new("THE JASON THE GOAT's masterpiece",
            vec![
                Sound::new(SineWave, 105.0, 0.1),
                Sound::new(SineWave, 150.0, 0.1),
            ]
        ),

    ]);

    // variables
    // all variables with audio_ prefix are clones of another variables designed to be moved to the audio thread
    // so that they can be accessed from outside

    let current_song_index = Arc::new(AtomicUsize::new(0));
    let current_sound_index = Arc::new(AtomicUsize::new(0));

    let audio_songs = Arc::clone(&songs);
    let audio_current_song_index = Arc::clone(&current_song_index);
    let audio_current_sound_index = Arc::clone(&current_sound_index);


    let volume = Arc::new(AtomicI32::new(30));
    let audio_volume = Arc::clone(&volume);


    let mut current_audio_frame = 0.0;
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for chunk in data.chunks_mut(channels) {

                // You can not use ARC's as normal variables, because they are shared between threads.
                // To read the variable .load is used, to write to the variable .store is used

                let current_idx = audio_current_song_index.load(Ordering::Relaxed);
                let song_sounds = &audio_songs[current_idx].sounds;
                let current_sound_index = audio_current_sound_index.load(Ordering::Relaxed);

                //
                if current_sound_index < song_sounds.len() {
                    let current_sound = &song_sounds[current_sound_index];
                    phase = (phase + (current_sound.frequency() / sample_rate)) % 1.0;
                    let value = current_sound.get_value(phase) * (audio_volume.load(Ordering::Relaxed) as f32 / 100.0);

                    current_audio_frame += 1 as f32;
                    if current_audio_frame/sample_rate > current_sound.duration() {
                        audio_current_sound_index.store(current_sound_index+1, Ordering::Relaxed);
                        current_audio_frame = 0.0;
                    }

                    for sample in chunk {
                        *sample = value;
                    }
                } else {
                    audio_current_sound_index.store(0, Ordering::Relaxed);
                }
            }
        },

        |_err| { /* handle error */ },
        None,
    ).unwrap();

    stream.play().unwrap();
    println!("type help if you don't know what to do");
    // My cli 
    // In addition to the while for and other loops rust has a loop loop, which is just a while (true) loop
    loop {
        let mut input = String::new();
        print!(">>> ");
        // When using print! instead of println! to get text on the same line, just will not output the string immediately
        // instead it has to be flushed manualy
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut input).expect("ERROR");
        let command = input.trim();
        let commands: Vec<&str> = command.split_ascii_whitespace().collect();
        if command.len() < 1 {
            println!("Empty command")
        } else {
            match commands[0].trim() {
                "quit" | "exit" | "e" | "q" => break,
                "help" | "h" => list_commands(),
                "list" | "l" => list_songs(&songs),
                "play" | "p" => set_song(&commands, &current_song_index, &current_sound_index, &songs),
                "stop" | "s" => set_song(&["stop", "0"], &current_song_index, &current_sound_index, &songs),
                "volume" | "vol" | "v" => set_volume(&commands, &volume),
                _ => println!("What was that?")
            }
        }
    }
    println!("It was fun while it lasted, goodbye friend!");
}

fn list_songs(songs: &[Song]) {
    for (i, song) in songs.iter().enumerate() {
        println!("[{i}]: {}", song.name);
    }
}

fn set_volume(commands: &[&str], volume: &AtomicI32) {
    if commands.len() < 2 {
        println!("No argument supplied; propper ussage: volume {{value}}; for example: volume 50")
    } else if let Ok(number) = commands[1].parse::<i32>(){
        volume.store(number, Ordering::Relaxed);
    } else {
        println!("{} is not a valid integer", commands[1]);
    }
}

fn set_song(commands: &[&str], current_song_index: &Arc<AtomicUsize>, current_sound_index: &Arc<AtomicUsize>, songs: &[Song]) {
    if commands.len() < 2 {
        println!("No argument supplied; propper ussage: play {{value}}; for example: play 1")
    } else if let Ok(number) = commands[1].parse::<usize>() {
        // if let Ok(value1) = value2 is a simplified version of try and catch with resourses
        if songs.len() > number {
            current_song_index.store(number, Ordering::Relaxed);
            current_sound_index.store(0, Ordering::Relaxed);
        } else {
            println!("Song out of bounds");
        }
    } else {
        println!("{} is not a valid positive number", commands[1]);
    }
}

fn list_commands() {
    println!("
    help -> show this list
    list -> list all songs
    play {{song number}} -> select the song to play
    stop -> stop playing
    volume {{value}} -> set the volume; 0 is min; 100 is max; you can go above 100 but the sound will be distorted
    ");
}
