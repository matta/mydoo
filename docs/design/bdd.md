Reasoning about Gherkin Keywords To move away from an "ad hoc" feeling, it helps
to view Gherkin keywords through the lens of State Transitions.

Given (Setup/State): This puts the system into a known state. It represents the
"Pre-conditions."

When (Action/Transition): This is the key action the user performs. There should
generally be only one primary When per scenario to maintain focus.

Then (Observation/Assertion): This verifies the outcome. It represents the
"Post-conditions."

And / But: These are purely syntactic sugar to make the sentences readable. They
inherit the logic of the preceding keyword.

High-level, scalable Gherkin is Declarative; it describes what the user is
doing.Imperative (Low Level)Declarative (High Level)Given I enter "user1" in the
username fieldGiven I am logged in as a Premium MemberAnd I enter "password" in
the password fieldWhen I attempt to access the Gold LoungeAnd I click the
"Login" buttonThen I should be granted access

In complex Single Page Applications (SPAs), the traditional "One Class Per Page"
approach of the Page Object Model (POM) often breaks down because "pages" are no
longer discrete. Instead, you have persistent headers, dynamic sidebars, and
nested dialogs that appear and disappear without the URL ever changing.

To scale effectively, you should transition from Page Objects to a
Component-Based Architecture.

1. Move to Component Object Model (COM) Instead of one massive DashboardPage
   class, break the UI into independent, reusable "Component Objects." A complex
   SPA "Page" becomes a composition of these objects.

Page Object: Orchestrates high-level navigation and assembles components.

Component Object: Manages a specific slice of the UI (e.g., UserMenu, DataTable,
NotificationToast).

```js
// A component representing a reusable Modal/Dialog
class UserSettingsModal {
  constructor(rootSelector) {
    this.root = rootSelector;
  }
  async changeEmail(newEmail) {
    await fill(`${this.root} #email-input`, newEmail);
    await click(`${this.root} #save-btn`);
  }
}

// The Page Object assembles the components
class ProfilePage {
  constructor() {
    this.sidebar = new Sidebar('.nav-sidebar');
    this.settingsModal = new UserSettingsModal('.modal-container');
  }
```

Handling Dialogs and Popups Dialogs should be treated as Transient Components.
Since they often float on top of multiple pages, they shouldn't be "owned" by a
single Page Object.

Global Registry: Store common dialogs (Login, Confirmation, Error) in a shared
location.

Return Values: When a method on a Page Object opens a dialog, have it return an
instance of that Dialog component.

Example: const dialog = await dashboard.clickDeleteUser(); — Now your test has
the correct context to interact with the popup.

2. Model "State" Rather Than "URLs" In SPAs, the system's "state" is more
   important than its "location." To handle complex transitions (like a dialog
   that only opens after a specific sequence), your Gherkin and code should
   reflect these states.

Gherkin: Given the "Export Data" dialog is open

Implementation: The code shouldn't just click a button; it should verify the
dialog is visible before proceeding. This makes your tests resilient to race
conditions common in SPAs (where the JS hasn't finished rendering the modal).
