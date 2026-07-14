//! DNS query construction: random query names and raw wire-format packets.
use rand::RngExt;
use rand::seq::IndexedRandom;

/// Record types to vary, as (qtype value, name).
pub const QTYPES: &[(u16, &str)] = &[
    (1, "A"),
    (28, "AAAA"),
    (16, "TXT"),
    (15, "MX"),
    (2, "NS"),
    (5, "CNAME"),
];

fn random_label(rng: &mut impl RngExt, len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    (0..len)
        .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
        .collect()
}

// forcing known txt records to be txt records
const FORCED_TXT_PREFIXES: &[&str] = &["_domainkey", "_dmarc"];

fn is_forced_txt(domain: &str) -> bool {
    FORCED_TXT_PREFIXES
        .iter()
        .any(|prefix| domain.contains(prefix))
}

fn is_a(domain: &str) -> bool {
    domain.ends_with("in-addr.arpa")
}

fn is_aaaa(domain: &str) -> bool {
    domain.ends_with("ip6.arpa")
}

pub fn pick_query(
    rng: &mut impl RngExt,
    pool: &[String],
    random_prefix: bool,
) -> (String, u16, &'static str) {
    let base = pool.choose(rng).expect("domain pool must not be empty");

    if is_forced_txt(base) {
        return (base.clone(), 16, "TXT");
    }

    if is_a(base) {
        return (base.clone(), 1, "A");
    }

    if is_aaaa(base) {
        return (base.clone(), 28, "AAAA");
    }

    let qname = if random_prefix {
        if rng.random_bool(0.5) {
            base.clone()
        } else {
            let label_len = rng.random_range(3..12);
            format!("{}.{}", random_label(rng, label_len), base)
        }
    } else {
        base.clone()
    };

    let (qtype, qtype_name) = *QTYPES.choose(rng).unwrap();
    (qname, qtype, qtype_name)
}

/// Encodes a domain name into DNS wire format (length-prefixed labels).
fn encode_qname(name: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for label in name.split('.') {
        out.push(label.len() as u8);
        out.extend_from_slice(label.as_bytes());
    }
    out.push(0); // root terminator
    out
}

/// Builds a minimal valid DNS query packet for the given name/qtype.
pub fn build_query(rng: &mut impl RngExt, qname: &str, qtype: u16) -> Vec<u8> {
    let mut packet = Vec::with_capacity(64);

    let id: u16 = rng.random();
    packet.extend_from_slice(&id.to_be_bytes());

    // Flags: standard query, recursion
    packet.extend_from_slice(&0x0100u16.to_be_bytes());

    // QDCOUNT = 1, ANCOUNT/NSCOUNT/ARCOUNT = 0
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet.extend_from_slice(&0u16.to_be_bytes());
    packet.extend_from_slice(&0u16.to_be_bytes());
    packet.extend_from_slice(&0u16.to_be_bytes());

    // Question section
    packet.extend_from_slice(&encode_qname(qname));
    packet.extend_from_slice(&qtype.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes()); // QCLASS = IN

    packet
}

/// Jittered delay
pub fn next_delay(rng: &mut impl RngExt, min_ms: u64, max_ms: u64) -> std::time::Duration {
    let span = max_ms.saturating_sub(min_ms).max(1);
    // Skew toward shorter delays using a squared random factor.
    let t: f64 = rng.random::<f64>().powi(2);
    let ms = min_ms + (t * span as f64) as u64;
    std::time::Duration::from_millis(ms)
}
