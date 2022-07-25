Some interesting parameters:

Introduce some stack mutation and higher mutation frequency for faster
evolution.

Start with a descendant.

cargo run --release -- run sample_code/descendant.apil --memory-mutation-amount 1 --processor-stack-mutation-amount 1 --mutation-frequency 10000 --dump true
