mod feed_entry;

use atom_syndication::*;
use feed_entry::FeedEntry;
use ferrishook::*;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let feed: Feed = get_data("https://samorzad.pja.edu.pl/feed/atom").await?;
    send_webhook(&feed).await
}

async fn get_data(feed_url: &str) -> Result<Feed, Box<dyn Error>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.45 Safari/537.36")
        .build()?;
    let request = client.get(feed_url);
    let response = request.send().await?;
    let response_data = response.text().await?;
    let feed = response_data.parse::<Feed>()?;
    Ok(feed)
}

async fn send_webhook(feed: &Feed) -> Result<(), Box<dyn Error>> {
    let secret = std::env::var("WEBHOOK_SECRET")?;
    let feed_entry: FeedEntry = feed.entries().first().unwrap().into();
    println!("{}", feed_entry.footer.published);

    Ok(webhook::new(&secret, |webhook| {
        webhook
            .username("Listonosz PJATK")
            .embed(|_| feed_entry.into())
    })
    .send()
    .await?)
}
