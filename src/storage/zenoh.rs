use std::time::Duration;
use zenoh::{
    self,
    prelude::{sync::SyncResolve, Config},
};

pub fn test_zenoh() {
    let session = zenoh::open(Config::default()).res_sync().unwrap();
    let sub1 = session.declare_subscriber("toot/test").res_sync().unwrap();
    let publ = session.declare_publisher("toot/test").res_sync().unwrap();

    publ.put(1.0).res_sync();
    publ.put(2.0).res_sync();
    publ.put(3.0).res_sync();
    let sub2 = session.declare_subscriber("toot/test").res_sync().unwrap();

    publ.put(4.0).res_sync();
    publ.put(5.0).res_sync();

    for i in 0..1000 {
        println!("sub1: {:?}", sub1.try_recv());
        println!("sub2: {:?}", sub2.try_recv());
        std::thread::sleep(Duration::from_secs(1));
        if i % 3 == 0 {
            // publ.put(1.0).res_sync();
        }
    }
}
