# Steadyum
**Steadyum** is a simple physics sandbox for experimenting with the Rapier physics engine as well as distributed physics.

## Standalone mode
**Steadyum** can be run standalone (without the distributed system feature):

```bash
cargo run --features "dim3" --release # Run the 3D version of Steadyum.
cargo run --features "dim2" --release # Run the 2D version of Steadyum.
```

## Distributed mode
In order to operate in the distributed simulation mode four steps are needed:
- Setup a local [Redis](https://redis.com/) server. It must be accessible locally behinde the port `6379`.
- Compile (without starting it) the runner: `cargo build --release -p steadyum-runner --features dim3`
- Start the partitionner the runner: `cargo build --release -p steadyum-partitionner --features dim3`
- Start the client: `cargo run --features "dim3" --release -- --distributed-physics`. For testing very large scene, it
  is recommended to disable some of the fancy graphics with the `--lower-graphics` option.
