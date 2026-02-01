import { expect, type Page } from "@playwright/test";
import type { PlanPage } from "../pages/plan-page";
import { parseDuration } from "../utils/duration-parser";

export class Steps {
  private documentIds: Map<string, string> = new Map();

  constructor(
    private plan: PlanPage,
    private page: Page,
  ) {}

  public Given = {
    documentExists: async () => {
      await this.When.createsNewDocument();
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
      await this.When.createTask(title);
    },

    currentTimeIs: async (isoTime: string) => {
      await this.plan.setClock(new Date(isoTime));
    },

    seededWithSampleData: async () => {
      await this.plan.primeWithSampleData();
    },

    onMobileDevice: async () => {
      // Intentionally empty as project config handles this.
    },

    taskWithChild: async (parent: string, child: string) => {
      await this.When.createTask(parent);
      await this.When.addsChild(parent, child);
      await this.plan.closeEditor();
    },

    taskAsChildOf: async (child: string, parent: string) => {
      await this.When.addsChild(parent, child);
      await this.plan.closeEditor();
    },

    createsRoutineTask: async (
      title: string,
      repeatStr: string,
      leadTimeStr: string,
    ) => {
      const repeat = parseDuration(repeatStr);
      const lead = parseDuration(leadTimeStr);

      await this.plan.createRoutineTask(title, {
        frequency:
          repeat.uiUnit === "Days" ? "daily" : repeat.uiUnit.toLowerCase(),
        interval: repeat.value,
        leadTimeVal: lead.value,
        leadTimeUnit: lead.uiUnit,
      });
    },

    addsChildTask: async (childTitle: string, parentTitle: string) => {
      await this.When.addsChildTask(childTitle, parentTitle);
    },

    marksTaskAsSequential: async (title: string) => {
      await this.When.marksTaskAsSequential(title);
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

    createsFirstTask: async (title: string) => {
      await this.plan.addFirstTask(title);
    },

    addsSiblingTo: async (sibling: string, target: string) => {
      await this.plan.addSibling(target, sibling);
    },

    completesTask: async (title: string) => {
      await this.plan.completeTask(title);
    },

    clearsCompletedTasks: async () => {
      await this.plan.clearCompletedTasks();
    },

    refreshesDoList: async () => {
      await this.plan.refreshDoList();
    },

    waits: async (durationStr: string) => {
      const parts = durationStr.split(" ");
      const val = parseInt(parts[0] || "0", 10);
      const unit = parts[1]?.toLowerCase() || "";

      let minutes = 0;
      if (unit.startsWith("minute")) minutes = val;
      else if (unit.startsWith("hour")) minutes = val * 60;
      else if (unit.startsWith("day")) minutes = val * 60 * 24;

      await this.plan.advanceTime(minutes);
    },

    completesTaskFromDoList: async (title: string) => {
      await this.When.completesTask(title);
    },

    switchesToDocument: async (docName: string) => {
      const id = this.documentIds.get(docName);
      if (!id) throw new Error(`Document ${docName} ID not found in library`);
      await this.plan.switchToDocument(id);
    },

    opensCreateTaskModal: async () => {
      await this.plan.openCreateTaskModal();
    },

    opensTaskEditor: async (title: string) => {
      await this.plan.openTaskEditor(title);
    },

    addsChild: async (parent: string, child: string) => {
      await this.plan.openTaskEditor(parent);
      await this.plan.addChild(child);
    },

    opensAddChildModal: async (parent: string) => {
      await this.plan.openTaskEditor(parent);
      await this.page.getByRole("button", { name: "Add Child" }).click();
    },

    setDueDate: async (taskTitle: string, dateStr: string) => {
      await this.plan.openTaskEditor(taskTitle);
      await this.plan.setTaskDueDate(dateStr);
      await this.plan.closeEditor();
    },

    setLeadTime: async (taskTitle: string, leadTimeStr: string) => {
      await this.plan.openTaskEditor(taskTitle);
      const lead = parseDuration(leadTimeStr);
      await this.plan.setTaskLeadTime(lead.value, lead.uiUnit);
      await this.plan.closeEditor();
    },

    addsChildTask: async (childTitle: string, parentTitle: string) => {
      await this.plan.switchToPlanView();
      await this.plan.openTaskEditor(parentTitle);
      await this.plan.addChild(childTitle);
      await this.plan.closeEditor();
    },

    marksTaskAsSequential: async (title: string) => {
      await this.plan.setSequential(title, true);
    },

    drillsDownInto: async (title: string) => {
      await this.plan.mobileDrillDown(title);
    },

    navigatesUpLevel: async () => {
      await this.plan.mobileNavigateUpLevel();
    },

    findsInPlan: async (title: string) => {
      await this.plan.findInPlan(title);
    },

    renamesTask: async (oldTitle: string, newTitle: string) => {
      await this.plan.editTaskTitle(oldTitle, newTitle);
    },

    deletesTask: async (title: string) => {
      await this.plan.deleteTask(title);
    },

    reloadsPage: async () => {
      await this.page.reload();
      await this.plan.waitForAppReady();
    },

    movesTaskTo: async (child: string, target: string) => {
      await this.plan.openMovePicker(child);
      await this.plan.moveTaskTo(target);
    },

    opensMovePickerFor: async (title: string) => {
      await this.plan.openMovePicker(title);
    },

    setsImportance: async (title: string, value: number) => {
      await this.plan.openTaskEditor(title);
      await this.plan.setImportance(value);
      await this.plan.closeEditor();
    },

    setsEffort: async (title: string, value: number) => {
      await this.plan.openTaskEditor(title);
      await this.plan.setEffort(value);
      await this.plan.closeEditor();
    },

    setsNotes: async (title: string, notes: string) => {
      await this.plan.openTaskEditor(title);
      await this.plan.setNotes(notes);
      await this.plan.closeEditor();
    },

    // Balance View
    switchToBalanceView: async () => {
      await this.plan.switchToBalanceView();
    },

    adjustsDesiredCredits: async (title: string, value: number) => {
      await this.plan.setDesiredCredits(title, value);
    },
  };

  public Then = {
    taskIsVisible: async (title: string) => {
      await this.plan.verifyTaskVisible(title);
    },

    taskIsHidden: async (title: string) => {
      await this.plan.verifyTaskHidden(title);
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

    taskHasUrgency: async (taskTitle: string, urgency: string) => {
      await this.plan.verifyTaskUrgency(taskTitle, urgency);
    },

    shouldSeeMarkedAsCompleted: async (title: string) => {
      await this.plan.verifyTaskCompleted(title);
    },

    taskIsDue: async (taskTitle: string, dateText: string) => {
      if (["Tomorrow", "Yesterday", "Today"].includes(dateText)) {
        await this.plan.verifyDueDateText(taskTitle, dateText);
      } else {
        await this.plan.verifyDueDateTextContains(taskTitle, dateText);
      }
    },

    shouldSeeMobileBottomBar: async () => {
      await this.plan.mobileVerifyMobileBottomBar();
    },

    viewTitleShouldBe: async (title: string) => {
      await this.plan.mobileVerifyViewTitle(title);
    },

    shouldSeeInBreadcrumbs: async (title: string) => {
      await expect(
        this.page.getByRole("button", { name: title }),
      ).toBeVisible();
    },

    shouldBeInPlanView: async () => {
      await expect(this.page).toHaveURL(/\/plan/);
    },

    shouldSeeInPlanView: async (title: string) => {
      await this.plan.switchToPlanView();
      await this.plan.verifyTaskVisible(title);
    },

    shouldSeeDisabledOrHiddenInMovePicker: async (title: string) => {
      await this.plan.verifyMovePickerExcludes(title);
    },

    importanceShouldBe: async (title: string, value: string) => {
      await this.plan.openTaskEditor(title);
      await this.plan.verifyImportance(value);
      await this.plan.closeEditor();
    },

    effortShouldBe: async (title: string, value: string) => {
      await this.plan.openTaskEditor(title);
      await this.plan.verifyEffort(value);
      await this.plan.closeEditor();
    },

    notesShouldBe: async (title: string, notes: string) => {
      await this.plan.openTaskEditor(title);
      await this.plan.verifyNotes(notes);
      await this.plan.closeEditor();
    },

    // Balance View
    balanceItemIsVisible: async (title: string) => {
      await this.plan.verifyBalanceItemVisible(title);
    },

    balanceItemHasStatus: async (
      title: string,
      status: "Starving" | "Balanced",
    ) => {
      await this.plan.verifyBalanceStatus(title, status);
    },

    balanceItemIsStarving: async (title: string) => {
      await this.plan.verifyBalanceItemStarving(title, true);
    },

    balanceItemIsBalanced: async (title: string) => {
      await this.plan.verifyBalanceItemStarving(title, false);
    },

    balanceViewIsEmpty: async () => {
      const count = await this.plan.getBalanceItemCount();
      expect(count).toBe(0);
      await expect(this.page.getByText("No goals to balance.")).toBeVisible();
    },

    balanceItemCount: async (expected: number) => {
      const count = await this.plan.getBalanceItemCount();
      expect(count).toBe(expected);
    },

    // Task ordering in Do view
    taskAppearsBeforeInDoList: async (
      firstTask: string,
      secondTask: string,
    ) => {
      await this.plan.verifyTaskAppearsBeforeInDoList(firstTask, secondTask);
    },

    taskIsAtPosition: async (title: string, position: number) => {
      const actualPosition = await this.plan.getTaskPosition(title);
      expect(actualPosition).toBe(position);
    },
  };
}
