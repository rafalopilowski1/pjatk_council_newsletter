use atom_syndication::*;
use ferrishook::*;
use regex::Regex;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://samorzad.pja.edu.pl";
    let feed: Feed = get_data("https://samorzad.pja.edu.pl/feed/atom").await?;
    send_webhook(&feed, url).await
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

async fn send_webhook(feed: &Feed, url: &str) -> Result<(), Box<dyn Error>> {
    let secret = std::env::var("WEBHOOK_SECRET")?;
    Ok(webhook::new(&secret, |webhook| {
        let feed_entry = feed.entries().first().unwrap();
        webhook.username("Listonosz PJATK").embed(|embed| {
            let mut feed_content_str = feed_entry.content().unwrap().value().unwrap();
            let feed_content_dom = scraper::html::Html::parse_document(feed_content_str);
            let mut text = String::new();
            for textOnce in feed_content_dom.root_element().text().into_iter() {
                text.push_str(textOnce);
            }
            let regex = Regex::new(r"\n+").unwrap();
            text = regex.replace_all(&mut text, "\n\n").to_string();
            let mut feed_content_image_url = String::from(url);
            for image_element in feed_content_dom
                .select(&scraper::Selector::parse("img").unwrap())
                .take(1)
            {
                feed_content_image_url.push_str(image_element.value().attr("src").unwrap());
            }

            embed
                .author(|author| {
                    let feed_author = feed_entry.authors().first().unwrap();
                    author
                        .name(feed_author.name())
                        .url(feed_entry.links().first().unwrap().href())
                })
                .color(0x8ebda7)
                .image(feed_content_image_url)
                .description(text)
                .footer(|footer| {
                    footer.text(
                        "Data publikacji: ".to_owned()
                            + &feed_entry
                                .published()
                                .unwrap()
                                .naive_local()
                                .format("%d-%m-%Y %H:%M:%S")
                                .to_string(),
                    )
                })
                .title(feed_entry.title())
        })
    })
    .send()
    .await?)
}
