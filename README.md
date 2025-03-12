# Run (mac os x)

`ping --apple-time 8.8.8.8 | cargo run`

or a combination of 

`ping --apple-time 8.8.8.8 | tee -a $(date +"%F").out` and `cat -f $(date +"%F").out | cargo run`