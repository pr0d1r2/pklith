//! Generated blocks inside hand-written files, between
//! `<!-- BEGIN name ... -->` and `<!-- END name -->`.

/// `doc` with block `name` replaced by `block` (markers included); `None`
/// when `doc` has no such block.
#[must_use]
pub fn splice(doc: &str, name: &str, block: &str) -> Option<String> {
    let end = format!("<!-- END {name} -->");
    let start = doc.find(&format!("<!-- BEGIN {name}"))?;
    let stop = doc.get(start..)?.find(&end)? + start + end.len();
    let after = doc.get(stop..)?;
    let rest = after.strip_prefix('\n').unwrap_or(after);
    Some(format!("{}{block}{rest}", doc.get(..start)?))
}

#[cfg(test)]
mod tests {
    use super::splice;

    #[test]
    fn only_the_named_block_is_replaced() {
        let doc = "# t\n\n<!-- BEGIN x: note -->\nold\n<!-- END x -->\n\nbody\n";
        assert_eq!(
            splice(doc, "x", "NEW\n").as_deref(),
            Some("# t\n\nNEW\n\nbody\n")
        );
        let end = "<!-- BEGIN x -->\nold\n<!-- END x -->";
        assert_eq!(splice(end, "x", "NEW\n").as_deref(), Some("NEW\n"));
        assert_eq!(splice(doc, "y", "x"), None);
        assert_eq!(splice("<!-- BEGIN x -->\nno end", "x", "x"), None);
    }
}
