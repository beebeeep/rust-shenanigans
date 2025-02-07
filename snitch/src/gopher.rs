use anyhow::{anyhow, Context, Result};
use std::io::Read;
use std::{fmt::Display, io::BufRead};
use std::{
    io::{BufReader, Cursor, Write},
    net::TcpStream,
};

const _INVALID_ENTRY: DirEntry = DirEntry {
    item_type: GopherItem::Unknown,
    label: String::new(),
    url: None,
};

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum GopherItem {
    TextFile,
    Submenu,
    Nameserver,
    Error,
    BinHex,
    Dos,
    UuencodeFile,
    FullTextSearch,
    Telnet,
    BinaryFile,
    Mirror,
    GifFile,
    ImageFile,
    Telnet3270,
    BitmapFile,
    MovieFile,
    SoundFile,
    DocFile,
    HtmlFile,
    Info,
    PngFile,
    RtfFile,
    WavFile,
    PdfFile,
    XmlFile,
    Unknown,
}

impl From<char> for GopherItem {
    fn from(c: char) -> GopherItem {
        match c {
            '0' => Self::TextFile,
            '1' => Self::Submenu,
            '2' => Self::Nameserver,
            '3' => Self::Error,
            '4' => Self::BinHex,
            '5' => Self::Dos,
            '6' => Self::UuencodeFile,
            '7' => Self::FullTextSearch,
            '8' => Self::Telnet,
            '9' => Self::BinaryFile,
            '+' => Self::Mirror,
            'g' => Self::GifFile,
            'I' => Self::ImageFile,
            'T' => Self::Telnet3270,
            ':' => Self::BitmapFile,
            ';' => Self::MovieFile,
            '<' => Self::SoundFile,
            'd' => Self::DocFile,
            'h' => Self::HtmlFile,
            'i' => Self::Info,
            'p' => Self::PngFile,
            'r' => Self::RtfFile,
            's' => Self::WavFile,
            'P' => Self::PdfFile,
            'X' => Self::XmlFile,
            _ => Self::Unknown,
        }
    }
}

impl Into<char> for GopherItem {
    fn into(self) -> char {
        match self {
            Self::TextFile => '0',
            Self::Submenu => '1',
            Self::Nameserver => '2',
            Self::Error => '3',
            Self::BinHex => '4',
            Self::Dos => '5',
            Self::UuencodeFile => '6',
            Self::FullTextSearch => '7',
            Self::Telnet => '8',
            Self::BinaryFile => '9',
            Self::Mirror => '+',
            Self::GifFile => 'g',
            Self::ImageFile => 'I',
            Self::Telnet3270 => 'T',
            Self::BitmapFile => ':',
            Self::MovieFile => ';',
            Self::SoundFile => '<',
            Self::DocFile => 'd',
            Self::HtmlFile => 'h',
            Self::Info => 'i',
            Self::PngFile => 'p',
            Self::RtfFile => 'r',
            Self::WavFile => 's',
            Self::PdfFile => 'P',
            Self::XmlFile => 'X',
            Self::Unknown => '?',
        }
    }
}

impl Display for GopherItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<char>::into(self.clone()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GopherURL {
    pub host: String,
    pub port: u16,
    pub gopher_type: GopherItem,
    pub selector: String,
}

impl TryFrom<&str> for GopherURL {
    type Error = anyhow::Error;
    fn try_from(url_str: &str) -> Result<Self> {
        let gopher_url_re = regex_static::static_regex!(
            r#"(?:gopher://)?(?P<host>[^:/]+)(?::(?P<port>\d+))?(?:/(?P<type>[A-z0-9:+:;<?])(?P<selector>.*))?$"#
        );
        let Some(matches) = gopher_url_re.captures(url_str) else {
            return Err(anyhow!("failed to parse URL"));
        };
        Ok(Self {
            host: String::from(matches.name("host").unwrap().as_str()),
            port: matches
                .name("port")
                .map_or(70, |p| p.as_str().parse().unwrap_or(70)),
            gopher_type: matches.name("type").map_or(GopherItem::Submenu, |t| {
                t.as_str().chars().next().unwrap().into()
            }),
            selector: matches
                .name("selector")
                .map_or(String::from(""), |s| String::from(s.as_str())),
        })
    }
}

impl Display for GopherURL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.selector.is_empty() {
            write!(f, "gopher://{}:{}", self.host, self.port)
        } else {
            write!(
                f,
                "gopher://{}:{}/{}{}",
                self.host, self.port, self.gopher_type, self.selector
            )
        }
    }
}

impl GopherURL {
    fn new(host: &str, port: &str, item_type: &GopherItem, selector: &str) -> Self {
        Self {
            host: String::from(host),
            port: port.parse().unwrap_or(70),
            gopher_type: item_type.clone(),
            selector: String::from(selector),
        }
    }
}

#[derive(Debug)]
pub struct DirEntry {
    pub item_type: GopherItem,
    pub label: String,
    pub url: Option<GopherURL>,
}

