import {createBdd} from 'playwright-bdd';
import {expect, test} from '../fixtures';

const {Given, When, Then} = createBdd(test);

Given('the user is on a document', async ({plan, documentContext}) => {
  await plan.primeWithSampleData();
  const docUrl = await plan.getCurrentDocumentId();
  if (!docUrl) throw new Error('Failed to get document ID');
  documentContext.documents.set('original', docUrl);
});

When('the user creates a new document', async ({plan}) => {
  await plan.createNewDocument();
});

Then('the document ID changes', async ({plan, documentContext}) => {
  const newId = await plan.getCurrentDocumentId();
  expect(newId).not.toBe(documentContext.documents.get('original'));
});

Then('the new document is empty', async ({plan}) => {
  await plan.switchToPlanView();
  await plan.verifyTaskHidden('Project Alpha');
});

Given(
  'a document {string} with task {string}',
  async ({plan, documentContext}, name: string, task: string) => {
    // We assume we are already on a document, or we create one if it's the first time
    if (documentContext.documents.size === 0) {
      await plan.primeWithSampleData();
    } else {
      await plan.createNewDocument();
    }

    await plan.createTask(task);
    const docUrl = await plan.getCurrentDocumentId();
    if (!docUrl) throw new Error('Failed to get document URL');
    documentContext.documents.set(name, docUrl);
  },
);

When(
  'the user switches to document {string} by its ID',
  async ({plan, documentContext}, name: string) => {
    const docUrl = documentContext.documents.get(name);
    if (!docUrl) throw new Error(`Document ${name} not found in context`);
    await plan.switchToDocument(docUrl);
  },
);

Then(
  'the document ID should be the ID of {string}',
  async ({plan, documentContext}, name: string) => {
    const expectedUrl = documentContext.documents.get(name);
    const actualUrl = await plan.getCurrentDocumentId();
    expect(actualUrl).toBe(expectedUrl);
  },
);

Then('the task {string} should be visible', async ({plan}, task: string) => {
  await plan.switchToPlanView();
  await plan.verifyTaskVisible(task);
});
