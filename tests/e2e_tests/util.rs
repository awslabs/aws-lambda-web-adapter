use http::Uri;
use percent_encoding::{AsciiSet, CONTROLS};

const BASE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'/')
    // RFC-3986 §3.3 allows sub-delims (defined in section2.2) to be in the path component.
    // This includes both colon ':' and comma ',' characters.
    // Smithy protocol tests & AWS services percent encode these expected values. Signing
    // will fail if these values are not percent encoded
    .add(b':')
    .add(b',')
    .add(b'?')
    .add(b'#')
    .add(b'[')
    .add(b']')
    .add(b'{')
    .add(b'}')
    .add(b'|')
    .add(b'@')
    .add(b'!')
    .add(b'$')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b';')
    .add(b'=')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'"')
    .add(b'^')
    .add(b'`')
    .add(b'\\');

#[inline]
pub fn utf8_percent_encode<'a>(input: &'a str, ascii_set: &'static AsciiSet) -> percent_encoding::PercentEncode<'a> {
    percent_encoding::percent_encode(input.as_bytes(), ascii_set)
}

pub fn percent_encode_query<T: AsRef<str>>(t: T) -> String {
    utf8_percent_encode(t.as_ref(), BASE_SET).to_string()
}

#[allow(missing_debug_implementations)]
pub struct QueryWriter {
    base_uri: Uri,
    new_path_and_query: String,
    prefix: Option<char>,
}

impl QueryWriter {
    /// Creates a new `QueryWriter` based off the given `uri`.
    pub fn new(uri: &Uri) -> Self {
        let new_path_and_query = uri.path_and_query().map(|pq| pq.to_string()).unwrap_or_default();
        let prefix = if uri.query().is_none() {
            Some('?')
        } else if !uri.query().unwrap_or_default().is_empty() {
            Some('&')
        } else {
            None
        };
        QueryWriter {
            base_uri: uri.clone(),
            new_path_and_query,
            prefix,
        }
    }

    /// Inserts a new query parameter. The key and value are percent encoded
    /// by `QueryWriter`. Passing in percent encoded values will result in double encoding.
    pub fn insert(&mut self, k: &str, v: &str) {
        if let Some(prefix) = self.prefix {
            self.new_path_and_query.push(prefix);
        }
        self.prefix = Some('&');
        self.new_path_and_query.push_str(&percent_encode_query(k));
        self.new_path_and_query.push('=');

        self.new_path_and_query.push_str(&percent_encode_query(v));
    }

    /// Returns a full [`Uri`] with the query string updated.
    pub fn build_uri(self) -> Uri {
        let mut parts = self.base_uri.into_parts();
        parts.path_and_query = Some(
            self.new_path_and_query
                .parse()
                .expect("adding query should not invalidate URI"),
        );
        Uri::from_parts(parts).expect("a valid URL in should always produce a valid URL out")
    }
}
