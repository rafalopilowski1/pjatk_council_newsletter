mod feed_entry;

use atom_syndication::*;
use chrono::{NaiveDateTime, Utc};
use feed_entry::FeedEntry;
use ferrishook::webhook;
use futures::executor::block_on;

use std::{
    error::Error,
    io::{BufRead, Write},
    time::Duration,
};

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
    let mut timestamp_file: std::fs::File = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("timestamp.txt")?;
    let mut buf_read = std::io::BufReader::new(timestamp_file.by_ref());
    let mut timestamp_string = String::new();
    buf_read.read_line(&mut timestamp_string)?;
    let last_date_time;
    if let Ok(timestamp) = timestamp_string.parse::<i64>() {
        last_date_time =
            chrono::DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc);
    } else {
        last_date_time =
            chrono::DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc);
    };

    for feed_entry in feed
        .entries()
        .iter()
        .rev()
        .filter(|entry| entry.published().unwrap().timestamp() > last_date_time.timestamp())
    {
        println!(
            "{0} > {1}",
            feed_entry.published().unwrap().timestamp(),
            last_date_time.timestamp()
        );
        let feedEntry: FeedEntry = feed_entry.into();
        tokio::time::sleep(Duration::new(1, 0)).await;
        block_on(async {
            webhook::new(&secret, |webhook| {
                webhook
                    .username("Listonosz PJATK")
                    .embed(|_| feedEntry.into())
            })
            .send()
            .await;
        });
    }
    let newest_timestamp = feed
        .entries
        .first()
        .unwrap()
        .published()
        .unwrap()
        .timestamp()
        .to_string();

    let mut buf_write = std::io::BufWriter::new(timestamp_file.by_ref());
    buf_write.write_all(newest_timestamp.as_bytes())?;
    buf_write.flush()?;
    Ok(())
}
