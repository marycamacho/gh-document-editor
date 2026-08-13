import { marked } from "marked";
import DOMPurify from "dompurify";

/**
 * Render markdown to sanitized HTML for the preview pane. Content comes from
 * the repo (other people's edits), so it is always sanitized before display.
 */
export function renderMarkdown(src: string): string {
  const html = marked.parse(src, { gfm: true, async: false }) as string;
  return DOMPurify.sanitize(html);
}
