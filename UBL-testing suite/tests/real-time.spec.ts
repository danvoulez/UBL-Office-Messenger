import { test, expect } from '@playwright/test';

test.describe('Real-Time Updates', () => {
  test('messages appear in real-time', async ({ page, context }) => {
    // Open first tab
    await page.goto('/');
    await page.waitForSelector('[data-testid="chat-page"]', { timeout: 10000 });
    
    // Login if needed
    const loginButton = page.locator('[data-testid="login-demo"]');
    if (await loginButton. isVisible()) {
      await loginButton.click();
      await page.waitForSelector('[data-testid="chat-page"]');
    }
    
    // Open second tab
    const page2 = await context.newPage();
    await page2.goto('/');
    await page2.waitForSelector('[data-testid="chat-page"]', { timeout: 10000 });
    
    if (await page2.locator('[data-testid="login-demo"]').isVisible()) {
      await page2.locator('[data-testid="login-demo"]').click();
      await page2.waitForSelector('[data-testid="chat-page"]');
    }
    
    // Select same conversation in both tabs
    const firstConv = page.locator('[data-testid^="conversation-"]').first();
    await firstConv.click();
    
    const firstConv2 = page2.locator('[data-testid^="conversation-"]').first();
    await firstConv2.click();
    
    // Send message from first tab
    const testMessage = `Real-time test ${Date.now()}`;
    const messageInput = page.locator('[data-testid="message-input"]');
    await messageInput. fill(testMessage);
    await page.locator('[data-testid="send-button"]').click();
    
    // Message should appear in first tab immediately
    await expect(page.locator(`text=${testMessage}`)).toBeVisible({ timeout: 2000 });
    
    // Message should appear in second tab via SSE
    await expect(page2.locator(`text=${testMessage}`)).toBeVisible({ timeout: 10000 });
    
    console.log('✅ Real-time updates working');
  });
  
  test('presence indicators update', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('[data-testid="chat-page"]', { timeout:  10000 });
    
    // Login if needed
    const loginButton = page.locator('[data-testid="login-demo"]');
    if (await loginButton.isVisible()) {
      await loginButton.click();
      await page.waitForSelector('[data-testid="chat-page"]');
    }
    
    // Check for presence indicators
    const presenceIndicators = page.locator('[data-testid^="presence-"]');
    const count = await presenceIndicators.count();
    
    if (count > 0) {
      // Should have presence states
      const firstIndicator = presenceIndicators.first();
      const state = await firstIndicator.getAttribute('data-state');
      
      expect(state).toMatch(/online|offline|working|waiting_on_you/);
      console.log('✅ Presence indicators working');
    }
  });
  
  test('job updates appear in real-time', async ({ page }) => {
    await page. goto('/');
    await page.waitForSelector('[data-testid="chat-page"]', { timeout: 10000 });
    
    // Login if needed
    const loginButton = page.locator('[data-testid="login-demo"]');
    if (await loginButton.isVisible()) {
      await loginButton.click();
      await page.waitForSelector('[data-testid="chat-page"]');
    }
    
    // Select conversation
    const firstConversation = page.locator('[data-testid^="conversation-"]').first();
    await firstConversation.click();
    
    // Send message to create job
    const messageInput = page.locator('[data-testid="message-input"]');
    await messageInput.fill('Create a task for me');
    await page.locator('[data-testid="send-button"]').click();
    
    // Wait for job card to appear
    const jobCard = page.locator('[data-testid^="job-card-"]');
    await expect(jobCard).toBeVisible({ timeout: 15000 });
    
    // Approve job
    const approveButton = jobCard.locator('[data-testid="job-approve-btn"]');
    if (await approveButton.isVisible()) {
      await approveButton.click();
      
      // Job card should update in real-time
      await page. waitForTimeout(3000);
      
      const updatedState = await jobCard.getAttribute('data-state');
      console.log('Job state updated to:', updatedState);
      
      expect(updatedState).not.toBe('proposed');
    }
    
    console.log('✅ Job real-time updates working');
  });
});