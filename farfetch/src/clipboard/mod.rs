use anyhow::Result;

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    use copypasta::{ClipboardContext, ClipboardProvider};
    let mut ctx = ClipboardContext::new()
        .map_err(|e| anyhow::anyhow!("Clipboard unavailable: {e}"))?;
    ctx.set_contents(text.to_owned())
        .map_err(|e| anyhow::anyhow!("Clipboard write failed: {e}"))?;
    Ok(())
}

pub fn get_clipboard() -> Option<String> {
    use copypasta::{ClipboardContext, ClipboardProvider};
    let mut ctx = ClipboardContext::new().ok()?;
    ctx.get_contents().ok()
}
