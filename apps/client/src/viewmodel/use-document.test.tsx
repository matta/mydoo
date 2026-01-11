import {
  type AutomergeUrl,
  type DocumentId,
  generateAutomergeUrl,
  Repo,
} from "@automerge/automerge-repo";
import type { TunnelState } from "@mydoo/tasklens/persistence";
import { createEmptyTunnelState } from "@mydoo/tasklens/test";
import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { createTestWrapper } from "../test/setup";

import { useDocument } from "./use-document";

describe("useDocument", () => {
  let repo: Repo;
  let docUrl: AutomergeUrl;

  beforeEach(() => {
    // Setup repo
    repo = new Repo({
      network: [], // No network for tests
    });

    // Create a document so the middleware can find it
    const handle = repo.create(createEmptyTunnelState());
    docUrl = handle.url;

    // Clear localStorage
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it("should create a new document if no ID in storage", async () => {
    const wrapper = createTestWrapper(repo, docUrl);
    const { result } = renderHook(() => useDocument(), { wrapper });
    if (!result.current) throw new Error("Document ID not found");

    // Wait for effect to create document and update state
    await waitFor(() => {
      expect(result.current).toBeTruthy();
    });

    // Storage should be updated
    expect(localStorage.getItem("mydoo:doc_id")).toBe(result.current);

    // Document should be initialized
    const handle = await repo.find<TunnelState>(
      result.current as string as DocumentId,
    );

    await handle.whenReady();
    const doc = handle.doc();
    expect(doc.nextTaskId).toBe(1);
  });

  it("should use existing ID from storage if present", async () => {
    const existingId = generateAutomergeUrl();
    localStorage.setItem("mydoo:doc_id", existingId);

    const wrapper = createTestWrapper(repo, docUrl);
    const { result } = renderHook(() => useDocument(), { wrapper });

    // Even if it's synchronous in this branch, it's safer to be consistent
    await waitFor(() => {
      expect(result.current).toBe(existingId);
    });
  });
});
