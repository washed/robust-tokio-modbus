# robust-tokio-modbus

Builds on the excellent [tokio-modbus](https://crates.io/crates/tokio-modbus) crate and adds retries for reads/writes and establishing connections as a client. Mostly just a thin wrapper which makes it easier to build robust modbus clients that reconnect after server or network outages.

I have built this for my own personal projects and don't claim that this library is fit for any purpose.
