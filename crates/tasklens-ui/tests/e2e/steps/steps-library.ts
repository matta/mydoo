import type { Page } from "@playwright/test";
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

    uploadsDocument: async (filePath: string) => {
      await this.plan.uploadDocument(filePath);
    },

    expandsTask: async (title: string) => {
      await this.plan.toggleExpand(title, true);
    },
  };

  public Then = {
    taskIsVisible: async (title: string) => {
      await this.plan.verifyTaskVisible(title);
    },
  };
}
