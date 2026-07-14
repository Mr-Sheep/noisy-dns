# Noisy

A very dumb* tool that sends randomized decoy DNS queries at randomized intervals, hopefully can help obscure 
the timing/content pattern of real dns traffic.

works with Cisco's Umbrella Popularity List: [here](https://s3-us-west-1.amazonaws.com/umbrella-static/index.html)

```sh 
Generate decoy DNS queries

Usage: noisy-dns [OPTIONS] --domains <DOMAINS>

Options:
  -r, --resolver <RESOLVER>                    resolver to use [default: 127.0.0.1:53]
      --min-delay-ms <MIN_DELAY_MS>            [default: 5000]
      --max-delay-ms <MAX_DELAY_MS>            [default: 10000]
  -v, --verbose
  -d, --domains <DOMAINS>
      --sample-size <SAMPLE_SIZE>              [default: 50]
      --resample-interval <RESAMPLE_INTERVAL>  [default: 0]
  -h, --help                                   Print help
```

Proudly built with rust, Licensed GPL-V3
