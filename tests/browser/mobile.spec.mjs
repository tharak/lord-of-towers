import { test, expect } from '@playwright/test';
import { controlPosition } from './controls.mjs';

// Wait for rendered frames between touch packets so Bevy observes each phase.
async function frames(page, count = 4) {
  await page.evaluate(async count => {
    for (let i = 0; i < count; i++) {
      await new Promise(resolve => requestAnimationFrame(resolve));
    }
  }, count);
}

async function touch(page, session, type, points) {
  await session.send('Input.dispatchTouchEvent', {
    type,
    touchPoints: points.map(([id, x, y]) => ({ id, x, y, radiusX: 1, radiusY: 1, force: 1 })),
  });
  await frames(page);
}

async function tap(page, session, x, y) {
  await touch(page, session, 'touchStart', [[0, x, y]]);
  await touch(page, session, 'touchEnd', []);
}

async function tapControl(page, session, status, key) {
  await tap(page, session, ...await controlPosition(status, key));
}

async function camera(status) {
  return status.evaluate(el => Object.fromEntries(
    ['scale', 'x', 'z'].map(key => [key, Number(el.getAttribute(`data-camera-${key}`))])
  ));
}

test('touch tools, camera gestures, and mobile controls survive rotation', async ({ page, context }, testInfo) => {
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
  await expect(status).toHaveAttribute('data-mobile-layout', 'true');
  await expect(status).toHaveAttribute('data-trail-tool', 'draw');
  await expect(status).toHaveAttribute('data-drawn-trails', '0');
  await expect(status).toHaveAttribute('data-recharge-areas', '5');
  const session = await context.newCDPSession(page);

  await tapControl(page, session, status, 'hud');
  await expect(status).toHaveAttribute('data-hud-visible', 'false');
  await tapControl(page, session, status, 'hud');
  await expect(status).toHaveAttribute('data-hud-visible', 'true');

  await expect(status).toHaveAttribute('data-drawn-trails', '0');
  await touch(page, session, 'touchStart', [[0, 150, 420]]);
  for (let x = 160; x <= 240; x += 10) {
    await touch(page, session, 'touchMove', [[0, x, 420]]);
  }
  await touch(page, session, 'touchEnd', []);
  await expect.poll(async () => Number(await status.getAttribute('data-drawn-trails'))).toBeGreaterThan(1);
  const drawn = await status.getAttribute('data-drawn-trails');
  expect(Number(drawn)).toBeLessThanOrEqual(10);
  await page.screenshot({ path: testInfo.outputPath('mobile-drawn-path.png') });

  await tapControl(page, session, status, 'erase');
  await expect(status).toHaveAttribute('data-trail-tool', 'erase');
  await expect(status).toHaveAttribute('data-drawn-trails', drawn);
  await tap(page, session, 340, 230);
  await expect(status).toHaveAttribute('data-drawn-trails', drawn);
  await tap(page, session, 195, 420);
  await expect(status).toHaveAttribute('data-drawn-trails', '0');
  await tapControl(page, session, status, 'draw');
  await expect(status).toHaveAttribute('data-trail-tool', 'draw');

  const beforePan = await camera(status);
  expect(Object.values(beforePan).every(Number.isFinite)).toBe(true);
  await touch(page, session, 'touchStart', [[0, 135, 360], [1, 235, 360]]);
  await touch(page, session, 'touchMove', [[0, 165, 390], [1, 265, 390]]);
  await touch(page, session, 'touchEnd', []);
  await expect.poll(async () => {
    const after = await camera(status);
    return Math.hypot(after.x - beforePan.x, after.z - beforePan.z);
  }).toBeGreaterThan(0.01);
  await expect(status).toHaveAttribute('data-drawn-trails', '0');

  const beforePinch = await camera(status);
  await touch(page, session, 'touchStart', [[0, 150, 360], [1, 240, 360]]);
  await touch(page, session, 'touchMove', [[0, 110, 360], [1, 280, 360]]);
  await expect.poll(async () => Math.abs((await camera(status)).scale - beforePinch.scale)).toBeGreaterThan(0.01);
  // A remaining finger must stay in gesture mode until every finger lifts.
  await touch(page, session, 'touchMove', [[0, 110, 360]]);
  await touch(page, session, 'touchMove', [[0, 170, 390]]);
  await touch(page, session, 'touchEnd', []);
  await expect(status).toHaveAttribute('data-drawn-trails', '0');
  await page.screenshot({ path: testInfo.outputPath('mobile-camera-gestures.png') });

  await tapControl(page, session, status, 'hud');
  await expect(status).toHaveAttribute('data-hud-visible', 'false');

  await page.setViewportSize({ width: 844, height: 390 });
  await expect.poll(async () => page.locator('#game').evaluate(el => el.clientWidth)).toBe(844);
  await frames(page);
  await expect(status).toHaveAttribute('data-mobile-layout', 'true');
  await expect(status).toHaveAttribute('data-hud-visible', 'false');
  await tapControl(page, session, status, 'hud');
  await expect(status).toHaveAttribute('data-hud-visible', 'true');
  await expect(status).toHaveAttribute('data-drawn-trails', '0');
  await page.screenshot({ path: testInfo.outputPath('mobile-landscape.png') });
  await expect(page.locator('#error')).toBeHidden();
  expect(errors).toEqual([]);
  await session.detach();
});
