//! dns-noise: sends randomized decoy DNS queries at randomized intervals
//! to obscure the timing/content pattern of real DNS traffic.

mod dns;
mod domain_sampler;

use clap::Parser;
use rand;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(about = "Generate decoy DNS queries")]
struct Args {
    /// resolver to use
    #[arg(short, long, default_value = "127.0.0.1:53")]
    resolver: String,

    // minimum delay between each queries
    #[arg(long, default_value_t = 5000)]
    min_delay: u64,

    // maximum delay between each queries
    #[arg(long, default_value_t = 10000)]
    max_delay: u64,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    #[arg(short = 'd', long)]
    domains: PathBuf,

    #[arg(long, default_value_t = 50)]
    sample_size: usize,

    #[arg(long, default_value_t = 0)]
    resample_interval: u64,

    #[arg(long, default_value_t = false)]
    random_prefix: bool
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut rng = rand::rng();

    let mut pool: Vec<String> =
        domain_sampler::load_pool(&args.domains, args.sample_size, args.verbose, &mut rng)?;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_write_timeout(Some(Duration::from_secs(2)))?;

    let mut sent: u64 = 0;
    loop {
        if args.resample_interval != 0 && sent != 0 && sent % args.resample_interval == 0 {
            match domain_sampler::load_pool(&args.domains, args.sample_size, args.verbose, &mut rng)
            {
                Ok(fresh) => {
                    if args.verbose {
                        println!("resampled domain pool ({} domains)", fresh.len());
                    }
                    pool = fresh;
                }
                Err(e) => {
                    if args.verbose {
                        eprintln!("resample failed, keeping current pool: {e}");
                    }
                }
            }
        }

        let (qname, qtype, qtype_name) = dns::pick_query(&mut rng, &pool, args.random_prefix);
        let packet = dns::build_query(&mut rng, &qname, qtype);

        match socket.send_to(&packet, &args.resolver) {
            Ok(_) => {
                if args.verbose {
                    println!("sent {qtype_name} query for {qname}");
                }
            }
            Err(e) => {
                if args.verbose {
                    eprintln!("send failed: {e}");
                }
            }
        }

        sent += 1;
        let delay = dns::next_delay(&mut rng, args.min_delay, args.max_delay);
        std::thread::sleep(delay);
    }
}
