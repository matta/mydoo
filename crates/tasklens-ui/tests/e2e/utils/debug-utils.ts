import type { Page, TestInfo } from "@playwright/test";

/**
 * Serializes the current DOM into a simplified JSON structure optimized for
 * AI Agents. It prioritizes semantic elements and specific attributes
 * (data-testid, role) while stripping out layout noise.
 */
export async function dumpFailureContext(page: Page, testInfo: TestInfo) {
  console.log(`\n=== FAILURE CONTEXT: ${testInfo.title} ===`);
  console.log(`URL: ${page.url()}`);

  try {
    const syntheticTree = await page.evaluate(() => {
      // CONFIG: Attributes we trust for unique identification
      const RELEVANT_ATTRIBUTES = [
        "data-testid",
        "data-cy",
        "name",
        "id",
        "role",
        "aria-label",
        "placeholder",
      ];

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

        // 3. Keep elements with key testing attributes
        if (el.hasAttribute("data-testid") || el.hasAttribute("data-cy"))
          return true;

        // 4. Keep elements with interaction hints
        if (
          el.hasAttribute("data-expanded") ||
          el.hasAttribute("aria-label") ||
          el.hasAttribute("onclick")
        )
          return true;

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
        for (const attr of RELEVANT_ATTRIBUTES) {
          if (el.hasAttribute(attr)) node[attr] = el.getAttribute(attr) || "";
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

    console.log("--- SYNTHETIC DOM SNAPSHOT (Agent Optimized) ---");
    console.log(JSON.stringify(syntheticTree, null, 2));
    console.log("------------------------------------------------");
  } catch (e) {
    console.log("Failed to generate synthetic DOM:", e);
  }

  console.log("==============================================\n");
}
