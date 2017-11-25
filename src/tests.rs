struct Listener;

impl super::vlive::VLiveCallback for Listener {
    fn on_new(self, video: super::vlive::VLiveVideo) {
        println!("Hello from callback {}", video.video_id)
    }
}

impl Copy for Listener {}
impl Clone for Listener {
    fn clone(&self) -> Self {
        Self {}
    }
}

#[test]
fn callback() {
    use std::thread;
    let x = super::vlive::VLive::new(Listener);
    x.run();
}
