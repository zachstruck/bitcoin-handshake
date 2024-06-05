# Bitcoin Handshake Test

This simple program performs a handshake with a Bitcoin node.  It sends a version message, then waits to receive a version message as well as a verack message, and finally acknowledges this by sending its own verack message in return.

## How to Run

### Command to Execute Program

```sh
cargo r --release -- --ip-address <IP_ADDRESS> --port <PORT>
```

### Command to Display Help

There is a basic help:

```sh
cargo r --release -- --help
```

## How to Verify Handshake

### Example Command

This Bitcoin node seemed reliable during my testing:

```sh
cargo r --release -- --ip-address 65.109.34.157
```

The program will default to port 8333.

### Verification Message

If successfully connected and run, the program will print:

```text
successful handshake
```