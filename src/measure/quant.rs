use std::ops::{Add, Mul, Sub, Rem, Div};

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

// FIXME(#125): Autoderive arithmetic operations for Quant
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Quant(pub u32);

impl Encodable for Quant {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        let &Quant(this) = self;
        s.emit_u32(this)
    }
}

impl Decodable for Quant {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_u32().map(|q| Quant(q))
    }
}

impl Add for Quant {
    type Output = Quant;

    fn add(self, Quant(other): Quant) -> Quant {
        let Quant(this) = self;
        Quant(this + other)
    }
}

impl Sub for Quant {
    type Output = Quant;

    fn sub(self, Quant(other): Quant) -> Quant {
        let Quant(this) = self;
        Quant(this - other)
    }
}

impl Mul for Quant {
    type Output = Quant;

    fn mul(self, Quant(other): Quant) -> Quant {
        let Quant(this) = self;
        Quant(this * other)
    }
}

impl Div for Quant {
    type Output = Quant;

    fn div(self, Quant(other): Quant) -> Quant {
        let Quant(this) = self;
        Quant(this / other)
    }
}

impl Rem for Quant {
    type Output = Quant;

    fn rem(self, Quant(other): Quant) -> Quant {
        let Quant(this) = self;
        Quant(this % other)
    }
}
