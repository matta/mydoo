import { writeFile } from "node:fs/promises";
import type { ConsoleMessage, Page, TestInfo } from "@playwright/test";
import yaml from "js-yaml";

/**
 * Formats a console message to remove noisy CSS styling directives from WASM logs.
 * Example input: "%cINFO%c src/main.rs:10%c Log message color: red..."
 * Example output: "INFO src/main.rs:10 Log message"
 */
export async function formatConsoleMessage(
  msg: ConsoleMessage,
): Promise<string> {
  const text = msg.text();
  if (text.includes("%c")) {
    try {
      const args = await Promise.all(msg.args().map((arg) => arg.jsonValue()));
      if (args.length > 0 && typeof args[0] === "string") {
        // tracing-wasm usually puts the entire format string in the first arg
        // and styles in subsequent args.
        // We just want the text from the first arg, stripped of %c.
        return args[0].replace(/%c/g, "");
      }
    } catch (_e) {
      // Fallback if we can't get args (e.g. handle disposed)
      return text;
    }
  }
  return text;
}

/**
 * Captures a synthetic DOM snapshot on test failure and writes it to disk.
 * The output is a YAML tree optimized for AI agent debugging, prioritizing
 * semantic elements (data-testid, role, aria-*) while stripping layout noise.
 */
export async function dumpFailureContext(page: Page, testInfo: TestInfo) {
  try {
    const syntheticTree = await page.evaluate(() => {
      // CONFIG: Attributes we trust for unique identification
      const EXTRA_RELEVANT_ATTRIBUTES = ["id", "name", "placeholder", "role"];

      // HELPER: Should this element appear in the Agent's view?
      function isRelevant(el: Element): boolean {
        const tag = el.tagName.toLowerCase();

        // 1. Always keep interactive elements (even if hidden, we want to know they are there)
        if (
          [
            "button",
            "a",
            "input",
            "select",
            "textarea",
            "details",
            "summary",
          ].includes(tag)
        )
          return true;

        // 2. Keep semantic headers
        if (/^h[1-6]$/.test(tag)) return true;

        // 3. Keep elements with any data-*, aria-*, or extra relevant attributes
        const attributes = el.getAttributeNames();
        if (
          attributes.some(
            (name) =>
              name.startsWith("data-") ||
              name.startsWith("aria-") ||
              EXTRA_RELEVANT_ATTRIBUTES.includes(name),
          )
        ) {
          return true;
        }

        // 4. Keep elements with interaction hints (event handlers)
        if (attributes.some((name) => name.startsWith("on"))) {
          return true;
        }

        // 5. Keep elements with direct text content
        const hasDirectText = Array.from(el.childNodes).some(
          (n) =>
            n.nodeType === Node.TEXT_NODE &&
            (n.textContent || "").trim().length > 0,
        );

        return hasDirectText;
      }

      interface SerializedNode {
        tag: string;
        children?: SerializedNode[];
        text?: string;
        value?: string;
        isVisible: boolean;
        rect?: { x: number; y: number; width: number; height: number };
        reason?: string;
        [key: string]: unknown;
      }

      // RECURSIVE SERIALIZER
      function serialize(el: Element): SerializedNode {
        const node: SerializedNode = {
          tag: el.tagName.toLowerCase(),
          isVisible: false,
        };

        // A. Capture Attributes
        // 1. Explicitly trusted attributes
        for (const attr of EXTRA_RELEVANT_ATTRIBUTES) {
          if (el.hasAttribute(attr)) node[attr] = el.getAttribute(attr) || "";
        }
        // 2. Dynamic data-* and aria-* attributes
        for (const name of el.getAttributeNames()) {
          if (
            (name.startsWith("data-") || name.startsWith("aria-")) &&
            !EXTRA_RELEVANT_ATTRIBUTES.includes(name)
          ) {
            node[name] = el.getAttribute(name) || "";
          }
        }

        // B. Capture Visibility & Geometry
        const rect = el.getBoundingClientRect();
        const style = window.getComputedStyle(el);

        const metrics = getVisibilityMetrics(el, rect, style);
        node.isVisible = metrics.isVisible;
        node.rect = {
          x: Math.round(rect.x),
          y: Math.round(rect.y),
          width: Math.round(rect.width),
          height: Math.round(rect.height),
        };

        if (!node.isVisible) {
          node.reason = metrics.reasons.join(",");
        }

        // C. Capture Content (Value/Text)
        captureContent(el, node);

        // D. Process Children
        processChildren(el, node);

        return node;
      }

      function getVisibilityMetrics(
        el: Element,
        rect: DOMRect,
        style: CSSStyleDeclaration,
      ) {
        const hasSize = rect.width > 0 && rect.height > 0;
        const isStyleVisible =
          style.display !== "none" &&
          style.visibility !== "hidden" &&
          style.opacity !== "0";
        const isNotHidden = el.getAttribute("aria-hidden") !== "true";
        const isWithinViewport =
          rect.bottom > 0 &&
          rect.right > 0 &&
          rect.top < window.innerHeight &&
          rect.left < window.innerWidth;

        const isVisible =
          hasSize && isStyleVisible && isNotHidden && isWithinViewport;
        const reasons: string[] = [];
        if (!hasSize) reasons.push("zero-size");
        if (!isStyleVisible) reasons.push("style-hidden");
        if (!isNotHidden) reasons.push("aria-hidden");
        if (!isWithinViewport) reasons.push("out-of-viewport");

        return { isVisible, reasons };
      }

      function captureContent(el: Element, node: SerializedNode) {
        if (
          el instanceof HTMLInputElement ||
          el instanceof HTMLTextAreaElement
        ) {
          if (el.value) node.value = el.value;
        }

        let directText = "";
        for (const child of Array.from(el.childNodes)) {
          if (child.nodeType === Node.TEXT_NODE)
            directText += child.textContent || "";
        }
        directText = directText.replace(/\s+/g, " ").trim();
        if (directText) node.text = directText;
      }

      function processChildren(el: Element, node: SerializedNode) {
        const childrenNodes: SerializedNode[] = [];
        for (const child of Array.from(el.children)) {
          const serializedChild = serialize(child);

          if (isRelevant(child)) {
            childrenNodes.push(serializedChild);
          } else if (serializedChild.children) {
            childrenNodes.push(...serializedChild.children);
          }
        }

        if (childrenNodes.length > 0) node.children = childrenNodes;
      }

      return serialize(document.body);
    });

    const yamlContent = yaml.dump(syntheticTree, {
      lineWidth: -1,
      noRefs: true,
    });
    const markdownContent = `# Failure Context: ${testInfo.title}

**URL:** ${page.url()}

## Synthetic DOM Snapshot

\`\`\`yaml
${yamlContent}
\`\`\`

---
*This snapshot shows the DOM state at the time of failure, optimized for debugging.*
`;

    // Write to disk and attach to test report (mirrors Playwright's error-context.md pattern)
    const filePath = testInfo.outputPath("synthetic-dom.md");
    await writeFile(filePath, markdownContent, "utf-8");
    await testInfo.attach("synthetic-dom.md", {
      path: filePath,
      contentType: "text/markdown",
    });
    console.log(`Saved synthetic DOM to: ${filePath}`);
  } catch (e) {
    console.error("Failed to generate synthetic DOM:", e);
  }
}
