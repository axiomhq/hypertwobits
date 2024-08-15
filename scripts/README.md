This scripts originate from [hyperfine](https://github.com/sharkdp/hyperfine/tree/master/scripts) and are used to plot accuracy tests.

### Example:

```bash
cargo run --release --example accuracy -- 1000
for items in 100 1000 10000 100000 all; do
    for body in shakespeare ulysses war_and_peace combined; do
        python scripts/plot_whisker.py stats/${body}-${items}.json -o stats/${body}-${items}.png
    done
done

```

### Pre-requisites

To make these scripts work, you will need to install `numpy`, `matplotlib` and `scipy`. Install them via
your package manager or `pip`:

```bash
pip install numpy matplotlib scipy  # pip3, if you are using python3
```