impl From<&str> for DirEntry {
    fn from(value: &str) -> Self {
        let mut e = value.split('\t');
        match (e.next(), e.next(), e.next(), e.next()) {
            (Some(item_label), Some(selector), Some(host), Some(port)) => {
                let mut s = item_label.chars();
                let t: GopherItem = match s.next() {
                    Some(c) => c.into(),
                    None => {
                        return _INVALID_ENTRY;
                    }
                };
                let label: String = s.collect();
                DirEntry::new(t, label.as_str(), selector, host, port)
            }
            _ => _INVALID_ENTRY,
        }
    }
}

impl DirEntry {
    pub fn new(item_type: GopherItem, label: &str, selector: &str, host: &str, port: &str) -> Self {
        match item_type {
            GopherItem::Info => DirEntry {
                item_type,
                label: String::from(label),
                url: None,
            },
            _ => DirEntry {
                item_type,
                label: String::from(label),
                url: Some(GopherURL::new(host, port, &item_type, selector)),
            },
        }
    }
}

pub struct Menu {
    pub items: Vec<DirEntry>,
}

impl Menu {
    pub fn from_url(url: &GopherURL, query: Option<String>) -> Result<Self, anyhow::Error> {
        let mut items: Vec<DirEntry> = Vec::new();
        let mut response = fetch_url(&url, query)
            .context(format!("fetching {url}"))?
            .lines();
        while let Some(Ok(line)) = response.next() {
            if line == "." {
                break;
            }
            let entry = DirEntry::from(line.as_str());
            match entry.item_type {
                GopherItem::Unknown => continue,
                GopherItem::Info => {
                    if let Some(item) = items.last_mut() {
                        // merge subsequent info items into one paragraph
                        // to preserve whatever pseudographic may be there
                        if item.item_type == GopherItem::Info {
                            item.label.push_str(format!("\n{}", entry.label).as_str());
                            continue;
                        }
                    }
                    items.push(entry)
                }
                _ => items.push(entry),
            }
        }

        Ok(Self { items: items })
    }
}

pub fn fetch_url(url: &GopherURL, query: Option<String>) -> Result<impl BufRead> {
    let addr = format!("{}:{}", url.host, url.port);
    let mut stream = TcpStream::connect(&addr).context(format!("connecting to {addr}"))?;
    let selector = query.map_or(format!("{}\r\n", url.selector), |q| {
        format!("{}\t{}\r\n", url.selector, q)
    });
    stream
        .write_all(selector.as_bytes())
        .context(format!("querying {addr}"))?;
    let mut buf = BufReader::new(stream);

    /*
       Since gopher has no way to specify any metadata in its response,
       so instead of actual content there may be a dir entry with error.
       To handle this, we peek into response to see if it is
       possible to parse it into dir entry and whether there is an error.
       If not, returns original content.
    */
    let mut header = vec![0; 256];
    let bytes_read = buf.read(&mut header).context("reading gopher reply")?;
    if let Ok(first_line) = String::from_utf8(header.clone()) {
        match DirEntry::from(first_line.as_str()) {
            entry if entry.item_type == GopherItem::Error => {
                log::error!("got error fetching {}: {}", url, entry.label);
                return Err(anyhow!(entry.label));
            }
            _ => {}
        }
    }
    Ok(Cursor::new(header[0..bytes_read].to_vec()).chain(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_entries() {
        let mut e = DirEntry::from("1Test entry\t/test\texample.com\t70\r\n");
        assert_eq!(e.label, "Test entry");
        assert_eq!(e.item_type, GopherItem::Submenu);
        assert_eq!(e.url.unwrap().host, "example.com");
        e = DirEntry::from("0test2	selector	1.1.1.1	70\r\n");
        assert_eq!(e.label, "test2");
        assert_eq!(e.item_type, GopherItem::TextFile);
        let url = e.url.unwrap();
        assert_eq!(url.host, "1.1.1.1");
        assert_eq!(url.selector, "selector");
        assert_eq!(url.gopher_type, GopherItem::TextFile);
    }

    #[test]
    fn parsing_urls() {
        let mut u = GopherURL::try_from("gopher://example.com/0/path/to/document").unwrap();
        assert_eq!(u.gopher_type, GopherItem::TextFile);
        assert_eq!(u.host, "example.com");
        assert_eq!(u.port, 70);
        assert_eq!(u.selector, "/path/to/document");
        assert_eq!(u.to_string(), "gopher://example.com:70/0/path/to/document");

        u = GopherURL::try_from("gopher://example2.com:71").unwrap();
        assert_eq!(u.gopher_type, GopherItem::Submenu);
        assert_eq!(u.host, "example2.com");
        assert_eq!(u.port, 71);
        assert_eq!(u.selector, "");
        assert_eq!(u.to_string(), "gopher://example2.com:71");

        u = GopherURL::try_from("gopher://khzae.net:70/</music/khzae/khzae.ogg").unwrap();
        assert_eq!(u.gopher_type, GopherItem::SoundFile);
        assert_eq!(u.host, "khzae.net");
        assert_eq!(u.port, 70);

        u = GopherURL::new("1.1.1.1", "70", &GopherItem::TextFile, "some-selector");
        assert_eq!(u.to_string(), "gopher://1.1.1.1:70/0some-selector");
    }
}
