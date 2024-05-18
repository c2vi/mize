use cbor::Cbor as CborValue;

#[derive(Debug, Clone)]
pub struct MizeMessage {
    value: CborValue,
}
