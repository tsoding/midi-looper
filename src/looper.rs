use pm::types::MidiEvent;
use pm::OutputPort;

use sdl2::TimerSubsystem;

#[derive(PartialEq)]
pub enum State {
    Recording,
    Looping,
    Quit,
}

pub struct Looper {
    pub state: State,
    pub record_buffer: Vec<MidiEvent>,
    pub next_event: usize,
    pub dt: u32,
}

impl Default for Looper {
    fn default() -> Looper {
        Looper {
            state: State::Recording,
            record_buffer: Vec::new(),
            next_event: 0,
            dt: 0,
        }
    }
}

impl Looper {
    pub fn update(&mut self,
                  timer_subsystem: &mut TimerSubsystem,
                  out_port: &mut OutputPort) {
        if let State::Looping = self.state {
            if !self.record_buffer.is_empty() {
                let t = timer_subsystem.ticks() - self.dt;
                let event = &self.record_buffer[self.next_event];
                if t > event.timestamp {
                    out_port.write_message(event.message).unwrap();
                    self.next_event += 1;

                    if self.next_event >= self.record_buffer.len() {
                        self.dt = timer_subsystem.ticks();
                        self.next_event = 0;
                    }
                }
            }
        }
    }

    pub fn looping(&mut self, timer_subsystem: &mut TimerSubsystem) {
        self.state = State::Looping;
        if !self.record_buffer.is_empty() {
            self.dt = timer_subsystem.ticks();
            self.next_event = 0;
        }
    }

    pub fn on_midi_event(&mut self, event: &MidiEvent) {
        if let State::Recording = self.state {
            self.record_buffer.push(event.clone());
        }
    }
}