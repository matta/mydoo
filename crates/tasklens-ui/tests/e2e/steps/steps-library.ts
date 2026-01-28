import { expect, type Page } from "@playwright/test";
import type { PlanPage } from "../pages/plan-page";

export class Steps {
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
  };
}
