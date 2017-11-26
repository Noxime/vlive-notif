/// VLive notifications listener
///
/// # Examples
///
/// Simple listener
///
/// ```rust,ignore
/// use vlive::{VLiveCallback, VLive, VLiveVideo};
///
/// //Our listener
/// struct Handler;
///
/// impl VLiveCallback for Handler {
///     fn on_new(video: VLiveVideo) {
///         println("New video {} uploaded!", video.video_title);
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

    use std::{thread, time};
    use std::sync::mpsc::{channel, Sender, Receiver};

    /// VLive video type
    ///
    /// A video on VLive can either be a `VOD` (Video on demand), aka normal
    /// video or `LIVE`, aka a live stream.
    #[derive(Debug)]
    pub enum VideoType {
        VOD,
        LIVE,
    }

    /// VLive channel type
    ///
    /// A channel can either be a `BASIC` (normal) or a `PLUS` (Channel+), which
    /// is a special premium channel
    #[derive(Debug)]
    pub enum ChannelType {
        BASIC,
        PLUS,
    }

    /// Information about a VLive video or a live stream
    ///
    ///
    #[derive(Debug)]
    pub struct VLiveVideo {
        /// Common ID of a video
        ///
        /// This is the "user facing" ID of a video, most
        /// commonly seen in VLive links.
        /// Something like `"/video/50000"`.
        ///
        /// # Examples
        ///
        /// To turn this to a valid URL:
        /// `format!("https://vlive.tv{}", video.video_id);`
        ///
        pub video_id: String,
        /// Sequential video ID
        ///
        /// This ID is the "backend" ID, which is used
        /// internally withing VLive. If you are doing other
        /// api calls, you will most likely use this one to
        /// reference to a video.
        pub video_seq: u32,
        /// The visible title of a video
        ///
        /// String containing the name of a video, shown to
        /// users.
        pub video_title: String,
        /// Either `VideoType::VOD` or `VideoType::LIVE`
        ///
        /// Video type can either be a `VOD` (video on demand)
        /// or `LIVE` (live stream).
        pub video_type: VideoType,
        /// URL to the thumbnail of this video
        ///
        /// Some videos don't always have a thumbnail available,
        /// especially live streams.
        pub video_thumbnail: Option<String>,
        /// Common ID of the channel
        ///
        /// This ID is the "user facing" ID of a channel, usually
        /// seen in VLive channel links, something like `"/channels/EBDF"`
        ///
        /// # Examples
        ///
        /// To turn this to a valid channel URL:
        /// `format!("https://vlive.tv{}", video.channel_id);`
        pub channel_id: String,
        /// Sequential channel ID
        ///
        /// This is the "backend" ID of a channel. Similar to `video_seq`,
        /// if you are doing more VLive backend calls, you'll probably need
        /// this one.
        pub channel_seq: u32,
        /// Visible name of the channel
        ///
        /// User facing name of a channel
        pub channel_name: String,
        /// Either `ChannelType::BASIC` or `ChannelType::PLUS`
        ///
        /// VLive channels can be either `BASIC`, which is a normal, free to
        /// view channel, or a `PLUS` which is a special paid Channel+.
        /// You need a Channel+ subscription to view these videos
        pub channel_type: ChannelType,
    }

    pub struct VLiveStopper {
        tx: Sender<&'static str>
    }

    impl VLiveStopper {
        pub fn stop(self) {
            self.tx.send("stop").unwrap();
        }
    }

    /// Implement this in your own listener
    pub trait VLiveCallback: Send + 'static {
        fn on_new(&self, video: VLiveVideo);
    }

    pub struct VLive<CB> where CB: VLiveCallback {
        /// Up on new video, this callback is called
        callback: CB,
        /// How long to wait between refreshes
        wait: time::Duration,
        /// Our channel we use to control the thread with
        tx: Sender<&'static str>, rx: Receiver<&'static str>
    }

    impl<CB> VLive<CB> where CB: VLiveCallback {

        /// New listener
        ///
        /// `callback` is your implementation of `VLiveCallback`
        /// `wait` is the amount of time to wait between polls.
        /// 2 to 10 seconds is recommended value for `wait`
        pub fn new(callback: CB, wait: time::Duration) -> Self {
            let (tx, rx) = channel();

            VLive {
                callback,
                wait,
                tx, rx
            }
        }

        /// Start listening synchronously
        ///
        /// This is a blocking call until the async loop closes
        /// (which shouldn't happen until you close your program)
        /// See `run_async` if you need to perform actions after this
        pub fn run(self) {
            self.run_async();
            loop {}
        }

        /// Start listening async
        ///
        /// Asynchronous version of `run`
        /// This method starts the event loop, but make sure your
        /// program keeps running after this, most likely with a
        /// infinite loop
        pub fn run_async(self) -> VLiveStopper {
            let callback = self.callback;
            let wait = self.wait;
            let tx = self.tx;
            let rx = self.rx;

            let _ = thread::spawn(move || {
                use self::select::predicate::*;

                //Our parsing code
                //TODO: Move this somewhere else
                let parse_node = |node: select::node::Node| {

                    //Parse the 2 divs that have our needed attributes
                    let html_thumb = match node.find(Class("thumb_area")).last() {
                        Some(value) => value,
                        None => return None,
                    };
                    let html_name = match node.find(Class("name")).last() {
                        Some(value) => value,
                        None => return None,
                    };

                    //Do some crazy shit
                    Some(VLiveVideo {
                        video_id: match html_thumb.attr("href") { Some(v) => v, _ => "" }.to_string(),
                        video_seq: match html_thumb.attr("data-seq") { Some(v) => v.parse().unwrap(), _ => 0u32 },
                        video_title: match html_thumb.attr("data-ga-name") { Some(v) => v, _ => "" }.to_string(),
                        video_type: match html_thumb.attr("data-ga-type") { Some("LIVE") => VideoType::LIVE, _ => VideoType::VOD },
                        video_thumbnail: match html_thumb.find(Attr("src", ())).last() { Some(val) => Some(val.attr("src").unwrap().to_string()), _ => None },
                        channel_id: match html_name.attr("href") { Some(value) => value, None => "", }.to_string(),
                        channel_seq: match html_thumb.attr("data-ga-cseq") { Some(v) => v.parse().unwrap(), _ => 0u32 },
                        channel_name: match html_thumb.attr("data-ga-cname") { Some(v) => v, _ => "" }.to_string(),
                        channel_type: match html_thumb.attr("data-ga-ctype") { Some("PLUS") => ChannelType::PLUS, _ => ChannelType::BASIC },
                    })
                };

                let mut id = 0u32;

                loop {
                    if let Ok(value) = rx.recv() {
                        match value {
                            "start" => println!("VLive thread started"),
                            "stop" => { println!("VLive thread stopped"); break },
                            _ => eprintln!("VLive Error: Unknown signal sent to thread")
                        }

                    }

                    //Fetch HTML from recents page
                    let request = match requests::get("http://www.vlive.tv/home/video/more?pageNo=1&pageSize=15&viewType=recent") {
                        Ok(value) => value,
                        Err(why) => { eprintln!("VLive Error: {}", why); continue }
                    };
                    //Parse HTML
                    let request = request.text().unwrap();

                    //Get latest videos
                    let document = select::document::Document::from(request);
                    let mut new = document.find(Class("video_list_cont"));
                    let first = match parse_node(new.next().unwrap()) {
                        Some(value) => value,
                        None => {
                            eprintln!("VLive Error: Could not parse node (ignored)");
                            continue;
                        },
                    };

                    //Is there a new video?
                    if first.video_seq != id {
                        //Post the new pic
                        let new_id = first.video_seq;
                        callback.on_new(first);

                        //There's a chance more than 1 vid was posted so iterate through those
                        for node in new {
                            let node = match parse_node(node) {
                                Some(value) => value,
                                None => {
                                    eprintln!("VLive Error: Could not parse node (ignored)");
                                    continue;
                                },
                            };

                            //Found where we left off, stop posting
                            if node.video_seq == id {
                                break;
                            }

                            callback.on_new(node);
                        }

                        //Okay go back to your eternal slumber, until you are required again
                        id = new_id;
                    }

                    thread::sleep(wait);
                }
            });
            tx.send("start").unwrap();

            VLiveStopper {
                tx
            }
        }
    }
}

mod tests;
