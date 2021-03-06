use pm::types::Result;
use midi::{TypedMidiMessage, MidiSink, MidiNoteTracker};

// TODO(#225): find a way to get rid of the DummyMidiNoteTracker warn
pub struct DummyMidiNoteTracker;

impl MidiNoteTracker for DummyMidiNoteTracker {
    fn close_opened_notes(&mut self) {}
}

impl MidiSink for DummyMidiNoteTracker {
    fn feed(&mut self, _: TypedMidiMessage) -> Result<()> {
        Ok(())
    }
}
