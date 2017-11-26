# VLive-notifs
Small crate to notify of new videos on [VLive](https://vlive.tv)

### Examples
VLive-notifs is quite easy to use

```rust
extern crate vlive;
use vlive::{VLiveCallback, VLive, VLiveVideo};
use std::time::Duration;

//Our listener
struct Handler;

impl VLiveCallback for Handler {
    fn on_new(video: VLiveVideo) {
        println("New video {} uploaded!", video.video_title);
    }
}

fn main() {
    let _ = VLive::new(Handler, Duration::from_secs(5));
}
```