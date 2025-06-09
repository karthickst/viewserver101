# viewserver101

basic example of how you can create an in-memory view server in Rust with RocksDB, allowing clients to create filtered custom views and receive real-time notifications for changes to the views. This example uses the tokio crate for async programming, rocksdb crate for interacting with RocksDB, and tokio-tungstenite crate for WebSocket support.

