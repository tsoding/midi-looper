use midi::{AbsMidiEvent, TypedMidiMessage, MidiNoteTracker, MidiSink};
use config::*;
use num::integer::lcm;

use traits::{Updatable, Renderable};
use graphicsprimitives::CircleRenderer;
use measure::*;

use sdl2::render::Renderer;
use sdl2::pixels::Color;
use sdl2::rect::Point;

pub mod sample;

use self::sample::Sample;

#[derive(PartialEq)]
pub enum State {
    Recording,
    Looping,
    Pause,
}

pub struct Looper {
    pub state: State,
    pub next_state: Option<State>,

    pub composition: Vec<Sample>,
    pub record_buffer: Vec<AbsMidiEvent>,


    pub note_tracker: MidiNoteTracker,

    time_cursor: u32,
    amount_of_measures: u32,

    pub measure: Measure,
}

impl Updatable for Looper {
    fn update(&mut self, delta_time: u32) {
        if self.state != State::Pause {
            let current_measure_bar = self.measure.timestamp_to_measure(self.time_cursor);
            let current_quant = self.measure.timestamp_to_quant(self.time_cursor);

            let next_time_cursor = self.time_cursor + delta_time;
            let next_measure_bar = self.measure.timestamp_to_measure(next_time_cursor);
            let next_quant = self.measure.timestamp_to_quant(next_time_cursor);

            if current_measure_bar < next_measure_bar {
                self.on_measure_bar();
            }

            if current_quant < next_quant {
                for sample in &mut self.composition {
                    // FIXME(): make Quants range iterable
                    let Quant(start) = current_quant;
                    let Quant(end) = next_quant;
                    for q in start + 1..end + 1 {
                        sample.replay_quant(Quant(q), &mut self.note_tracker);
                    }
                }
            }

            self.time_cursor = next_time_cursor;
        }
    }
}

impl Renderable for Looper {
    fn render(&self, renderer: &mut Renderer) {
        let window_width = renderer.viewport().width();
        let window_height = renderer.viewport().height();
        let measure_size_millis = self.measure.measure_size_millis();
        let beat_size_millis = self.measure.beat_size_millis();

        for sample in &self.composition {
            sample.render(self.measure.timestamp_to_measure(self.time_cursor), renderer);
        }

        let draw_time_cursor = |time_cursor: u32, renderer: &mut Renderer| {
            let x = ((time_cursor as f32) /
                     measure_size_millis as f32 *
                     (window_width as f32 - 10.0) + 5.0) as i32;
            renderer.draw_line(Point::from((x, 0)),
                               Point::from((x, window_height as i32))).unwrap();
        };

        // Time Cursor
        renderer.set_draw_color(Color::RGB(255, 255, 255));
        draw_time_cursor(self.time_cursor % measure_size_millis, renderer);

        // Measure Beats
        for i in 0 .. self.measure.measure_size_bpm {
            renderer.set_draw_color(Color::RGB(50, 50, 50));
            draw_time_cursor(i * beat_size_millis, renderer);
        }

        { // Circle
            let r = 15;
            let p = 25;
            let x = window_width as i32 - r - 2 * p;
            let y = r + p;
            renderer.set_draw_color(Color::RGB(255, 0, 0));

            if let State::Recording = self.state {
                renderer.fill_circle(x, y, r);
            } else {
                renderer.draw_circle(x, y, r);
            }
        }
    }
}

impl Looper {
    pub fn new(note_tracker: MidiNoteTracker) -> Looper {
        let mut looper = Looper {
            state: State::Looping,
            next_state: None,
            composition: Vec::new(),
            record_buffer: Vec::new(),
            note_tracker: note_tracker,
            amount_of_measures: 1,
            time_cursor: 0,
            measure: Measure {
                tempo_bpm: DEFAULT_TEMPO_BPM,
                measure_size_bpm: DEFAULT_MEASURE_SIZE_BPM,
                quantation_level: DEFAULT_QUANTATION_LEVEL,
            },
        };
        looper.reset();
        looper
    }

