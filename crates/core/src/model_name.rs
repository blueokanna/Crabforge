fn snapshot_date(s: &str) -> bool {
    if !s.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    match s.len() {
        8 => s.starts_with("20"),
        6 => {
            ("23"..="39").contains(&&s[..2])
                && ("01"..="12").contains(&&s[2..4])
                && ("01"..="31").contains(&&s[4..])
        }
        _ => false,
    }
}

pub(crate) fn same_model(model: &str) -> String {
    let leaf = model
        .trim()
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_lowercase();
    let mut key = String::new();
    let mut chars = leaf.chars().peekable();
    let mut digit = false;
    while let Some(c) = chars.next() {
        if c == '-' || c == '_' || c.is_whitespace() {
            if !key.is_empty() && !key.ends_with('-') {
                key.push('-')
            }
        } else if matches!(c, '.' | 'p') && digit && chars.peek().is_some_and(char::is_ascii_digit)
        {
            key.push('-');
        } else {
            key.push(c)
        }
        digit = c.is_ascii_digit();
    }
    key.truncate(key.trim_end_matches('-').len());
    if let Some((prefix, date)) = key.rsplit_once(['-', '@'])
        && !prefix.is_empty()
        && snapshot_date(date)
    {
        key.truncate(prefix.len())
    }
    key
}
