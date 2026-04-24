# Controller Test Backend

A backend service for testing controllers. It uses `inputtino` and WebSockets (`tokio-tungstenite`) to handle controller input and testing.

## Features

- Built with Rust and Tokio
- WebSocket communication
- Controller emulation via `inputtino`
- Local network discovery with QR Code generation
- Real-time controller state updates
- Configurable button mapping

## Usage

1. Ensure you have Rust and Cargo installed.
2. Clone the repository and navigate to the project directory.
3. Run `cargo run --release` to start the server.
4. Connect your controller client to the displayed QR code or the printed IP address and port (default 7879).
5. Interact with the controller; the backend will forward inputs to the emulated Xbox One pad.


## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
