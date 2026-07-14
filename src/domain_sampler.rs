//! Domain pool handling: streaming reservoir-sampled loading from a user-supplied CSV file.

use rand::{RngExt};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

fn sample_domains(
    path: &PathBuf,
    sample_size: usize,
    rng: &mut impl RngExt,
) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut reservoir: Vec<String> = Vec::with_capacity(sample_size);
    let mut seen: usize = 0;

    for (line_no, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let field = line.rsplit(',').next().unwrap_or("").trim();
        if field.is_empty() {
            eprintln!("noisy: skipping malformed line {}: {:?}", line_no + 1, line);
            continue;
        }

        let looks_like_domain = field
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_');
        if !looks_like_domain {
            eprintln!(
                "noisy: skipping invalid domain on line {}: {:?}",
                line_no + 1,
                field
            );
            continue;
        }

        let domain = field.to_ascii_lowercase();

        // Reservoir sampling (Algorithm R): the first `sample_size` valid
        // domains fill the reservoir directly. After that, each new item
        // has a `sample_size / seen` chance of bumping out a random
        // existing slot — this guarantees a uniform random sample of size
        // `sample_size` over the whole stream without knowing its length
        // in advance or storing more than `sample_size` items at once.
        if reservoir.len() < sample_size {
            reservoir.push(domain);
        } else {
            let j = rng.random_range(0..=seen);
            if j < sample_size {
                reservoir[j] = domain;
            }
        }
        seen += 1;
    }

    Ok(reservoir)
}

pub fn load_pool(
    domains: &PathBuf,
    sample_size: usize,
    verbose: bool,
    rng: &mut impl RngExt,
) -> io::Result<Vec<String>> {
    let sampled = sample_domains(domains, sample_size, rng)?;

    if sampled.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{:?} contained no usable domains", domains),
        ));
    }

    if verbose {
        println!("sampled {} domains from {:?}", sampled.len(), domains);
    }

    Ok(sampled)
}
