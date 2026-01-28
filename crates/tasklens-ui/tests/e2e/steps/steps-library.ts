import { expect, type Page } from "@playwright/test";
import type { PlanPage } from "../pages/plan-page";

export class Steps {
  private documentIds: Map<string, string> = new Map();

  constructor(
    private plan: PlanPage,
    private page: Page,
  ) {}

  public Given = {
    cleanWorkspace: async () => {
      await this.plan.setupClock();
      await this.page.goto("/");
      await this.page.evaluate(() => localStorage.clear());
      await this.page.reload();
      await this.plan.setupClock();
      await this.plan.waitForAppReady();
    },

    onHomePage: async () => {
      await this.page.goto("/");
      await this.plan.waitForAppReady();
    },

    documentExists: async () => {
      await this.plan.createNewDocument();
    },

    taskExistsInView: async (title: string, view: string) => {
      if (view === "Plan") {
        await this.plan.switchToPlanView();
      } else {
        await this.plan.switchToDoView();
      }
      await this.plan.createTask(title);
    },

    documentWithTask: async (docName: string, taskTitle: string) => {
      await this.plan.createNewDocument();
      const id = await this.plan.getCurrentDocumentId();
      if (!id) throw new Error("Could not get document ID");
      this.documentIds.set(docName, id);
      await this.plan.createTask(taskTitle);
    },

    taskExists: async (title: string) => {
      await this.plan.createTask(title);
    },
  };

  public When = {
    switchToDoView: async () => {
      await this.plan.switchToDoView();
    },

    switchToPlanView: async () => {
      await this.plan.switchToPlanView();
    },

    createTask: async (title: string) => {
      await this.plan.createTask(title);
    },

    createTaskInDoView: async (title: string) => {
      await this.plan.createTaskInDoView(title);
    },

    downloadsDocument: async () => {
      return await this.plan.downloadDocument();
    },

    clearsApplicationState: async () => {
      await this.page.evaluate(() => localStorage.clear());
      await this.page.reload();
      await this.plan.waitForAppReady();
    },

    uploadsDocument: async (filePath: string) => {
      await this.plan.uploadDocument(filePath);
    },

    expandsTask: async (title: string) => {
      await this.plan.toggleExpand(title, true);
    },

    opensSyncSettings: async () => {
      await this.plan.openSyncSettings();
    },

    closesSyncSettings: async () => {
      await this.plan.closeSyncSettings();
    },

    changesSyncServerUrl: async (url: string) => {
      await this.plan.setSyncServerUrl(url);
    },

    savesSyncSettings: async () => {
      await this.plan.saveSyncSettings();
    },

    createsNewDocument: async () => {
      await this.plan.createNewDocument();
    },

    switchesToDocument: async (docName: string) => {
      const id = this.documentIds.get(docName);
      if (!id) throw new Error(`Document ${docName} ID not found in library`);
      await this.plan.switchToDocument(id);
    },

    opensCreateTaskModal: async () => {
      await this.plan.openCreateTaskModal();
    },

    addsChild: async (parent: string, child: string) => {
      await this.plan.openTaskEditor(parent);
      await this.plan.addChild(child);
    },

    opensAddChildModal: async (parent: string) => {
      await this.plan.openTaskEditor(parent);
      await this.page.getByRole("button", { name: "Add Child" }).click();
    },
  };

  public Then = {
    taskIsVisible: async (title: string) => {
      await this.plan.verifyTaskVisible(title);
    },

    syncServerUrlShouldBe: async (url: string) => {
      await this.plan.verifySyncServerUrl(url);
    },

    documentIdShouldRemain: async (oldId: string | undefined) => {
      const newId = await this.plan.getCurrentDocumentId();
      expect(newId).toBe(oldId);
    },

    documentUrlShouldUseSchema: async (schema: string) => {
      const url = await this.plan.getDetailedDocumentUrl();
      expect(url).toContain(schema);
    },

    documentIdChanges: async (oldId: string | undefined) => {
      const newId = await this.plan.getCurrentDocumentId();
      expect(newId).not.toBe(oldId);
    },

    documentIdShouldBe: async (docName: string) => {
      const expectedId = this.documentIds.get(docName);
      const actualId = await this.plan.getCurrentDocumentId();
      expect(actualId).toBe(expectedId);
    },

    documentShouldBeEmpty: async () => {
      await expect(this.page.getByTestId("task-item")).toHaveCount(0);
    },

    getCurrentDocumentId: async () => {
      return await this.plan.getCurrentDocumentId();
    },

    pageTitleContains: async (text: string) => {
      await expect(this.page).toHaveTitle(new RegExp(text));
    },

    shouldSeeText: async (text: string) => {
      await expect(this.page.getByText(text).first()).toBeVisible();
    },

    shouldSeeLeadTime: async (val: string, unit: string) => {
      await this.plan.verifyFieldValue("Lead Time", val);
      await this.plan.verifyFieldValue("Lead Time Unit", unit);
    },

    shouldSeeSelector: async (label: string) => {
      await this.plan.verifyElementVisible(label);
    },
  };
}
