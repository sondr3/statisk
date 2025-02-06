use std::{
    fmt::{self, Display, Formatter},
    io::Write,
};

use anyhow::{Context, Result};
use jiff::civil::Date;
use url::Url;
use xml::{common::XmlVersion, writer::XmlEvent, EmitterConfig, EventWriter};

use crate::content::Content;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ChangeFreq {
    Always,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Never,
}

impl Display for ChangeFreq {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ChangeFreq::Always => write!(f, "always"),
            ChangeFreq::Hourly => write!(f, "hourly"),
            ChangeFreq::Daily => write!(f, "daily"),
            ChangeFreq::Weekly => write!(f, "weekly"),
            ChangeFreq::Monthly => write!(f, "monthly"),
            ChangeFreq::Yearly => write!(f, "yearly"),
            ChangeFreq::Never => write!(f, "never"),
        }
    }
}

#[derive(Debug)]
pub struct UrlEntry {
    pub loc: Url,
    pub last_mod: Option<Date>,
    pub change_freq: Option<String>,
    pub priority: Option<f32>,
}

impl UrlEntry {
    pub fn new(
        loc: Url,
        last_mod: Option<Date>,
        change_freq: Option<ChangeFreq>,
        priority: Option<f32>,
    ) -> Self {
        assert!(priority.map_or(true, |e| (0.0..=1.0).contains(&e)));

        Self {
            loc,
            last_mod,
            change_freq: change_freq.map(|e| e.to_string()),
            priority,
        }
    }

    pub fn from_content(value: &Content, base: &Url) -> Result<Self> {
        let url = base.join(&value.url)?;

        Ok(UrlEntry::new(
            url,
            value.frontmatter.last_modified,
            Some(ChangeFreq::Monthly),
            None,
        ))
    }

    pub fn to_xml<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<()> {
        writer.write(XmlEvent::start_element("url"))?;

        writer.write(XmlEvent::start_element("loc"))?;
        writer.write(&*self.loc.to_string())?;
        writer.write(XmlEvent::end_element())?;

        if let Some(last_mod) = &self.last_mod {
            writer.write(XmlEvent::start_element("lastmod"))?;
            writer.write(&*last_mod.to_string())?;
            writer.write(XmlEvent::end_element())?;
        }
        if let Some(change_freq) = &self.change_freq {
            writer.write(XmlEvent::start_element("changefreq"))?;
            writer.write(change_freq.as_str())?;
            writer.write(XmlEvent::end_element())?;
        }
        if let Some(priority) = &self.priority {
            writer.write(XmlEvent::start_element("priority"))?;
            writer.write(priority.to_string().as_str())?;
            writer.write(XmlEvent::end_element())?;
        }

        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

pub fn create(urls: Vec<UrlEntry>) -> Result<String> {
    let mut res = Vec::new();
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut res);

    let doc = XmlEvent::StartDocument {
        version: XmlVersion::Version10,
        encoding: Some("UTF-8"),
        standalone: None,
    };

    writer.write(doc)?;

    let stylesheet = XmlEvent::processing_instruction(
        "xml-stylesheet",
        Some(r#"type="text/xsl" href="/sitemap.xsl"?"#),
    );

    writer.write(stylesheet)?;

    let url_set = XmlEvent::start_element("urlset")
        .ns("", "http://www.sitemaps.org/schemas/sitemap/0.9")
        .ns("image", "http://www.google.com/schemas/sitemap-image/1.1")
        .ns("video", "http://www.google.com/schemas/sitemap-video/1.1");

    writer.write(url_set)?;

    for url in urls {
        url.to_xml(&mut writer)?;
    }

    writer.write(XmlEvent::end_element())?;

    String::from_utf8(res).context("Sitemap failed to serialize")
}

#[cfg(test)]
mod tests {
    use jiff::civil::Date;
    use url::Url;

    use crate::sitemap::{create, ChangeFreq, UrlEntry};

    #[test]
    fn test_sitemap() {
        let urls = vec![
            UrlEntry::new(
                Url::parse("http://www.example.com/").unwrap(),
                None,
                None,
                None,
            ),
            UrlEntry::new(
                Url::parse("https://example.org/").unwrap(),
                Some(Date::new(2005, 1, 1).unwrap()),
                Some(ChangeFreq::Monthly),
                Some(0.8),
            ),
            UrlEntry::new(
                Url::parse("http://www.example.com/catalog?item=12&amp;desc=vacation_hawaii")
                    .unwrap(),
                None,
                Some(ChangeFreq::Weekly),
                None,
            ),
            UrlEntry::new(
                Url::parse("http://www.example.com/catalog?item=73&amp;desc=vacation_new_zealand")
                    .unwrap(),
                Some(Date::new(2004, 12, 23).unwrap()),
                Some(ChangeFreq::Weekly),
                None,
            ),
            UrlEntry::new(
                Url::parse("http://www.example.com/catalog?item=74&amp;desc=vacation_newfoundland")
                    .unwrap(),
                Some(Date::new(2004, 12, 23).unwrap()),
                None,
                Some(0.3),
            ),
            UrlEntry::new(
                Url::parse("http://www.example.com/catalog?item=83&amp;desc=vacation_usa").unwrap(),
                Some(Date::new(2004, 11, 23).unwrap()),
                None,
                None,
            ),
        ];

        let sitemap = create(urls).unwrap();
        insta::assert_snapshot!(sitemap);
    }
}
