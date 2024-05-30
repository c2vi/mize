use ciborium::Value as CborValue;

#[derive(Debug, Clone)]
pub struct MizeMessage {
    value: CborValue,
}
