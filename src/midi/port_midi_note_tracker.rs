use pm::OutputPort;
use pm::types::Result;
use midi::{TypedMidiMessage, MidiSink, MidiNoteTracker};
use hardcode::*;

pub struct PortMidiNoteTracker {
    out_port: OutputPort,
    notes: [[bool; 128]; 16],
}

impl PortMidiNoteTracker {
    pub fn new(out_port: OutputPort) -> PortMidiNoteTracker {
        PortMidiNoteTracker {
            out_port: out_port,
            notes: [[false; 128]; 16],
        }
    }
}

impl MidiNoteTracker for PortMidiNoteTracker {
    fn close_opened_notes(&mut self) {
        for channel in 0..AMOUNT_OF_MIDI_CHANNELS {
            for key in 0..AMOUNT_OF_MIDI_KEYS {
                if self.notes[channel][key] {
                    self.out_port.write_message(TypedMidiMessage::NoteOff {
                        channel: channel as u8,
                        key: key as u8,
                        velocity: 0,
                    }).unwrap();
                }
            }
        }
    }
}

impl MidiSink for PortMidiNoteTracker {
    // FIXME(#172): don't use Result from the PortMidi library. Make the method more abstract.
    fn feed(&mut self, midi_message: TypedMidiMessage) -> Result<()> {
        match midi_message {
            TypedMidiMessage::NoteOn { channel, key, .. } =>
                self.notes[channel as usize][key as usize] = true,
            TypedMidiMessage::NoteOff { channel, key, .. } =>
                self.notes[channel as usize][key as usize] = false,
            _ => (),
        }

        self.out_port.write_message(midi_message)
    }
}
