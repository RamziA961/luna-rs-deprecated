use html_escape::decode_html_entities as decode;

pub(crate) mod banish;
pub(crate) mod source_retriever;
pub(crate) mod summon;

pub(crate) use banish::banish;
pub(crate) use summon::summon;

pub(crate) fn decode_html_encoded_string(s: &String) -> String {
    decode(&s.clone()).to_string()
}

pub(crate) fn to_seconds(m: Option<u64>, s: Option<u64>) -> Option<u64> {
    let s = if m.is_some() {
        s.and_then(|v| (v < 60).then_some(v))
    } else {
        s
    };

    match (m, s) {
        (Some(m), Some(s)) => Some(m * 60 + s),
        (Some(_), None) => None,
        (None, Some(s)) => Some(s),
        (None, None) => None,
    }
}
