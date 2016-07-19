#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Quant(u32);

#[derive(Debug, Clone)]
pub struct Measure {
    pub tempo_bpm: u32,
    pub measure_size_bpm: u32,
    pub quantation_level: u32,
}

impl Measure {
    pub fn timestamp_to_quant(&self, timestamp: u32) -> Quant {
        Quant((timestamp + self.quant_size_millis() / 2) / self.quant_size_millis())
    }

    pub fn quant_to_timestamp(&self, Quant(quant_value): Quant) -> u32 {
        quant_value * self.quant_size_millis()
    }

    pub fn measure_size_millis(&self) -> u32 {
        self.beat_size_millis() * self.measure_size_bpm
    }

    pub fn beat_size_millis(&self) -> u32 {
        (60000.0 / self.tempo_bpm as f32) as u32
    }

    pub fn quant_size_millis(&self) -> u32 {
        let mut result = self.measure_size_millis() as f32;

        for _ in 0..self.quantation_level {
            result /= self.measure_size_bpm as f32
        }

        result as u32
    }
}

#[cfg(test)]
mod tests {
    use super::{Measure, Quant};

    const TEMPO_BPM: u32 = 120;
    const MEASURE_SIZE_BPM: u32 = 4;
    const QUANTATION_LEVEL: u32 = 2;

    const MEASURE_SIZE_MILLIS: u32 =  2000;
    const BEAT_SIZE_MILLIS: u32 =  500;
    const QUANT_SIZE_MILLIS: u32 =  125;


    #[test]
    fn test_measure_new() {
        let measure = Measure {
            tempo_bpm: TEMPO_BPM,
            measure_size_bpm: MEASURE_SIZE_BPM,
            quantation_level: QUANTATION_LEVEL,
        };

        assert_eq!(TEMPO_BPM, measure.tempo_bpm);
        assert_eq!(MEASURE_SIZE_BPM, measure.measure_size_bpm);
        assert_eq!(QUANTATION_LEVEL, measure.quantation_level);

        assert_eq!(MEASURE_SIZE_MILLIS, measure.measure_size_millis());
        assert_eq!(BEAT_SIZE_MILLIS, measure.beat_size_millis());
        assert_eq!(QUANT_SIZE_MILLIS, measure.quant_size_millis());
    }

    #[test]
    fn test_measure_update() {
        let mut measure = Measure {
            tempo_bpm: TEMPO_BPM,
            measure_size_bpm: MEASURE_SIZE_BPM,
            quantation_level: QUANTATION_LEVEL
        };

        assert_eq!(MEASURE_SIZE_MILLIS, measure.measure_size_millis());
        assert_eq!(BEAT_SIZE_MILLIS, measure.beat_size_millis());
        assert_eq!(QUANT_SIZE_MILLIS, measure.quant_size_millis());

        measure = Measure { tempo_bpm: TEMPO_BPM + 40, .. measure };

        assert_eq!(1500, measure.measure_size_millis());
        assert_eq!(375, measure.beat_size_millis());
        assert_eq!(93, measure.quant_size_millis());
    }

    #[test]
    fn test_timestamp_quant_conversion() {
        let measure = Measure {
            tempo_bpm: TEMPO_BPM,
            measure_size_bpm: MEASURE_SIZE_BPM,
            quantation_level: QUANTATION_LEVEL,
        };

        // timestamp to quant
        assert_eq!(Quant(0), measure.timestamp_to_quant(0));
        assert_eq!(Quant(1), measure.timestamp_to_quant(measure.quant_size_millis()));
        assert_eq!(Quant(0), measure.timestamp_to_quant(measure.quant_size_millis() / 2 - 1));
        assert_eq!(Quant(1), measure.timestamp_to_quant(measure.quant_size_millis() / 2 + 1));

        // quant to timestamp
        assert_eq!(5 * measure.quant_size_millis(), measure.quant_to_timestamp(Quant(5)));
    }
}
