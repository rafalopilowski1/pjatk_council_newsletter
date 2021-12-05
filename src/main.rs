use atom_syndication::*;
use ferrishook::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let feed: Feed = get_data("https://samorzad.pja.edu.pl/feed/atom").await?;
    send_webhook(&feed).await
}

async fn get_data(feed_url: &str) -> Result<Feed, Box<dyn Error>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Android 4.4; Mobile; rv:41.0) Gecko/41.0 Firefox/41.0")
        .build()?;
    let request = client.get(feed_url);
    let response = request.send().await?;
    let response_data = response.text().await?;
    let feed = response_data.parse::<Feed>()?;
    Ok(feed)
}

async fn send_webhook(feed: &Feed) -> Result<(), Box<dyn Error>> {
    let secret = "https://discord.com/api/webhooks/917158640119611442/K8F4SWhJGzqM0GiH3VVy36wQlLK02VoVz2EVXTLbkZcnLG3gi_pvVuW09rXxEteL9nQs";
    Ok(webhook::new(secret, |webhook| {
        let feed_entry = feed.entries().last().unwrap();
        webhook.username("Listonosz PJATK").embed(|embed| {
            let feed_content = feed_entry.content()?.value()?;
            embed
                .author(|author| {
                    let feed_author = feed_entry.authors().first().unwrap();
                    author.name(feed_author.name())
                    //.url(feed_author.uri().unwrap())
                })
                .color(0)
                .description(feed_content)
                .footer(|footer| footer.text(feed_entry.published().unwrap().to_rfc2822()))
                .title(feed_entry.title())
        })
    })
    .send()
    .await?)
}