    pub fn reset(&mut self) {
        let beats = self.make_metronome();

        self.state = State::Looping;
        self.composition.clear();
        self.composition.push(beats);
        self.record_buffer.clear();

        self.amount_of_measures = 1;

        self.note_tracker.close_opened_notes();
    }

    pub fn toggle_recording(&mut self) {
        match self.state {
            State::Recording => {
                self.next_state = Some(State::Looping);
            }

            State::Looping => {
                self.state = State::Recording;
                self.record_buffer.clear();
            }

            _ => (),
        }

    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            State::Looping => {
                self.state = State::Pause;
                self.note_tracker.close_opened_notes();
            },
            State::Pause => self.state = State::Looping,
            _ => (),
        }
    }

    pub fn undo_last_recording(&mut self) {
        if let State::Recording = self.state {
            self.record_buffer.clear();
        } else {
            if self.composition.len() > 1 {
                self.composition.pop();
                self.amount_of_measures = 1;
                for sample in &self.composition {
                    self.amount_of_measures = lcm(self.amount_of_measures,
                                                  sample.amount_of_measures);
                }
                self.note_tracker.close_opened_notes();
            }
        }
    }

    pub fn on_measure_bar(&mut self) {
        if let Some(state) = self.next_state.take() {
            self.state = state;

            match self.state {
                State::Looping => {
                    self.normalize_record_buffer();
                    let sample = Sample::new(&self.record_buffer, &self.measure);
                    self.amount_of_measures = lcm(self.amount_of_measures, sample.amount_of_measures);
                    self.composition.push(sample);
                },

                _ => ()
            }
        }
    }

    pub fn on_midi_event(&mut self, event: &AbsMidiEvent) {
        if let State::Recording = self.state {
            self.record_buffer.push(event.clone());
        }

        self.note_tracker.feed(event.message).unwrap();
    }

    pub fn update_tempo_bpm(&mut self, tempo_bpm: u32) {
        let new_measure = Measure { tempo_bpm: tempo_bpm, .. self.measure };

        // FIXME(): scale absolute cursor properly
        // self.measure_time_cursor =
        //     self.measure.scale_time_cursor(&new_measure,
        //                                    self.amount_of_measures,
        //                                    self.measure_time_cursor);

        self.time_cursor =
            self.measure.scale_time_cursor(&new_measure,
                                           self.amount_of_measures,
                                           self.time_cursor % (self.amount_of_measures * self.measure.measure_size_millis()));

        for sample in self.composition.iter_mut() {
            sample.update_measure(&new_measure)
        }

        self.measure = new_measure;
    }

    fn make_metronome(&self) -> Sample {
        let beat_size_millis = self.measure.beat_size_millis();

        let mut buffer = Vec::new();

        for i in 0..self.measure.measure_size_bpm {
            buffer.push(AbsMidiEvent {
                message: TypedMidiMessage::NoteOn {
                    channel: CONTROL_CHANNEL_NUMBER,
                    key: BEAT_KEY_NUMBER,
                    velocity: if i == 0 { BEAT_ACCENT_VELOCITY } else { BEAT_VELOCITY },
                },
                timestamp: i * beat_size_millis,
            });

            buffer.push(AbsMidiEvent {
                message: TypedMidiMessage::NoteOff {
                    channel: CONTROL_CHANNEL_NUMBER,
                    key: BEAT_KEY_NUMBER,
                    velocity: 0,
                },
                timestamp: i * beat_size_millis + 1,
            })
        }

        Sample::new(&buffer, &self.measure)
    }

    fn normalize_record_buffer(&mut self) {
        if !self.record_buffer.is_empty() {
            let t0 = self.record_buffer[0].timestamp;

            for event in self.record_buffer.iter_mut() {
                event.timestamp -= t0;
            }
        }
    }
}
