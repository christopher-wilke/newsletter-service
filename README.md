# newsletter-service
Newsletter Service written in Rust 

## Start
For a better dev and debugging experience, I recommend to use `cargo watch`.

```sh
$ cargo install cargo-watch 
```

You can then re-compile the solution by just saving your changes. The command below also makes sure to run the tests and clears the screen every time you save a file.

```sh
$ cargo watch -x run -x test -c
```