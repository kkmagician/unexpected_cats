# unexpected_cats
A lightweight tool for forwarding VK posts with text ad images to Telegram.

Requires Redis for data storage and configuration.

## Configuration
### Environment variables
`TG_TOKEN` – Telegram Bot token<br>
`VK_TOKEN` – VK token for https://api.vk.com/method/wall.get method<br>
`REDIS_HOST` – Redis instance host, defaults to `127.0.0.1`<br>
`REDIS_PORT` – Redis instance port, defaults to `6379`<br>
`REDIS_DB` – Redis DB index, defaults to `0`<br>
`REDIS_PASS` – Optional password

### Data and storage
Create a list named `owner:chat`, which contains strings with the `:` (colon) divider like this:
`-12345:@my_channel:10`, where the first element is VK post owner ID (negative integer for communities, positive for users), the second one is a Telegram channel handle, and the third (optional) is how many posts are fetched each time the program runs.

### Deployment
Run with docker or build binary with `cargo build --release` and set a scheduler.