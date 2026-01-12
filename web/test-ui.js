const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

  console.log('=== Testing Roea AI POC ===\n');

  // Test Tasks tab
  console.log('1. Testing Tasks tab...');
  await page.goto('http://localhost:3000');
  await page.waitForTimeout(1000);
  await page.screenshot({ path: 'screenshot-tasks.png', fullPage: true });
  console.log('   Screenshot saved: screenshot-tasks.png');

  // Test Agents tab
  console.log('2. Testing Agents tab...');
  await page.click('text=Agents');
  await page.waitForTimeout(1000);
  await page.screenshot({ path: 'screenshot-agents.png', fullPage: true });
  console.log('   Screenshot saved: screenshot-agents.png');
  const agentContent = await page.textContent('main');
  console.log('   Agents page shows:', agentContent.slice(0, 200));

  // Test Monitor tab
  console.log('3. Testing Monitor tab...');
  await page.click('text=Monitor');
  await page.waitForTimeout(1000);
  await page.screenshot({ path: 'screenshot-monitor.png', fullPage: true });
  console.log('   Screenshot saved: screenshot-monitor.png');

  // Test creating a new task
  console.log('4. Testing New Task modal...');
  await page.click('text=Tasks');
  await page.waitForTimeout(500);
  await page.click('text=New Task');
  await page.waitForTimeout(500);
  await page.screenshot({ path: 'screenshot-new-task.png', fullPage: true });
  console.log('   Screenshot saved: screenshot-new-task.png');

  await browser.close();
  console.log('\n=== All tests completed successfully! ===');
})();
