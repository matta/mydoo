// Generated from: tests/e2e/features/document-switching.feature
import { test } from "../../../../fixtures.ts";

test.describe('Document Switching', () => {

  test('User creates a new document', async ({ Given, When, Then, And, documentContext, plan }) => { 
    await Given('the user is on a document', null, { documentContext, plan }); 
    await When('the user creates a new document', null, { plan }); 
    await Then('the document ID changes', null, { documentContext, plan }); 
    await And('the new document is empty', null, { plan }); 
  });

});

// == technical section ==

test.use({
  $test: [({}, use) => use(test), { scope: 'test', box: true }],
  $uri: [({}, use) => use('tests/e2e/features/document-switching.feature'), { scope: 'test', box: true }],
  $bddFileData: [({}, use) => use(bddFileData), { scope: "test", box: true }],
});

const bddFileData = [ // bdd-data-start
  {"pwTestLine":6,"pickleLine":3,"tags":[],"steps":[{"pwStepLine":7,"gherkinStepLine":4,"keywordType":"Context","textWithKeyword":"Given the user is on a document","stepMatchArguments":[]},{"pwStepLine":8,"gherkinStepLine":5,"keywordType":"Action","textWithKeyword":"When the user creates a new document","stepMatchArguments":[]},{"pwStepLine":9,"gherkinStepLine":6,"keywordType":"Outcome","textWithKeyword":"Then the document ID changes","stepMatchArguments":[]},{"pwStepLine":10,"gherkinStepLine":7,"keywordType":"Outcome","textWithKeyword":"And the new document is empty","stepMatchArguments":[]}]},
]; // bdd-data-end