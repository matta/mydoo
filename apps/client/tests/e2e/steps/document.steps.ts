import {createBdd} from 'playwright-bdd';
import {expect, test} from '../fixtures';

const {Given, When, Then} = createBdd(test);

Given('the user is on a document', async ({plan, documentContext}) => {
  await plan.primeWithSampleData();
  const docId = await plan.getCurrentDocumentId();
  if (!docId) throw new Error('Failed to get document ID');
  documentContext.documents.set('original', docId);
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
    const docId = await plan.getCurrentDocumentId();
    if (!docId) throw new Error('Failed to get document ID');
    documentContext.documents.set(name, docId);
  },
);

When(
  'the user switches to document {string} by its ID',
  async ({plan, documentContext}, name: string) => {
    const id = documentContext.documents.get(name);
    if (!id) throw new Error(`Document ${name} not found in context`);
    await plan.switchToDocument(id);
  },
);

Then(
  'the document ID should be the ID of {string}',
  async ({plan, documentContext}, name: string) => {
    const expectedId = documentContext.documents.get(name);
    const actualId = await plan.getCurrentDocumentId();
    expect(actualId).toBe(expectedId);
  },
);

Then('the task {string} should be visible', async ({plan}, task: string) => {
  await plan.switchToPlanView();
  await plan.verifyTaskVisible(task);
});
