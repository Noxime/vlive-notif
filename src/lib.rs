/// VLive notifications listener
///
/// # Examples
///
/// Simple listener
///
/// ```rust
/// struct Handler;
/// impl VLiveCallback for Handler {
///     fn on_new() {
///         println("New video uploaded!");
///     }
/// }
///
/// fn main() {
///     let _ = VLive::new(Handler {});
/// }
/// ```
///
pub mod vlive {
    extern crate requests;
    extern crate select;

    use std::thread;
    use std::sync::mpsc::channel;

    /// VLive video type
    ///
    /// A video on VLive can either be a `VOD` (Video on demand), aka normal
    /// video or `LIVE`, aka a live stream.
    pub enum VideoType {
        VOD,
        LIVE,
    }

    /// VLive channel type
    ///
    /// A channel can either be a `BASIC` (normal) or a `PLUS` (Channel+), which
    /// is a special premium channel
    pub enum ChannelType {
        BASIC,
        PLUS,
    }

    pub struct VLiveVideo {
        video_id: String,
        video_seq: u32,
        video_title: String,
        video_type: VideoType,
        video_thumbnail: Option<String>,
        channel_id: String,
        channel_seq: u32,
        channel_name: String,
        channel_type: ChannelType,
    }

    /// Implement this in your own listener
    pub trait VLiveCallback {
        fn on_new(self);
    }

    pub struct VLive<CB> where CB: VLiveCallback {
        /// Up on new video, this callback is called
        callback: CB,
        /// ID of the latest video
        latest_id: u32,
    }

    impl<CB> VLive<CB> where CB: VLiveCallback {

        /// New listener
        pub fn new(callback: CB) -> Self {
            VLive {
                callback,
                latest_id: 0
            }
        }

        /// Start listening synchronously
        ///
        /// This is a blocking call until the async loop closes
        /// (which shouldn't happen until you close your program)
        /// See `run_async` if you need to perform actions after this
        pub fn run(self) {
            self.run_async().join();
        }

        /// Start listening async
        ///
        /// Asynchronous version of `run`
        /// This method starts the event loop, but make sure your
        /// program keeps running after this, most likely with a
        /// infinite loop
        pub fn run_async(self) -> thread::JoinHandle<()> {
            let (tx, rx) = channel();

            let guard = thread::spawn(move || {

                //Our parsing code
                //TODO: Move this somewhere else
                let parse_node = |node: select::node::Node| {
                    println!("{}", node.text());
                };

                loop {
                    if let Ok("start") = rx.recv() {
                        println!("VLive thread started");
                    }

                    //Fetch HTML from recents page
                    let request = match requests::get("http://www.vlive.tv/home/video/more?pageNo=1&pageSize=5&viewType=recent") {
                        Ok(value) => value,
                        Err(why) => { eprintln!("VLive Error: {}", why); continue }
                    };
                    //Parse HTML
                    let request = request.text().unwrap();

                    use self::select::predicate::*;

                    //Find the correct info
                    let document = select::document::Document::from(request);
                    for node in document.find(Class("video_list_cont")) {
                        let node = parse_node(node);
                    }

                    thread::sleep_ms(10000);
                }
            });
            tx.send("start").unwrap();
            guard
        }
    }
}

#[cfg(test)]
mod tests {

    struct Listener;
    impl super::vlive::VLiveCallback for Listener {
        fn on_new(self) {
            println!("Hello from callback")
        }
    }

    #[test]
    fn callback() {
        use std::thread;
        let x = super::vlive::VLive::new(Listener);
        x.run();
    }
}
