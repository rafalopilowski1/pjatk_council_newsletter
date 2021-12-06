use std::borrow::Borrow;

use atom_syndication::Entry;
use ferrishook::embed::*;
use regex::Regex;
pub(crate) struct FeedEntry {
    title: String,
    author: FeedAuthor,
    image: String,
    description: String,
    pub footer: FeedFooter,
    color: u32,
}

impl From<&Entry> for FeedEntry {
    fn from(atomEntry: &Entry) -> Self {
        let feed_content_str = atomEntry.content().unwrap().value().unwrap();
        let feed_content_dom = scraper::html::Html::parse_document(feed_content_str);
        FeedEntry {
            title: atomEntry.title().to_string(),
            author: atomEntry.borrow().into(),
            image: {
                let mut feed_content_image_url = String::from("https://samorzad.pja.edu.pl");
                for image_element in feed_content_dom
                    .select(&scraper::Selector::parse("img").unwrap())
                    .take(1)
                {
                    feed_content_image_url.push_str(image_element.value().attr("src").unwrap());
                }
                feed_content_image_url
            },
            description: {
                let mut text = String::new();
                for textOnce in feed_content_dom.root_element().text().into_iter() {
                    text.push_str(textOnce);
                }
                let regex = Regex::new(r"\n+").unwrap();
                text = regex.replace_all(&text, "\n\n").to_string();
                text
            },
            footer: atomEntry.borrow().into(),
            color: 0x8ebda7,
        }
    }
}
pub(crate) struct FeedAuthor {
    name: String,
    url: String,
}

impl From<&Entry> for FeedAuthor {
    fn from(atomEntry: &Entry) -> Self {
        FeedAuthor {
            name: atomEntry.authors().first().unwrap().name().to_string(),
            url: atomEntry.links().first().unwrap().href().to_string(),
        }
    }
}
pub(crate) struct FeedFooter {
    pub published: i64,
    text: String,
}

impl From<&Entry> for FeedFooter {
    fn from(atomEntry: &Entry) -> Self {
        FeedFooter {
            published: atomEntry.published().unwrap().timestamp_nanos(),
            text: "Data publikacji: ".to_owned()
                + &atomEntry
                    .published()
                    .unwrap()
                    .naive_local()
                    .format("%d-%m-%Y %H:%M:%S")
                    .to_string(),
        }
    }
}

impl From<FeedEntry> for WebHookEmbed {
    fn from(feedEntry: FeedEntry) -> Self {
        WebHookEmbed {
            title: Some(feedEntry.title.clone()),
            description: Some(feedEntry.description.clone()),
            color: Some(feedEntry.color),
            author: Some(feedEntry.author.borrow().into()),
            fields: None,
            footer: Some(feedEntry.footer.borrow().into()),
            image: Some(feedEntry.borrow().into()),
            thumbnail: None,
        }
    }
}
impl From<&FeedAuthor> for WebHookEmbedAuthor {
    fn from(feedAuthor: &FeedAuthor) -> Self {
        Self {
            name: Some(feedAuthor.name.clone()),
            icon_url: None,
            url: Some(feedAuthor.url.clone()),
        }
    }
}
impl From<&FeedFooter> for WebHookEmbedFooter {
    fn from(feedFooter: &FeedFooter) -> Self {
        Self {
            text: Some(feedFooter.text.clone()),
            icon_url: None,
        }
    }
}
impl From<&FeedEntry> for WebHookEmbedImage {
    fn from(feedEntry: &FeedEntry) -> Self {
        Self {
            url: Some(feedEntry.image.clone()),
        }
    }
}
