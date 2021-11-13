# NRF52840 Demo

To test with watchdog groomers:
```
cargo run 
```

To test without watchdog groomers (a watchdog reset will occur):
```
cargo run --no-default-features
```

To test firmware with a heap:
```
cargo +nightly run --features heap
```

To view the size of the demo firmware:
```
cargo size --bin demo --release
```
