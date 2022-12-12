# Danfoss Ally API library

Very basic rust native library to interact with the Danfoss Ally API.

## Get started

Create a [Danfoss Developer](https://developer.danfoss.com/apis).
Follow the instructions on the website to create the credentials for the Danfoss
Ally API.

Then, Provide your Danfoss API credentials as environment variables.

```bash
export DANFOSS_API_KEY=YOUR_API_KEY
export DANFOSS_API_SECRET=YOUR_API_SECRET
```

After that you are all set and can query the API

```rust
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info! {"Starting up"};
    let mut danfoss_api: API = API::new();
    danfoss_api.get_token().await?;
    danfoss_api.get_devices().await?;
    danfoss_api.print_room_temperatures();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
```

You need to set the logging level to debug in order to print room temperatures.

```bash
RUST_LOG=debug cargo run
```

## Disclaimer

This is not an official library and i am not affiliated with Danfoss in any way.
