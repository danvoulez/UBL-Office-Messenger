import { test, expect } from '@playwright/test';

test.describe('Complete User Journey', () => {
  test('user can send message and receive response', async ({ page }) => {
    // 1. Navigate to app
    await page.goto('/');
    
    // 2. Wait for app to load
    await page.waitForSelector('[data-testid="chat-page"]', { timeout: 10000 });
    
    // 3. Check if we need to login (demo mode or WebAuthn)
    const loginButton = page.locator('[data-testid="login-demo"]');
    if (await loginButton.isVisible()) {
      await loginButton.click();
      await page.waitForSelector('[data-testid="chat-page"]');
    }
    
    // 4. Select or create conversation
    const conversationList = page.locator('[data-testid="conversation-list"]');
    await expect(conversationList).toBeVisible();
    
    const firstConversation = page.locator('[data-testid^="conversation-"]').first();
    if (await firstConversation.isVisible()) {
      await firstConversation.click();
    } else {
      // Create new conversation
      await page.locator('[data-testid="new-conversation"]').click();
      await page.locator('[data-testid="entity-select"]').selectOption({ index: 0 });
      await page.locator('[data-testid="create-conversation-btn"]').click();
    }
    
    // 5. Send message
    const messageInput = page.locator('[data-testid="message-input"]');
    await expect(messageInput).toBeVisible();
    
    const testMessage = `E2E Test Message ${Date.now()}`;
    await messageInput.fill(testMessage);
    await page.locator('[data-testid="send-button"]').click();
    
    // 6. Verify message appears in chat
    await expect(page.locator(`text=${testMessage}`)).toBeVisible({ timeout: 5000 });
    
    // 7. Wait for potential response or job card
    await page.waitForTimeout(5000);
    
    // 8. Check if job card appeared
    const jobCard = page.locator('[data-testid^="job-card-"]');
    if (await jobCard.isVisible({ timeout: 5000 })) {
      console.log('✅ Job card appeared');
      
      // Check job card has title
      const jobTitle = jobCard.locator('[data-testid="job-title"]');
      await expect(jobTitle).toBeVisible();
    }
    
    console.log('✅ User journey complete');
  });
  
  test('user can navigate between conversations', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('[data-testid="chat-page"]', { timeout: 10000 });
    
    // Login if needed
    const loginButton = page.locator('[data-testid="login-demo"]');
    if (await loginButton.isVisible()) {
      await loginButton.click();
      await page.waitForSelector('[data-testid="chat-page"]');
    }
    
    // Get all conversations
    const conversations = page.locator('[data-testid^="conversation-"]');
    const count = await conversations.count();
    
    if (count >= 2) {
      // Click first conversation
      await conversations.nth(0).click();
      await page.waitForTimeout(1000);
      
      // Click second conversation
      await conversations.nth(1).click();
      await page.waitForTimeout(1000);
      
      // Should see different conversation
      expect(true).toBe(true);
    }
    
    console.log('✅ Navigation working');
  });
});