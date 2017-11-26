struct Listener;

impl super::vlive::VLiveCallback for Listener {
    fn on_new(&self, video: super::vlive::VLiveVideo) {
        println!("Hello from callback {:?}", video);
    }
}

#[test]
fn callback() {
    use std::time::Duration;
    use std::thread::sleep;
    let x = super::vlive::VLive::new(Listener, Duration::from_secs(2));
    let stopper = x.run_async();
    sleep(Duration::from_secs(5));
    stopper.stop();
}
