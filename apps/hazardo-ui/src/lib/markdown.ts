import { marked } from "marked";
import DOMPurify from "dompurify";
import { openUrl } from "@tauri-apps/plugin-opener";

// Configure marked for safe link rendering
marked.setOptions({ breaks: true, gfm: true });

const renderer = new marked.Renderer();
renderer.link = function ({
  href,
  title,
  text,
}: {
  href: string;
  title?: string | null | undefined;
  text: string;
}) {
  const titleAttr = title ? ` title="${title}"` : "";
  return `<a data-href="${href}"${titleAttr} role="link" tabindex="0" class="md-link">${text}</a>`;
};
marked.use({ renderer });

/**
 * Sanitize HTML using DOMPurify — prevents XSS from AI-generated markdown.
 */
export function sanitizeHtml(html: string): string {
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: [
      "h1",
      "h2",
      "h3",
      "h4",
      "h5",
      "h6",
      "p",
      "br",
      "hr",
      "ul",
      "ol",
      "li",
      "strong",
      "em",
      "b",
      "i",
      "u",
      "s",
      "del",
      "a",
      "code",
      "pre",
      "blockquote",
      "table",
      "thead",
      "tbody",
      "tr",
      "th",
      "td",
      "img",
      "span",
      "div",
      "sup",
      "sub",
    ],
    ALLOWED_ATTR: [
      "href",
      "data-href",
      "title",
      "target",
      "rel",
      "src",
      "alt",
      "class",
      "role",
      "tabindex",
    ],
    ALLOW_DATA_ATTR: true,
  });
}

/**
 * Parse markdown to sanitized HTML — safe for {@html} rendering.
 */
export function renderMarkdown(content: string): string {
  const html = marked.parse(content) as string;
  return sanitizeHtml(html);
}

/**
 * Click handler for markdown containers — opens data-href links in external browser.
 * Use with on:click={handleMarkdownClick} on the element containing {@html renderMarkdown(...)}.
 */
export function handleMarkdownClick(e: MouseEvent) {
  const target = e.target as HTMLElement;
  const anchor = target.closest("a[data-href]") as HTMLElement | null;
  if (anchor) {
    const url = anchor.getAttribute("data-href");
    if (url && (url.startsWith("http://") || url.startsWith("https://"))) {
      e.preventDefault();
      e.stopPropagation();
      openUrl(url);
    }
  }
}
