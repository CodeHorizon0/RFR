Rust Functions Runtime aka CF Worker/AWS Functions clone

Features:
- JS functions stored as separate files in ./js
- automatic loading on startup
- HTTP proxy layer
- idle server waiting for requests
- request routing to JS workers
- isolated QuickJS contexts

Run:
cargo run

Call:
POST http://localhost:8080/functions/hello
