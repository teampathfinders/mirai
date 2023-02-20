# Nova
Lightweight dedicated server software for Minecraft: Bedrock Edition.

### Building
The tools required the build the server are
- Rust 1.64.0+
- CMake 3.0+

After cloning the repository, make sure to run `git submodule update --init` to download the required dependencies. After this, run `cargo build --release` to produce an optimised executable in the `target/release` folder. Alternatively, you can execute `cargo run --release` to immediately run the server as well.

### Usage
In case you want to connect to the server you are hosting locally, make sure to run the following command in an administrator Powershell window.
`CheckNetIsolation.exe LoopbackExempt -a -p=S-1-15-2-1958404141-86561845-1752920682-3514627264-368642714-62675701-733520436` (as shown in the bedrock_server_how_to.html bundled with the official dedicated server.). This will allow Minecraft to access local servers.
If you do not plan on connecting to a server on the same PC, this is not necessary.

To connect to the server in Minecraft, add a custom server with IP address 127.0.0.1 and port 19132. 

This project is licensed under the Apache 2.0 license.
