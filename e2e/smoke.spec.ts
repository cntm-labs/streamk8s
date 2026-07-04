import { test, expect } from '@playwright/test';

test('App loads and displays the IDE container', async ({ page }) => {
  // Mock Tauri APIs on the window object
  await page.addInitScript(() => {
    (window as any).__TAURI__ = {
      core: {
        invoke: async (cmd: string) => {
          if (cmd === 'get_available_contexts') {
            return [{ name: 'test-cluster', is_current: true }];
          }
          if (cmd === 'get_namespaces') {
            return ['default'];
          }
          if (cmd.startsWith('get_')) {
            return [];
          }
          return [];
        }
      },
      event: {
        listen: async () => {
          return () => {}; // Mock unlisten function
        }
      }
    };
  });

  await page.goto('/');

  // Check that the main IDE container is visible
  const ideContainer = page.locator('.ide-container');
  await expect(ideContainer).toBeVisible();

  // Check that either the Welcome View or the Main Area header is visible
  const headerOrWelcome = page.locator('.main-header, .welcome-container').first();
  await expect(headerOrWelcome).toBeVisible();
});
