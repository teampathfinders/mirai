![Inferno thumbnail](https://github.com/teampathfinders/inferno/blob/master/resources/thumb.png?raw=true)

Robust dedicated server software for Minecraft: Bedrock Edition built with safety in mind.

This project is still under development and not fit for production use.

## Running the project

### With Docker (recommended)
The recommended way to run Inferno is inside a Docker container. This prevents the hassle of setting up the required services manually. See the [Docker documentation](https://docs.docker.com/desktop/) for a guide on how to install Docker on your platform. Running the server is then as easy as running `docker compose up` in your terminal. Docker will then automatically download the required tools and run the server for you. When the setup finishes, you will be able to join the server at localhost:19132 (or whatever the configured address is). In case it is not working, make sure to check [loopback workaround](#loopback-workaround).

### Manual setup
In case you don't want to use the Docker image and instead want to set up the server manually you will first need to set up a [Redis instance](https://redis.io/docs/install/). This Redis instance will by default be running on port 6379. If you are using a different port make sure to set the `REDIS_PORT` environment variable before starting the server. Additionally, the `REDIS_HOST` variable can be used for instances on a different machine. 

The minimum supported Rust version required to compile the project is 1.72. Additionally, the `inferno-level` crate also requires at least CMake 3.13+ and a compiler capable of compiling C++11 code. This is used to build [LevelDB](https://github.com/teampathfinders/leveldb) from source.  
  
Minimum requirements:
- Rust 1.75
- CMake 3.13
- C++11 compliant compiler

After cloning the repository, make sure to run `git submodule update --init` to download the required Git dependencies. After this, run `cargo build --release` to produce an optimised executable in the `target/release` folder. Alternatively, you can execute `cargo run --release` to immediately run the server as well. If you're trying to join a server hosted on your own machine, make sure to check the [loopback workaround](#loopback-workaround) section.

### Configuration
Several environment variables can be used to modify the behaviour of the server.
* `REDIS_HOST`: Tells the server to connect to a Redis instance on a different machine. The value of this should be the address (without port) of the instance. If left empty, the default value is `localhost`. In case you are using the Docker image, the Redis address variables should be left unset as the Docker image will set them for you.
* `REDIS_PORT` - Sets the port the Redis instance is listening on. By default this is 6379, which is also the default for Redis.
* `LOG_LEVEL` - Defines the amount of logging the server will do. This can be set to `error`, `warn`, `info`, `debug`, `trace` or `off` to log the respective levels and the ones above that only. 

### Loopback workaround
In case you want to connect to the server you are hosting locally, make sure to run the following command in an administrator Powershell window. 
`CheckNetIsolation.exe LoopbackExempt -a -p=S-1-15-2-1958404141-86561845-1752920682-3514627264-368642714-62675701-733520436` (as shown in the bedrock_server_how_to.html bundled with the official dedicated server.). This will allow Minecraft to access local servers.

If you do not plan on connecting to a server on the same PC, this is not necessary.

#### Legal
This project is licensed under the Apache 2.0 license.
LevelDB is licensed under [BSD-3 license](https://github.com/teampathfinders/leveldb/blob/master/LICENSE).

