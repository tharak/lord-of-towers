import { expect } from '@playwright/test';

export async function controlPosition(status, key) {
  let position;
  await expect.poll(async () => {
    const positions = JSON.parse(await status.getAttribute('data-control-positions') || '{}');
    position = positions[key];
    return Array.isArray(position) && position.length === 2
      && position.every(value => Number.isFinite(value) && value > 0);
  }).toBe(true);
  return position;
}

export async function clickControl(page, status, key) {
  await page.mouse.click(...await controlPosition(status, key));
}
