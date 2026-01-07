import type {AutomergeUrl} from '@automerge/automerge-repo';
import {useDocHandle} from '@automerge/automerge-repo-react-hooks';
import {AppShell, Burger, Button, Group, Menu, Title} from '@mantine/core';
import {useDisclosure, useMediaQuery} from '@mantine/hooks';
import {type TunnelState, useTaskActions} from '@mydoo/tasklens';
import {
  IconCheckbox,
  IconDotsVertical,
  IconDownload,
  IconListTree,
  IconNetwork,
  IconScale,
  IconSeeding,
} from '@tabler/icons-react';
import {seedHierarchicalData} from '../../dev/seed-data';
import {DoViewContainer} from '../../viewmodel/containers/do-view-container';
import {MovePickerContainer} from '../../viewmodel/containers/move-picker-container';
import {TaskEditorContainer} from '../../viewmodel/containers/task-editor-container';
import {useNavigationState} from '../../viewmodel/ui/use-navigation-state';
import {BalanceViewContainer} from '../views/balance/balance-view-container';
import {PlanViewContainer} from '../views/plan/plan-view-container';
import {ConnectionModal} from './connection-modal';

// Height of the header and footer constants for consistent layout
const HEADER_HEIGHT = 60;
const FOOTER_HEIGHT = 60;

/**
 * Props for the AppShellContainer.
 */
interface AppShellContainerProps {
  /** The current Automerge document URL. */
  docUrl: AutomergeUrl;
}

/**
 * The main application shell component that provides the persistent layout structure.
 */
export function AppShellContainer({docUrl}: AppShellContainerProps) {
  // Global navigation state (Do vs Plan)
  const {activeTab, setActiveTab} = useNavigationState();

  // Mobile drawer state (Burger menu)
  const [mobileNavOpened, {toggle: toggleMobileNav}] = useDisclosure();

  const [
    connectionModalOpened,
    {open: openConnectionModal, close: closeConnectionModal},
  ] = useDisclosure(false);

  // Access actions for the Dev Tools menu actions (e.g. Seeding)
  const actions = useTaskActions();

  // Get the document handle to access the data for download
  const handle = useDocHandle<TunnelState>(docUrl);

  // Responsive Breakpoint: 768px (sm)
  const isDesktop = useMediaQuery('(min-width: 768px)');

  const handleReset = () => {
    localStorage.removeItem('mydoo:doc_id');
    window.location.href = '/';
  };

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

          {/* Options Menu */}
          <Menu shadow="md" width={200}>
            <Menu.Target>
              <Button
                variant="subtle"
                size="sm"
                px={4}
                leftSection={<IconDotsVertical size={20} />}
              >
                Options
              </Button>
            </Menu.Target>

            <Menu.Dropdown>
              <Menu.Label>General</Menu.Label>
              <Menu.Item
                leftSection={<IconNetwork size={14} />}
                onClick={openConnectionModal}
              >
                Connection
              </Menu.Item>
              <Menu.Item
                leftSection={<IconDownload size={14} />}
                onClick={() => {
                  if (!handle) return;
                  const doc = handle.doc();
                  if (!doc) return;
                  const blob = new Blob([JSON.stringify(doc, null, 2)], {
                    type: 'application/json',
                  });
                  const url = URL.createObjectURL(blob);
                  const a = document.createElement('a');
                  a.href = url;
                  a.download = `mydoo-backup-${new Date().toISOString()}.json`;
                  a.click();
                  URL.revokeObjectURL(url);
                }}
              >
                Download JSON
              </Menu.Item>

              {import.meta.env.DEV && (
                <>
                  <Menu.Divider />
                  <Menu.Label>Development</Menu.Label>
                  <Menu.Item
                    leftSection={<IconSeeding size={14} />}
                    onClick={() => seedHierarchicalData(actions)}
                  >
                    Seed Data
                  </Menu.Item>
                </>
              )}

              <Menu.Divider />
              <Menu.Label>
                Build: {__BUILD_INFO__.hash}
                {!__BUILD_INFO__.clean && ' (dirty)'}
                {import.meta.env.DEV && ' (dev)'}
                <div style={{fontWeight: 400, opacity: 0.5}}>
                  {new Date(__BUILD_INFO__.date).toLocaleString()}
                </div>
              </Menu.Label>
            </Menu.Dropdown>
          </Menu>
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
          mb="xs"
        >
          Plan
        </Button>
        <Button
          justify="flex-start"
          variant={activeTab === 'balance' ? 'light' : 'subtle'}
          leftSection={<IconScale size={20} />}
          onClick={() => {
            setActiveTab('balance');
            toggleMobileNav();
          }}
        >
          Balance
        </Button>
      </AppShell.Navbar>

      <AppShell.Main>
        {activeTab === 'do' && <DoViewContainer />}
        {activeTab === 'plan' && <PlanViewContainer />}
        {activeTab === 'balance' && <BalanceViewContainer />}
        <TaskEditorContainer />
        <MovePickerContainer />
        <ConnectionModal
          opened={connectionModalOpened}
          onClose={closeConnectionModal}
          currentUrl={docUrl}
          onReset={handleReset}
          onConnect={(url) => {
            localStorage.setItem('mydoo:doc_id', url);
            window.location.reload();
          }}
        />
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
          <Button
            flex={1}
            variant={activeTab === 'balance' ? 'light' : 'subtle'}
            radius={0}
            h="100%"
            onClick={() => setActiveTab('balance')}
          >
            <Group gap={4} style={{flexDirection: 'column'}}>
              <IconScale size={20} />
              <span style={{fontSize: '10px'}}>Balance</span>
            </Group>
          </Button>
        </AppShell.Footer>
      )}
    </AppShell>
  );
}
