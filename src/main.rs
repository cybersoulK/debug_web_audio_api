



use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::Duration;

use web_audio_api::context::{AudioContextOptions, AudioContext, BaseAudioContext};
use web_audio_api::AudioBuffer;
use web_audio_api::node::{AudioBufferSourceNode, GainNode, AudioNode, AudioBufferSourceOptions, GainOptions, AudioScheduledSourceNode};



pub struct AudioState {
    context: AudioContext,
    buffer: AudioBuffer,
    active_sounds: Vec<SoundHandle>,
}

impl AudioState {
    pub fn new() -> Self {
        
        let options = AudioContextOptions::default();
        let context = AudioContext::new(options);
           

        let data = include_bytes!("sound.wav");
        let cursor = std::io::Cursor::new(data);
        let buffer = context.decode_audio_data_sync(cursor).unwrap();


        use web_audio_api::AudioRenderCapacityOptions;

        let render_capacity = context.render_capacity();

        render_capacity.start(AudioRenderCapacityOptions::default());
        render_capacity.set_onupdate(|capacity| {
            println!("{:?}", capacity);
        });


        Self {  
            context,
            buffer,
            active_sounds: Vec::new(),
        }
    }


    pub fn update(&mut self) {

        self.active_sounds.retain(|sound| sound.is_playing());

        //can check that the active_sounds has no cache
        //println!("{}", self.active_sounds.len());
    }


    pub fn play(&mut self) -> SoundHandle {
  
        let sound = Sound::create(&self.context, self.buffer.clone());
        self.active_sounds.push(sound.clone());

        sound
    }
}




#[derive(PartialEq)]
pub enum SoundState {
    Playing = 0,
    Stopped = 1,
    Ended = 2,
}


pub struct Sound {
    source_node: AudioBufferSourceNode,
    gain_node: GainNode,

    state: Arc<AtomicUsize>,
}

#[derive(Clone)]
pub struct SoundHandle(Arc<Mutex<Sound>>);


impl Sound {
    pub fn create(context: &AudioContext, buffer: AudioBuffer) -> SoundHandle {

        let options = AudioBufferSourceOptions::default();
        let mut source_node = AudioBufferSourceNode::new(context, options);
        source_node.set_buffer(buffer);

        let mut options = GainOptions::default();
        options.gain = 0.3;
        let gain_node = GainNode::new(context, options);


        source_node.connect(&gain_node);
        gain_node.connect(&context.destination());


        let state = Arc::new(AtomicUsize::new(SoundState::Playing as usize));

        {
            let state = state.clone();

            source_node.set_onended(move |_| {
                state.store(SoundState::Ended as usize, Ordering::SeqCst);
            });
        }

        source_node.start();


        let sound = Sound {
            source_node: source_node,
            gain_node: gain_node,

            state,
        };

        SoundHandle(Arc::new(Mutex::new(sound)))
    }
}

impl Drop for Sound {
    fn drop(&mut self) {
        self.source_node.disconnect();
        self.gain_node.disconnect();
    }
}


impl SoundHandle {

    pub fn is_playing(&self) -> bool {
        let sound = self.0.lock().unwrap();
        sound.state.load(Ordering::SeqCst) == SoundState::Playing as usize
    }
}



fn main() {

    let mut audio_state = AudioState::new();

    loop {
        sleep(Duration::from_secs_f32(0.02));

        audio_state.play();

        audio_state.update();
    }
}