import {useRegisterSW} from 'virtual:pwa-register/react';
import {Button} from '@mantine/core';
import {notifications} from '@mantine/notifications';
import {useEffect, useRef} from 'react';

/**
 * PWA update prompt component using Mantine Notifications.
 *
 * Shows a notification in the top-right corner when a new service worker
 * version is detected and waiting. The user can click "Reload" to activate
 * the new version.
 *
 * NOTE: In dev mode, the SW only regenerates when the dev server restarts,
 * not on HMR updates. To test: restart dev server, then reload browser.
 */
export function ReloadPrompt() {
  const notificationShownRef = useRef(false);

  const {
    needRefresh: [needRefresh],
    updateServiceWorker,
  } = useRegisterSW({
    onRegistered(r) {
      if (r) {
        console.log('[SW] Registered:', r.scope);
      }
    },
    onRegisterError(error) {
      console.error('[SW] Registration error:', error);
    },
  });

  useEffect(() => {
    if (needRefresh && !notificationShownRef.current) {
      notificationShownRef.current = true;
      notifications.show({
        id: 'sw-update',
        title: 'Update Available',
        message: (
          <Button size="xs" mt="xs" onClick={() => updateServiceWorker(true)}>
            Reload to update
          </Button>
        ),
        autoClose: false,
        withCloseButton: true,
        color: 'blue',
      });
    }

    // Cleanup: hide notification if component unmounts (e.g., during testing
    // or if the app structure changes). The notification ID ensures we only
    // hide our specific notification.
    return () => {
      if (notificationShownRef.current) {
        notifications.hide('sw-update');
      }
    };
  }, [needRefresh, updateServiceWorker]);

  // This component doesn't render anything visible - the notification
  // is shown via the Notifications system
  return null;
}
