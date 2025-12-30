// Generated from: tests/e2e/features/document-switching.feature
import { test } from "../../../../fixtures.ts";

test.describe('Document Switching', () => {

  test('User creates a new document', async ({ Given, When, Then, And, documentContext, plan }) => { 
    await Given('the user is on a document', null, { documentContext, plan }); 
    await When('the user creates a new document', null, { plan }); 
    await Then('the document ID changes', null, { documentContext, plan }); 
    await And('the new document is empty', null, { plan }); 
  });

  test('User switches to an existing document by ID', async ({ Given, When, Then, And, documentContext, plan }) => { 
    await Given('a document "A" with task "Task in A"', null, { documentContext, plan }); 
    await And('a document "B" with task "Task in B"', null, { documentContext, plan }); 
    await When('the user switches to document "A" by its ID', null, { documentContext, plan }); 
    await Then('the document ID should be the ID of "A"', null, { documentContext, plan }); 
    await And('the task "Task in A" should be visible', null, { plan }); 
    await When('the user switches to document "B" by its ID', null, { documentContext, plan }); 
    await Then('the document ID should be the ID of "B"', null, { documentContext, plan }); 
    await And('the task "Task in B" should be visible', null, { plan }); 
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
  {"pwTestLine":13,"pickleLine":9,"tags":[],"steps":[{"pwStepLine":14,"gherkinStepLine":10,"keywordType":"Context","textWithKeyword":"Given a document \"A\" with task \"Task in A\"","stepMatchArguments":[{"group":{"start":11,"value":"\"A\"","children":[{"start":12,"value":"A","children":[{"children":[]}]},{"children":[{"children":[]}]}]},"parameterTypeName":"string"},{"group":{"start":25,"value":"\"Task in A\"","children":[{"start":26,"value":"Task in A","children":[{"children":[]}]},{"children":[{"children":[]}]}]},"parameterTypeName":"string"}]},{"pwStepLine":15,"gherkinStepLine":11,"keywordType":"Context","textWithKeyword":"And a document \"B\" with task \"Task in B\"","stepMatchArguments":[{"group":{"start":11,"value":"\"B\"","children":[{"start":12,"value":"B","children":[{"children":[]}]},{"children":[{"children":[]}]}]},"parameterTypeName":"string"},{"group":{"start":25,"value":"\"Task in B\"","children":[{"start":26,"value":"Task in B","children":[{"children":[]}]},{"children":[{"children":[]}]}]},"parameterTypeName":"string"}]},{"pwStepLine":16,"gherkinStepLine":12,"keywordType":"Action","textWithKeyword":"When the user switches to document \"A\" by its ID","stepMatchArguments":[{"group":{"start":30,"value":"\"A\"","children":[{"start":31,"value":"A","children":[{"children":[]}]},{"children":[{"children":[]}]}]},"parameterTypeName":"string"}]},{"pwStepLine":17,"gherkinStepLine":13,"keywordType":"Outcome","textWithKeyword":"Then the document ID should be the ID of \"A\"","stepMatchArguments":[{"group":{"start":36,"value":"\"A\"","children":[{"start":37,"value":"A","children":[{"children":[]}]},{"children":[{"children":[]}]}]},"parameterTypeName":"string"}]},{"pwStepLine":18,"gherkinStepLine":14,"keywordType":"Outcome","textWithKeyword":"And the task \"Task in A\" should be visible","stepMatchArguments":[{"group":{"start":9,"value":"\"Task in A\"","children":[{"start":10,"value":"Task in A","children":[{"children":[]}]},{"children":[{"children":[]}]}]},"parameterTypeName":"string"}]},{"pwStepLine":19,"gherkinStepLine":15,"keywordType":"Action","textWithKeyword":"When the user switches to document \"B\" by its ID","stepMatchArguments":[{"group":{"start":30,"value":"\"B\"","children":[{"start":31,"value":"B","children":[{"children":[]}]},{"children":[{"children":[]}]}]},"parameterTypeName":"string"}]},{"pwStepLine":20,"gherkinStepLine":16,"keywordType":"Outcome","textWithKeyword":"Then the document ID should be the ID of \"B\"","stepMatchArguments":[{"group":{"start":36,"value":"\"B\"","children":[{"start":37,"value":"B","children":[{"children":[]}]},{"children":[{"children":[]}]}]},"parameterTypeName":"string"}]},{"pwStepLine":21,"gherkinStepLine":17,"keywordType":"Outcome","textWithKeyword":"And the task \"Task in B\" should be visible","stepMatchArguments":[{"group":{"start":9,"value":"\"Task in B\"","children":[{"start":10,"value":"Task in B","children":[{"children":[]}]},{"children":[{"children":[]}]}]},"parameterTypeName":"string"}]}]},
]; // bdd-data-end