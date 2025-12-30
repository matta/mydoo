import {createBdd} from 'playwright-bdd';
import {expect, test} from '../fixtures';

const {Given, When, Then} = createBdd(test);

Given('the user is on a document', async ({plan, documentContext}) => {
  await plan.primeWithSampleData();
  const docId = await plan.getCurrentDocumentId();
  expect(docId).toBeTruthy();
  documentContext.previousDocId = docId;
});

When('the user creates a new document', async ({plan}) => {
  await plan.createNewDocument();
});

Then('the document ID changes', async ({plan, documentContext}) => {
  const newId = await plan.getCurrentDocumentId();
  expect(newId).not.toBe(documentContext.previousDocId);
});

Then('the new document is empty', async ({plan}) => {
  await plan.switchToPlanView();
  await plan.verifyTaskHidden('Project Alpha');
});
