import { test, expect } from '@playwright/test';

test('battlefield loads, spawns armies, and spectator controls work', async ({ page }, testInfo) => {
  test.setTimeout(180_000);
  const errors = [];
  page.on('console', message => { if (message.type() === 'error') console.log('Browser console:', message.text()); });
  page.on('pageerror', error => {
    if (!error.message.includes('Using exceptions for control flow')) errors.push(error.message);
  });
  page.on('response', response => {
    if (response.status() >= 400) errors.push(`${response.status()} ${response.url()}`);
  });
  await page.goto('./');
  const status = page.locator('#game-status');
  try { await expect(status).toHaveAttribute('data-ready', 'true'); }
  catch (error) { console.log('Startup error panel:', await page.locator('#error').textContent()); throw error; }
  await expect(page.locator('#loading')).toBeHidden();
  await expect(page.locator('#error')).toBeHidden();
  await expect(status).toHaveAttribute('data-towers', '5');
  await expect(status).toHaveAttribute('data-raw-deposits', '30');
  await expect(status).toHaveAttribute('data-tower-beacons', '5');
  await expect.poll(async () => Number(await status.getAttribute('data-minions'))).toBeGreaterThanOrEqual(15);
  await expect.poll(async () => Number(await status.getAttribute('data-stack-pieces'))).toBeGreaterThan(150);
  const initialRaw = Number(await status.getAttribute('data-raw-initial'));
  expect(initialRaw).toBeGreaterThanOrEqual(150);
  expect(initialRaw).toBeLessThanOrEqual(300);
  await expect.poll(async () => Number(await status.getAttribute('data-links'))).toBeGreaterThan(0);
  await page.mouse.click(720, 450);
  await expect.poll(async () => Number(await status.getAttribute('data-attack-trails'))).toBe(1);
  await page.locator('#game').focus();
  await page.keyboard.press('Space');
  await expect(status).toHaveAttribute('data-paused', 'true');
  const elapsed = Number(await status.getAttribute('data-elapsed'));
  const pulses = await status.getAttribute('data-energy-pulses');
  const minions = Number(await status.getAttribute('data-minions'));
  const energy = Number(await status.getAttribute('data-total-minion-energy'));
  expect(energy).toBeGreaterThanOrEqual(0);
  expect(energy).toBeLessThanOrEqual(minions * 5);
  await expect(status).toHaveAttribute('data-energy-orbs', String(energy));
  expect(Number(await status.getAttribute('data-links'))).toBeLessThanOrEqual(minions);
  await page.waitForTimeout(800);
  expect(Number(await status.getAttribute('data-elapsed'))).toBe(elapsed);
  await expect(status).toHaveAttribute('data-energy-pulses', pulses);
  await page.keyboard.press('Digit3');
  await expect(status).toHaveAttribute('data-speed', '4');
  await page.keyboard.press('F3');
  await expect(status).toHaveAttribute('data-inspector', 'true');
  await page.screenshot({ path: testInfo.outputPath('inspector.png') });
  await page.keyboard.press('F3');
  await expect(status).toHaveAttribute('data-inspector', 'false');
  // Exercise the actual ECS button, not just the keyboard shortcut.
  await page.mouse.click(75, 900 - 42);
  await expect(status).toHaveAttribute('data-paused', 'false');
  await expect.poll(async () => Number(await status.getAttribute('data-energy-pulses'))).toBeGreaterThan(Number(pulses));
  await expect.poll(async () => Number(await status.getAttribute('data-trails'))).toBeGreaterThan(0);
  await expect.poll(async () => Number(await status.getAttribute('data-raw-spent')), { timeout: 120_000 }).toBeGreaterThanOrEqual(5);
  await page.keyboard.press('Space');
  await expect(status).toHaveAttribute('data-paused', 'true');
  const raw = await status.evaluate(el => Object.fromEntries(
    ['remaining', 'carried', 'stock', 'spent', 'initial'].map(key => [key, Number(el.getAttribute(`data-raw-${key}`))])
  ));
  expect(raw.remaining + raw.carried + raw.stock + raw.spent).toBe(raw.initial);
  expect(raw.initial).toBe(initialRaw);
  const trails = await status.getAttribute('data-trails');
  await page.waitForTimeout(800);
  await expect(status).toHaveAttribute('data-trails', trails);
  await expect(status).toHaveAttribute('data-donut-visuals', String(raw.carried + raw.stock));
  await expect(status).toHaveAttribute('data-energy-orbs', await status.getAttribute('data-total-minion-energy'));
  await page.screenshot({ path: testInfo.outputPath('gathering-trails.png') });
  await page.keyboard.press('KeyR');
  await expect(status).toHaveAttribute('data-speed', '1');
  await expect(status).toHaveAttribute('data-energy-pulses', '0');
  await expect(status).toHaveAttribute('data-raw-deposits', '30');
  await expect(status).toHaveAttribute('data-minions', '15');
  await page.keyboard.press('Space');
  await expect(status).toHaveAttribute('data-paused', 'true');
  const restartedRaw = await status.evaluate(el => Object.fromEntries(
    ['remaining', 'carried', 'stock', 'spent'].map(key => [key, Number(el.getAttribute(`data-raw-${key}`))])
  ));
  expect(restartedRaw.remaining + restartedRaw.carried + restartedRaw.stock + restartedRaw.spent).toBe(initialRaw);
  await expect(status).toHaveAttribute('data-donut-visuals', String(restartedRaw.carried + restartedRaw.stock));
  await expect(status).toHaveAttribute('data-attack-trails', '0');
  await expect(status).toHaveAttribute('data-tower-beacons', '5');
  await expect(status).toHaveAttribute('data-energy-orbs', await status.getAttribute('data-total-minion-energy'));
  await expect(status).toHaveAttribute('data-towers', '5');
  await page.setViewportSize({ width: 1100, height: 700 });
  await expect.poll(async () => page.locator('#game').evaluate(el => el.clientWidth)).toBe(1100);
  await page.screenshot({ path: testInfo.outputPath('battlefield.png') });
  console.log('Final error panel:', await page.locator('#error').textContent());
  await expect(page.locator('#error')).toBeHidden();
  expect(errors).toEqual([]);
});

test('startup displays an actionable error without WebGL2', async ({ page }) => {
  await page.addInitScript(() => {
    const original = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function(type, ...args) {
      return type === 'webgl2' ? null : original.call(this, type, ...args);
    };
  });
  await page.goto('./');
  await expect(page.locator('#error')).toBeVisible();
  await expect(page.locator('#error')).toContainText('WebGL2 is unavailable');
});
