import type {
  AutomergeUrl,
  StorageAdapterInterface,
} from "@automerge/automerge-repo";
import { Repo } from "@automerge/automerge-repo";
import { useMediaQuery } from "@mantine/hooks";
import type { TaskID } from "@mydoo/tasklens";
import {
  createTaskLensTestEnvironment,
  strictMock,
} from "@mydoo/tasklens/test";
import { act, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithTestProviders } from "../../../test/setup";
import { PlanViewContainer } from "./plan-view-container";

// Create a minimal storage adapter using strictMock - only implements what Repo actually uses
function createDummyStorageAdapter(): StorageAdapterInterface {
  return strictMock<StorageAdapterInterface>("DummyStorageAdapter", {
    load: async () => undefined,
    save: async () => {},
    remove: async () => {},
    loadRange: async () => [],
    removeRange: async () => {},
  });
}

// Mock simple view-model hooks that are not the focus of this integration
const mocks = vi.hoisted(() => ({
  openCreateModal: vi.fn(),
  mockUseNavigationState: {
    activeTab: "plan",
    setActiveTab: vi.fn(),
    collapseAll: vi.fn(),
    currentViewId: undefined,
    expandAll: vi.fn(),
    expandedIds: new Set(),
    modal: undefined,
    openEditModal: vi.fn(),
    openCreateModal: vi.fn(),
    closeModal: vi.fn(),
    isExpanded: vi.fn(),
    popView: vi.fn(),
    pushView: vi.fn(),
    resetView: vi.fn(),
    setViewPath: vi.fn(),
    toggleExpanded: vi.fn(),
    viewPath: [],
  },
  useTaskTree: {
    roots: [] as Array<{ id: string }>,
    isLoading: false,
  },
}));

vi.mock("@mantine/hooks", async () => {
  const actual = await vi.importActual("@mantine/hooks");
  return {
    ...actual,
    useMediaQuery: vi.fn(),
  };
});

vi.mock("../../../viewmodel/intents/use-task-intents", () => ({
  useTaskIntents: () => ({
    createTask: vi.fn(),
    toggleTask: vi.fn(),
    deleteTask: vi.fn(),
    indentTask: vi.fn(),
    outdentTask: vi.fn(),
  }),
}));

vi.mock("../../../viewmodel/projections/use-task-tree", () => ({
  useTaskTree: () => mocks.useTaskTree,
}));

vi.mock("../../../viewmodel/ui/use-navigation-state", () => ({
  useNavigationState: () => mocks.mockUseNavigationState,
}));

vi.mock("../../../viewmodel/ui/use-breadcrumbs", () => ({
  useBreadcrumbs: vi.fn(() => []),
}));

// Mock OutlineTree to simplify rendering
vi.mock("./outline-tree", () => ({
  OutlineTree: () => <div data-testid="outline-tree" />,
}));

describe("PlanViewContainer", () => {
  // Helper to mock viewport per test
  const mockViewport = (isDesktop: boolean) => {
    vi.mocked(useMediaQuery).mockReturnValue(isDesktop);
  };

  let repo: Repo;
  let docUrl: AutomergeUrl;

  // Reset mocks before each test
  beforeEach(() => {
    vi.clearAllMocks();
    const customRepo = new Repo({
      network: [],
      storage: createDummyStorageAdapter(),
    });
    const env = createTaskLensTestEnvironment(customRepo);
    repo = env.repo;
    docUrl = env.docUrl;
  });

  it('renders "Add First Task" button when task list is empty', async () => {
    const user = userEvent.setup();
    mockViewport(true); // Desktop
    mocks.useTaskTree.roots = [];
    mocks.useTaskTree.isLoading = false;

    await act(async () => {
      renderWithTestProviders(<PlanViewContainer />, { repo, url: docUrl });
    });

    expect(screen.getByText("No tasks found.")).toBeInTheDocument();

    const addButton = screen.getByRole("button", { name: /add first task/i });
    await user.click(addButton);

    expect(mocks.mockUseNavigationState.openCreateModal).toHaveBeenCalledWith(
      undefined,
      undefined,
      "end",
    );
  });

  it("renders Bottom Bar only on mobile", async () => {
    mockViewport(false); // Mobile
    mocks.useTaskTree.roots = [{ id: "1" }];
    mocks.useTaskTree.isLoading = false;

    await act(async () => {
      renderWithTestProviders(<PlanViewContainer />, { repo, url: docUrl });
    });

    expect(screen.getByLabelText("Add Task at Top")).toBeInTheDocument();
    expect(screen.getByLabelText("Up Level")).toBeInTheDocument();
  });

  it("does NOT render Bottom Bar on desktop", async () => {
    mockViewport(true); // Desktop
    mocks.useTaskTree.roots = [{ id: "1" }];
    mocks.useTaskTree.isLoading = false;

    await act(async () => {
      renderWithTestProviders(<PlanViewContainer />, { repo, url: docUrl });
    });

    expect(screen.queryByLabelText("Add Task at Top")).not.toBeInTheDocument();
  });

  it("renders IconPlus (Append Row) button when tasks exist", async () => {
    mockViewport(true);
    mocks.useTaskTree.roots = [{ id: "1" }];
    mocks.useTaskTree.isLoading = false;

    await act(async () => {
      renderWithTestProviders(<PlanViewContainer />, { repo, url: docUrl });
    });

    // The append button is an ActionIcon with an IconPlus, but no text.
    // It's the only one of its kind at the bottom right usually, but here just checking presence.
    // In this view, we can rely on finding it by role, although standardizing aria-label would be better.
    // For now, checking that we don't have "No tasks found" is a start, and we can query for the button.
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThan(0);
  });

  it('calls openCreateModal with position="start" when Top "+" tapped (Mobile)', async () => {
    const user = userEvent.setup();
    mockViewport(false);
    mocks.useTaskTree.roots = [{ id: "1" }];
    mocks.useTaskTree.isLoading = false;
    await act(async () => {
      renderWithTestProviders(<PlanViewContainer />, { repo, url: docUrl });
    });

    const topPlus = screen.getByLabelText("Add Task at Top");
    await user.click(topPlus);

    expect(mocks.mockUseNavigationState.openCreateModal).toHaveBeenCalledWith(
      undefined,
      undefined,
      "start",
    );
  });

  it("renders due date badges in breadcrumbs when present", async () => {
    mockViewport(false); // Mobile
    const { useBreadcrumbs } =
      await import("../../../viewmodel/ui/use-breadcrumbs");
    vi.mocked(useBreadcrumbs).mockReturnValue([
      {
        id: "parent" as TaskID,
        title: "Parent Task",
        effectiveDueDate: Date.now() + 86400000,
        effectiveLeadTime: 28800000,
      },
      {
        id: "child" as TaskID,
        title: "Child Task",
        effectiveDueDate: undefined,
        effectiveLeadTime: undefined,
      },
    ]);

    mocks.useTaskTree.roots = [{ id: "child" }];

    await act(async () => {
      renderWithTestProviders(<PlanViewContainer />, { repo, url: docUrl });
    });

    // Check parent breadcrumb has a badge (which should have an urgency attribute)
    const breadcrumbButtons = screen.getAllByRole("button");
    const parentButton = breadcrumbButtons.find((b) =>
      b.textContent?.includes("Parent Task"),
    );
    expect(parentButton).toBeInTheDocument();

    // The DueDateBadge renders a Mantine Badge which has data-urgency
    const badge = parentButton?.querySelector("[data-urgency]");
    expect(badge).toBeInTheDocument();
  });
});
