use sentry::Hub;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    assert!(Hub::current().client().is_none());
    assert!(Hub::main().client().is_none());

    tokio::task::spawn(async {
        println!(
            "current thread id for `sentry::init()`: {:?}",
            std::thread::current().id()
        );
        let _guard = sentry::init(sentry::ClientOptions::default());
        assert!(Hub::current().client().is_some());
        assert!(Hub::main().client().is_some());

        let mut futs = vec![];
        for _ in 0..10 {
            futs.push(tokio::task::spawn(async {
                println!("current thread id: {:?}", std::thread::current().id());
                assert!(Hub::current().client().is_some());
                assert!(Hub::main().client().is_some());
            }));
        }
        futures::future::join_all(futs).await;

        println!("passed");
    })
    .await?;

    Ok(())
}
