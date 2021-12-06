mod feed_entry;

use atom_syndication::*;
use chrono::{NaiveDateTime, Utc};
use feed_entry::FeedEntry;
use ferrishook::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    let mut timestamp_file = tokio::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open("timestamp.txt")
        .await?;
    let mut timestamp_string = String::new();
    timestamp_file.read_to_string(&mut timestamp_string).await?;
    let last_date_time;
    if let Ok(timestamp) = timestamp_string.parse::<u32>() {
        last_date_time =
            chrono::DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, timestamp), Utc);
    } else {
        last_date_time =
            chrono::DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc);
    };
    for feed_entry in feed.entries().iter().rev().filter(|entry| {
        entry
            .published()
            .unwrap()
            .signed_duration_since::<Utc>(last_date_time)
            .num_nanoseconds()
            .unwrap()
            > 0
    }) {
        let feedEntry: FeedEntry = feed_entry.into();
        webhook::new(&secret, |webhook| {
            webhook
                .username("Listonosz PJATK")
                .embed(|_| feedEntry.into())
        })
        .send()
        .await?;
    }
    let newest_timestamp = feed
        .entries
        .first()
        .unwrap()
        .published()
        .unwrap()
        .timestamp_nanos()
        .to_string();
    timestamp_file
        .write_all(newest_timestamp.as_bytes())
        .await?;
    Ok(())
}
