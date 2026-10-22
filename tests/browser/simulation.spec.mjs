import { test, expect } from '@playwright/test';
import { clickControl } from './controls.mjs';

test('shared trails, tower growth, and centered controls work', async ({ page }, testInfo) => {
  test.setTimeout(180_000);
  const errors = [];
  page.on('pageerror', error => {
    if (!error.message.includes('Using exceptions for control flow')) errors.push(error.message);
  });
  page.on('response', response => {
    if (response.status() >= 400) errors.push(String(response.status()) + ' ' + response.url());
  });
  await page.goto('./');
  const status = page.locator('#game-status');
  try { await expect(status).toHaveAttribute('data-ready', 'true'); }
  catch (error) { console.log('Startup error panel:', await page.locator('#error').textContent()); throw error; }
  await expect(page.locator('#loading')).toBeHidden();
  await expect(page.locator('#error')).toBeHidden();
  // Rendering may finish after gathering begins; check a live simulation.
  await expect(status).toHaveAttribute('data-towers', '5');
  await expect(status).toHaveAttribute('data-player-towers', '1');
  const minions = Number(await status.getAttribute('data-minions'));
  expect(minions).toBeGreaterThan(0);
  expect(minions).toBeLessThanOrEqual(15);
  const deposits = Number(await status.getAttribute('data-raw-deposits'));
  expect(deposits).toBeGreaterThanOrEqual(0);
  expect(deposits).toBeLessThanOrEqual(30);
  await expect(status).toHaveAttribute('data-tower-beacons', '5');
  await expect(status).toHaveAttribute('data-recharge-areas', '5');
  expect(Number(await status.getAttribute('data-max-tower-radius'))).toBeGreaterThanOrEqual(15);
  await expect(status).toHaveAttribute('data-hud-visible', 'true');
  const initialRaw = Number(await status.getAttribute('data-raw-initial'));
  expect(initialRaw).toBeGreaterThanOrEqual(150);
  expect(initialRaw).toBeLessThanOrEqual(300);
  const startupRaw = await status.evaluate(el =>
    ['remaining', 'carried', 'stock'].map(key => Number(el.getAttribute('data-raw-' + key)))
  );
  expect(startupRaw.reduce((sum, amount) => sum + amount, 0)).toBe(initialRaw);

  await page.mouse.click(720, 450);
  await expect(status).toHaveAttribute('data-drawn-trails', '1');
  await page.mouse.move(500, 600);
  await page.mouse.down();
  await page.waitForTimeout(100);
  await page.mouse.move(900, 600, { steps: 30 });
  await page.waitForTimeout(100);
  await page.mouse.up();
  await expect.poll(async () => Number(await status.getAttribute('data-drawn-trails'))).toBeGreaterThan(2);
  const drawn = Number(await status.getAttribute('data-drawn-trails'));
  expect(drawn).toBeLessThanOrEqual(10);
  await page.mouse.click(1100, 450, { button: 'right' });
  await expect(status).toHaveAttribute('data-drawn-trails', String(drawn));
  await page.mouse.click(720, 450, { button: 'right' });
  await expect(status).toHaveAttribute('data-drawn-trails', String(drawn - 1));
  await page.mouse.click(720, 600, { button: 'right' });
  await expect(status).toHaveAttribute('data-drawn-trails', '0');

  for (let i = 0; i < 12; i++) {
    await page.mouse.click(720, 450);
    await expect(status).toHaveAttribute('data-drawn-trails', String(Math.min(i + 1, 10)));
  }
  for (let remaining = 9; remaining >= 0; remaining--) {
    await page.mouse.click(720, 450, { button: 'right' });
    await expect(status).toHaveAttribute('data-drawn-trails', String(remaining));
  }

  await clickControl(page, status, 'hud');
  await expect(status).toHaveAttribute('data-hud-visible', 'false');
  await page.setViewportSize({ width: 1100, height: 700 });
  await expect.poll(async () => page.locator('#game').evaluate(el => el.clientWidth)).toBe(1100);
  await page.waitForTimeout(100);
  await expect(status).toHaveAttribute('data-hud-visible', 'false');
  await clickControl(page, status, 'hud');
  await expect(status).toHaveAttribute('data-hud-visible', 'true');
  const controls = JSON.parse(await status.getAttribute('data-control-positions'));
  expect(controls).not.toHaveProperty('entities');
  await page.locator('#game').focus();
  await page.keyboard.press('F3');
  await expect(status).toHaveAttribute('data-inspector', 'true');
  await page.screenshot({ path: testInfo.outputPath('inspector.png') });
  await page.keyboard.press('F3');
  await expect(status).toHaveAttribute('data-inspector', 'false');
  await page.keyboard.press('Digit3');
  await expect.poll(async () => Number(await status.getAttribute('data-raw-stock')), { timeout: 120_000 }).toBeGreaterThan(0);
  await expect.poll(async () => Number(await status.getAttribute('data-max-tower-radius'))).toBeGreaterThan(15);
  expect(Number(await status.getAttribute('data-minions'))).toBeLessThanOrEqual(15);
  const raw = await status.evaluate(el => Object.fromEntries(
    ['remaining', 'carried', 'stock', 'initial'].map(key => [key, Number(el.getAttribute('data-raw-' + key))])
  ));
  expect(raw.remaining + raw.carried + raw.stock).toBe(raw.initial);
  expect(raw.initial).toBe(initialRaw);
  // Read inventory and its visuals together while the simulation continues.
  await expect.poll(() => status.evaluate(el => {
    const value = key => Number(el.getAttribute('data-' + key));
    return [
      value('donut-visuals') === value('raw-carried') + value('raw-stock'),
      value('energy-orbs') === value('total-minion-energy'),
    ];
  })).toEqual([true, true]);
  await page.screenshot({ path: testInfo.outputPath('tower-recharge-areas.png') });
  await clickControl(page, status, 'hud');
  await page.locator('#game').focus();
  await page.keyboard.press('KeyR');
  await expect(status).toHaveAttribute('data-speed', '1');
  await expect(status).toHaveAttribute('data-minions', '15');
  await expect(status).toHaveAttribute('data-drawn-trails', '0');
  await expect(status).toHaveAttribute('data-max-tower-radius', '15');
  await expect(status).toHaveAttribute('data-recharge-areas', '5');
  await expect(status).toHaveAttribute('data-hud-visible', 'false');
  await clickControl(page, status, 'hud');
  await expect(status).toHaveAttribute('data-hud-visible', 'true');
  await page.screenshot({ path: testInfo.outputPath('battlefield.png') });
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
