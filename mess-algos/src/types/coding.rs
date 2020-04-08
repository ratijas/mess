//! - `Coding`
//!     * `Hamming = Coding`
//!     * `Parity = Coding`
//!     * `R3 = Coding`
//!     * `R5 = Coding`

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Coding {
    Hamming,
    Parity,
    R3,
    R5,
}

impl Into<Box<::coding::Coding>> for Coding {
    fn into(self) -> Box<::coding::Coding> {
        match self {
            Coding::Hamming => Box::new(::coding::hamming::Hamming),
            Coding::Parity => Box::new(::coding::parity::Parity),
            Coding::R3 => Box::new(::coding::repetition3::Repetition3),
            Coding::R5 => Box::new(::coding::repetition5::Repetition5),
        }
    }
}

impl From<::coding::hamming::Hamming> for Coding {
    fn from(_: ::coding::hamming::Hamming) -> Self {
        Coding::Hamming
    }
}

impl From<::coding::parity::Parity> for Coding {
    fn from(_: ::coding::parity::Parity) -> Self {
        Coding::Parity
    }
}

impl From<::coding::repetition3::Repetition3> for Coding {
    fn from(_: ::coding::repetition3::Repetition3) -> Self {
        Coding::R3
    }
}

impl From<::coding::repetition5::Repetition5> for Coding {
    fn from(_: ::coding::repetition5::Repetition5) -> Self {
        Coding::R5
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn ser() {
        let json = serde_json::to_string(&Coding::R3).unwrap();
        assert_eq!(r#""r3""#, json);
    }

    #[test]
    fn de() {
        let coding = serde_json::from_str(r#""parity""#).unwrap();
        match coding {
            Coding::Parity => assert!(true),
            _ => unreachable!(),
        }
    }
}