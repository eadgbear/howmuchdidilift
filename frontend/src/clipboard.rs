//! Share / copy helpers. The native Web Share API carries image + text to the
//! OS share sheet (great on mobile, unreliable on desktop), so we only offer it
//! on mobile and give desktop explicit "copy image" / "copy text" actions.
//! Logic is inline JS — these Web APIs are steadier there than via web-sys.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
export function isMobile() {
  return !!navigator.share && /Android|iPhone|iPad|iPod|Mobile/i.test(navigator.userAgent || '');
}

export async function shareCard(imageUrl, text, title) {
  let file = null;
  try {
    const blob = await (await fetch(imageUrl)).blob();
    file = new File([blob], 'howmuchdidilift.png', { type: blob.type || 'image/png' });
  } catch (e) {}
  try {
    if (file && navigator.canShare && navigator.canShare({ files: [file] })) {
      await navigator.share({ title, text, files: [file] });
      return 'shared';
    }
    if (navigator.share) { await navigator.share({ title, text }); return 'shared'; }
  } catch (e) {
    if (e && e.name === 'AbortError') return 'cancelled';
  }
  return 'fail';
}

export async function copyImage(imageUrl) {
  try {
    const blob = await (await fetch(imageUrl)).blob();
    await navigator.clipboard.write([ new ClipboardItem({ [blob.type || 'image/png']: blob }) ]);
    return true;
  } catch (e) { return false; }
}

export async function copyText(text) {
  try { await navigator.clipboard.writeText(text); return true; }
  catch (e) { return false; }
}

export function downloadImage(dataUrl, filename) {
  const a = document.createElement('a');
  a.href = dataUrl;
  a.download = filename || 'howmuchdidilift.png';
  document.body.appendChild(a);
  a.click();
  a.remove();
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = isMobile)]
    fn is_mobile_js() -> bool;
    #[wasm_bindgen(js_name = downloadImage)]
    fn download_image_js(data_url: &str, filename: &str);
    #[wasm_bindgen(js_name = shareCard, catch)]
    async fn share_card_js(image_url: &str, text: &str, title: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = copyImage, catch)]
    async fn copy_image_js(image_url: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = copyText, catch)]
    async fn copy_text_js(text: &str) -> Result<JsValue, JsValue>;
}

/// True on touch devices with Web Share (where the share sheet works well).
#[must_use]
pub fn is_mobile() -> bool {
    is_mobile_js()
}

/// Trigger a browser download of the card image (a `data:` URL) as `filename`.
pub fn download_image(data_url: &str, filename: &str) {
    download_image_js(data_url, filename);
}

/// Share image + text via the OS sheet. Returns `shared`, `cancelled`, or `fail`.
pub async fn share_card(image_url: &str, text: &str, title: &str) -> String {
    share_card_js(image_url, text, title)
        .await
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "fail".to_string())
}

/// Copy just the card image to the clipboard.
pub async fn copy_image(image_url: &str) -> bool {
    copy_image_js(image_url)
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Copy just the share text to the clipboard.
pub async fn copy_text(text: &str) -> bool {
    copy_text_js(text)
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
