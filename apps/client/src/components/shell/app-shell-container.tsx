import {AppShell, Burger, Button, Group, Menu, Title} from '@mantine/core';
import {useDisclosure, useMediaQuery} from '@mantine/hooks';
import {useTaskActions} from '@mydoo/tasklens';
import {
  IconCheckbox,
  IconDotsVertical,
  IconListTree,
  IconSeeding,
} from '@tabler/icons-react';
import {seedHierarchicalData} from '../../dev/seed-data';
import {DoViewContainer} from '../../viewmodel/containers/do-view-container';
import {MovePickerContainer} from '../../viewmodel/containers/move-picker-container';
import {TaskEditorContainer} from '../../viewmodel/containers/task-editor-container';
import {useNavigationState} from '../../viewmodel/ui/use-navigation-state';
import {PlanViewContainer} from '../views/plan/plan-view-container';

// Height of the header and footer constants for consistent layout
const HEADER_HEIGHT = 60;
const FOOTER_HEIGHT = 60;

/**
 * The main application shell component that provides the persistent layout structure.
 */
export function AppShellContainer() {
  // Global navigation state (Do vs Plan)
  const {activeTab, setActiveTab} = useNavigationState();

  // Mobile drawer state (Burger menu)
  const [mobileNavOpened, {toggle: toggleMobileNav}] = useDisclosure();

  // Access actions for the Dev Tools menu actions (e.g. Seeding)
  const actions = useTaskActions();

  // Responsive Breakpoint: 768px (sm)
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
        collapsed: !!isDesktop,
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
                  onClick={() => seedHierarchicalData(actions)}
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
        {activeTab === 'do' && <DoViewContainer />}
        {activeTab === 'plan' && <PlanViewContainer />}
        <TaskEditorContainer />
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
