import {AppShell, Burger, Button, Group, Menu, Title} from '@mantine/core';
import {useDisclosure, useMediaQuery} from '@mantine/hooks';
import type {DocumentHandle} from '@mydoo/tasklens';
import {useTunnel} from '@mydoo/tasklens';
import {
  IconCheckbox,
  IconDotsVertical,
  IconListTree,
  IconSeeding,
} from '@tabler/icons-react';
import {seedHierarchicalData} from '../../dev/SeedData';
import {DoViewContainer} from '../../viewmodel/containers/DoViewContainer';
import {MovePickerContainer} from '../../viewmodel/containers/move-picker-container';
import {TaskEditorContainer} from '../../viewmodel/containers/TaskEditorContainer';
import {useNavigationState} from '../../viewmodel/ui/useNavigationState';
import {PlanViewContainer} from '../views/plan/PlanViewContainer';

// Height of the header and footer constants for consistent layout
const HEADER_HEIGHT = 60;
const FOOTER_HEIGHT = 60;

/**
 * The main application shell component that provides the persistent layout structure.
 *
 * Responsibilities:
 * 1. **Layout Management**: Uses Mantine's `AppShell` to orchestrate the Header, Navbar (Sidebar),
 *    Main content area, and Footer via a responsive design.
 * 2. **Navigation State**: Connects to `useNavigationState` to switch between primary views
 *    ('Do' vs 'Plan') and manages the mobile navigation drawer state.
 * 3. **Responsive Adaptation**:
 *    - **Desktop**: Shows a persistent sidebar for navigation.
 *    - **Mobile**: Hides the sidebar (drawer) and shows a bottom tab bar for quick access.
 * 4. **Dev Tools**: In development mode, provides access to data seeding utilities.
 *
 * @param props.docUrl - The Automerge document handle, passed down to view containers.
 */
export function AppShellContainer({docUrl}: {docUrl: DocumentHandle}) {
  // Global navigation state (Do vs Plan)
  const {activeTab, setActiveTab} = useNavigationState();

  // Mobile drawer state (Burger menu)
  const [mobileNavOpened, {toggle: toggleMobileNav}] = useDisclosure();

  // Access ops for the Dev Tools menu actions (e.g. Seeding)
  const {ops} = useTunnel(docUrl);

  // Responsive Breakpoint: 768px (sm)
  // Used to conditionally render the bottom footer on mobile and toggle sidebar visibility logic.
  const isDesktop = useMediaQuery('(min-width: 768px)');

  return (
    <AppShell
      header={{height: HEADER_HEIGHT}}
      navbar={{
        width: 250,
        breakpoint: 'sm',
        collapsed: {mobile: !mobileNavOpened, desktop: false},
      }}
      // Footer is only for mobile tab bar
      footer={{
        height: FOOTER_HEIGHT,
        collapsed: !!isDesktop, // Show on mobile (collapsed=false), hide on desktop (collapsed=true)
      }}
      padding="md"
    >
      <AppShell.Header>
        <Group h="100%" px="md" justify="space-between">
          <Group>
            <Burger
              opened={mobileNavOpened}
              onClick={toggleMobileNav}
              hiddenFrom="sm"
              size="sm"
            />
            <Title order={3}>Mydoo</Title>
          </Group>

          {/* Dev Tools Menu: Only visible in development mode */}
          {import.meta.env.DEV && (
            <Menu shadow="md" width={200}>
              <Menu.Target>
                <Button
                  variant="subtle"
                  size="sm"
                  px={4}
                  leftSection={<IconDotsVertical size={20} />}
                >
                  Dev
                </Button>
              </Menu.Target>

              <Menu.Dropdown>
                <Menu.Label>Development</Menu.Label>
                <Menu.Item
                  leftSection={<IconSeeding size={14} />}
                  onClick={() => seedHierarchicalData(ops)}
                >
                  Seed Data
                </Menu.Item>
              </Menu.Dropdown>
            </Menu>
          )}
        </Group>
      </AppShell.Header>

      {/* Desktop Sidebar Navigation */}
      <AppShell.Navbar p="md">
        <Button
          justify="flex-start"
          variant={activeTab === 'do' ? 'light' : 'subtle'}
          leftSection={<IconCheckbox size={20} />}
          onClick={() => {
            setActiveTab('do');
            toggleMobileNav(); // Close mobile drawer if open
          }}
          mb="xs"
        >
          Do
        </Button>
        <Button
          justify="flex-start"
          variant={activeTab === 'plan' ? 'light' : 'subtle'}
          leftSection={<IconListTree size={20} />}
          onClick={() => {
            setActiveTab('plan');
            toggleMobileNav();
          }}
        >
          Plan
        </Button>
      </AppShell.Navbar>

      <AppShell.Main>
        {activeTab === 'do' && <DoViewContainer docUrl={docUrl} />}
        {activeTab === 'plan' && <PlanViewContainer docUrl={docUrl} />}
        <TaskEditorContainer docUrl={docUrl} />
        <MovePickerContainer />
      </AppShell.Main>

      {/* Mobile Bottom Tab Bar: Only visible on small screens */}
      {!isDesktop && (
        <AppShell.Footer p={0} style={{display: 'flex'}}>
          <Button
            flex={1}
            variant={activeTab === 'do' ? 'light' : 'subtle'}
            radius={0}
            h="100%"
            onClick={() => setActiveTab('do')}
          >
            <Group gap={4} style={{flexDirection: 'column'}}>
              <IconCheckbox size={20} />
              <span style={{fontSize: '10px'}}>Do</span>
            </Group>
          </Button>
          <Button
            flex={1}
            variant={activeTab === 'plan' ? 'light' : 'subtle'}
            radius={0}
            h="100%"
            onClick={() => setActiveTab('plan')}
          >
            <Group gap={4} style={{flexDirection: 'column'}}>
              <IconListTree size={20} />
              <span style={{fontSize: '10px'}}>Plan</span>
            </Group>
          </Button>
        </AppShell.Footer>
      )}
    </AppShell>
  );
}
